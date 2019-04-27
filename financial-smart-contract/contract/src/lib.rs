#![no_std]
#![allow(non_snake_case)]
#![feature(proc_macro_hygiene)]

// Prevent complaining about unused structs, some combinators may be unused validly
#![allow(dead_code)]

extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate pwasm_ethereum;
extern crate pwasm_std;
mod combinators;

use pwasm_abi::eth::EndpointInterface;
use pwasm_std::{ Box, Vec, types::{ Address, U256, H256 } };
use pwasm_abi_derive::eth_abi;
use combinators::*;

static CALL_GAS: i64 = 2300;

// Executed when the contract is called
#[no_mangle]
pub fn call() {
    // Dispatch contract call to contract endpoint with given input, return result
    let contract = FinancialScContract::new();
    let mut endpoint = FinancialScEndpoint::new(contract);
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

// Executed when the contract is deployed
#[no_mangle]
pub fn deploy() {
    // Dispatch contract constructor call with given input
    let contract = FinancialScContract::new();
    let mut endpoint = FinancialScEndpoint::new(contract);
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}

// An entry in the storage table
struct Entry {
    key: H256,
    value: [u8; 32]
}

// Storage table, stores anything looked up while the contract is executing, clears upon exiting contract code
struct Storage {
    table: Vec<Entry>
}

// The implementing struct can store values of the given type (passed/returned by value).
trait Stores<T> {
    // Reads a value of the given type from storage
    fn read(&mut self, key: &H256) -> T;

    // Writes a value of the given type to storage
    fn write(&mut self, key: &H256, value: T);
}

// Storage method implementation
impl Storage {
    // Initialise a new storage table
    fn new() -> Storage {
        Storage { table: Vec::new() }
    }

    // Read the length of a vector from storage
    fn read_len(&mut self, key: &H256) -> i64 {
        Storage::to_i64(&pwasm_ethereum::read(key))
    }

    // Write the length of a vector to storage
    fn write_len(&mut self, key: &H256, len: i64) {
        pwasm_ethereum::write(key, &Storage::from_i64(len));
    }
    
    // Convert a stored value into an address
    fn to_address(value: &[u8; 32]) -> Address {
        H256::from(value).into()
    }

    // Convert an address into a storable value
    fn from_address(value: Address) -> [u8; 32] {
        H256::from(value).into()
    }

    // Convert a storable value into an i64
    fn to_i64(value: &[u8; 32]) -> i64 {
        let mut res: i64;

        // Deserialize accounting for endianness
        let unsigned = pwasm_std::read_u64(value);

        // Convert u64 to i64
        if unsigned > 2_u64.pow(63) {
            // Convert from 2^63..2^64 - 1 to -(2^63 - 1)..0
            res = (unsigned - 2_u64.pow(63)) as i64;
            res -= 2_i64.pow(62);
            res -= 2_i64.pow(62);
        } else {
            // Unsigned int is in same range as signed int (most significant bit is 0)
            res = unsigned as i64
        }

        res
    }

    // Convert an i64 into a storable value
    fn from_i64(mut value: i64) -> [u8; 32] {
        // Convert to u64
        let unsigned;
        if value < 0 {
            // Convert from two's complement negative to unsigned
            value += 2_i64.pow(62);
            value += 2_i64.pow(62);
            unsigned = value as u64 + 2_u64.pow(63);
        } else {
            // Signed int is positive, convert straight to unsigned
            unsigned = value as u64;
        }
        
        // Serialize accounting for endianness
        let mut res: [u8; 32] = [0; 32];
        pwasm_std::write_u64(&mut res, unsigned);
        res
    }
}

impl Stores<[u8; 32]> for Storage {
    // Read a value from storage, store locally if not already
    fn read(&mut self, key: &H256) -> [u8; 32] {
        for entry in &self.table {
            if entry.key == *key {
                return entry.value.clone();
            }
        }

        let value = pwasm_ethereum::read(key);
        self.table.push(Entry {
            key: key.clone(),
            value: value.clone()
        });
        value
    }

    // Write a value to storage and store locally
    fn write(&mut self, key: &H256, value: [u8; 32]) {
        pwasm_ethereum::write(key, &value);

        for entry in &mut self.table {
            if entry.key == *key {
                entry.value = value.clone();
                return;
            }
        }

        self.table.push(Entry {
            key: key.clone(),
            value: value.clone()
        })
    }
}

impl Stores<Address> for Storage {
    fn read(&mut self, key: &H256) -> Address {
        let value: [u8; 32] = self.read(key);
        Storage::to_address(&value)
    }

    fn write(&mut self, key: &H256, value: Address) {
        self.write(key, Storage::from_address(value));
    }
}

impl Stores<i64> for Storage {
    fn read(&mut self, key: &H256) -> i64 {
        let value: [u8; 32] = self.read(key);
        Storage::to_i64(&value)
    }

    fn write(&mut self, key: &H256, value: i64) {
        self.write(key, Storage::from_i64(value));
    }
}

impl<T> Stores<Vec<T>> for Storage where Storage: Stores<T>, Vec<T>: core::clone::Clone {
    fn read(&mut self, key: &H256) -> Vec<T> {
        let length: i64 = self.read_len(key);
        let mut res = Vec::new();

        for i in 1..(length + 1) {
            res.push(self.read(&add_to_key(*key, i as u64)));
        }

        res
    }

    fn write(&mut self, key: &H256, value: Vec<T>) {
        let length = value.len();
        self.write_len(key, length as i64);
        let mut clone = value.clone();

        for i in 0..length {
            self.write(&add_to_key(*key, (i + 1) as u64), clone.remove(0));
        }
    }
}

// Adds a number to a storage key
fn add_to_key(key: H256, mut value: u64) -> H256 {
    let mut i = 0;
    let mut new_key: [u8; 32] = key.clone().into();
    while value > 0 {
        if value >= 255 - (new_key[i] as u64) {
            value -= 255 - (new_key[i] as u64);
            new_key[i] = 255;
            i += 1;
        } else {
            new_key[i] = new_key[i] + value as u8;
            value = 0;
        }

        if i >= 32 {
            panic!("Storage key overflow.");
        }
    }

    H256::from(new_key)
}

// The contract holder address storage key
fn holder_address_key() -> H256 {
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
}

// The contract counter-party address storage key
fn counter_party_address_key() -> H256 {
    H256::from([1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
}

// The contract holder balance storage key
fn holder_balance_key() -> H256 {
    H256::from([2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
}

// The contract counter-party balance storage key
fn counter_party_balance_key() -> H256 {
    H256::from([3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
}

// The serialized combinator contract storage key (first slot is length, the following are elements)
fn serialized_combinator_contract_key() -> H256 {
    H256::from([4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
}

// The financial smart contract interface
#[eth_abi(FinancialScEndpoint)]
pub trait FinancialScInterface {
    // The contract constructor, takes the combinator contract definition (serialized) and the holder address
    fn constructor(&mut self, contract_definition: Vec<i64>, holder: Address);

    // Gets the address of the contract holder
    #[constant]
    fn get_holder(&mut self) -> Address;

    // Gets the address of the counter-party
    #[constant]
    fn get_counter_party(&mut self) -> Address;

    // Gets the combinator contract definition, returns the combinator contract serialized
    #[constant]
    fn get_contract_definition(&mut self) -> Vec<i64>;

    // Gets the current value of the contract (TODO: for dev purposes)
    #[constant]
    fn get_value(&mut self) -> i64;

    // Gets the current balance of the caller (if called by the holder or counter-party)
    #[constant]
    fn get_balance(&mut self) -> i64;

    // Gets whether or not the contract has concluded all operation (i.e. updating will never change the balance).
    #[constant]
    fn get_concluded(&mut self) -> bool;

    // Sets the preference of the given or combinator's sub-combinators
    fn set_or_choice(&mut self, or_index: u64, choice: bool);

    // Proposes a value for the given observable
    fn propose_obs_value(&mut self, obs_index: u64, value: i64);

    // Acquires the combinator contract at the current block-time (when called by the holder)
    fn acquire(&mut self);

    // Updates the balances of the holder and counter-party
    fn update(&mut self);

    // Acquires an anytime combinator's sub-contract
    fn acquire_anytime_sub_contract(&mut self, anytime_index: u64);

    // Stakes Eth with the contract (can be called by the holder or counter-party), returns the caller's total balance
    #[payable]
    fn stake(&mut self) -> i64;

    // Withdraws positive Eth balance up to the given amount from the contract (can be called by the holder or counter-party)
    fn withdraw(&mut self, amount: u64) -> bool ;
}

// The financial smart contract
pub struct FinancialScContract {
    // The contract storage table
    storage: Storage,

    // // The serialized combinator contract
    // serialized_combinators: Vec<i64>,

    // The combinator contract
    combinator: Box<ContractCombinator>,

    // The choices for each or combinator
    or_choices: Vec<Option<bool>>,

    // The values of each observable proposed by the holder
    holder_proposed_obs_values: Vec<Option<i64>>,

    // The values of each observable proposed by the counter-party
    counter_party_proposed_obs_values: Vec<Option<i64>>,

    // The concrete values of each observable
    concrete_obs_values: Vec<Option<i64>>,

    // The acquisition times for each anytime operator
    anytime_acquisition_times: Vec<Option<u32>>
}

// The financial smart contract interface implementation
impl FinancialScInterface for FinancialScContract {
    // The financial smart contract constructor
    fn constructor(&mut self, contract_definition: Vec<i64>, holder: Address) {
        if holder == pwasm_ethereum::sender() {
            panic!("Holder and counter-party must be different addresses.");
        }
        if contract_definition.len() == 0 {
            panic!("Provided combinator contract not valid.");
        }

        // Initialise storage
        self.storage.write(&holder_address_key(), holder);
        self.storage.write(&counter_party_address_key(), pwasm_ethereum::sender());
        self.storage.write(&holder_balance_key(), 0);
        self.storage.write(&counter_party_balance_key(), 0);
        self.storage.write(&serialized_combinator_contract_key() , contract_definition);

        self.set_combinator();
    }

    // Gets the address of the holder
    fn get_holder(&mut self) -> Address {
        let holder_address: [u8; 32] = self.storage.read(&holder_address_key());
        H256::from(holder_address).into()
    }

    // Gets the address of the counter-party
    fn get_counter_party(&mut self) -> Address {
        let counter_party_address: [u8; 32] = self.storage.read(&counter_party_address_key());
        H256::from(counter_party_address).into()
    }

    // Gets the combinator contract definition (serialized)
    fn get_contract_definition(&mut self) -> Vec<i64> {
        self.storage.read(&serialized_combinator_contract_key())
    }

    // Gets the current value of the contract
    fn get_value(&mut self) -> i64 {
        self.combinator.get_value(pwasm_ethereum::timestamp() as u32, &self.or_choices, &self.concrete_obs_values, &self.anytime_acquisition_times)
    }

    // Gets the total balance of the caller
    fn get_balance(&mut self) -> i64 {
        let sender = pwasm_ethereum::sender();
        let holder: Address = self.storage.read(&holder_address_key());
        let counter_party: Address = self.storage.read(&counter_party_address_key());

        if sender == holder {
            self.storage.read(&holder_balance_key())
        } else if sender == counter_party {
            self.storage.read(&counter_party_balance_key())
        } else {
            panic!("Only the contract holder or the counter-party may have stake in the contract.");
        }
    }

    // Gets whether or not the contract has concluded.
    fn get_concluded(&mut self) -> bool {
        let combinator_details = self.combinator.get_combinator_details();
        combinator_details.fully_updated
            || combinator_details.acquisition_time == None && self.combinator.past_horizon(pwasm_ethereum::timestamp() as u32)
    }

    // Sets the given or combinator's preference between its sub-combinators
    fn set_or_choice(&mut self, or_index: u64, prefer_first: bool) {
        let holder: Address = self.storage.read(&holder_address_key());
        if pwasm_ethereum::sender() != holder {
            panic!("Only the contract holder may set or-choices.");
        }

        let index = or_index as usize;
        if index >= self.or_choices.len() {
            panic!("Given or-index does not exist.");
        }

        self.or_choices[index as usize] = Some(prefer_first);
    }

    // Proposes the given observable's value
    fn propose_obs_value(&mut self, obs_index: u64, value: i64) {
        // Check that index is within bounds
        let index: usize = obs_index as usize;
        if index >= self.concrete_obs_values.len() {
            panic!("Given observable-index does not exist.");
        }

        // Set the proposed value for the sender
        let sender = pwasm_ethereum::sender();
        let holder: Address = self.storage.read(&holder_address_key());
        let counter_party: Address = self.storage.read(&counter_party_address_key());

        if sender == holder {
            self.holder_proposed_obs_values[index] = Some(value);
        } else if sender == counter_party {
            self.counter_party_proposed_obs_values[index] = Some(value);
        } else {
            panic!("Only the holder or counter-party set the value of observables.");
        }

        // Check if proposed values match
        if self.holder_proposed_obs_values[index] == self.counter_party_proposed_obs_values[index] {
            self.concrete_obs_values[index] = self.holder_proposed_obs_values[index];
        }
    }

    // Acquires the combinator contract at the current block-time (when called by the holder)
    fn acquire(&mut self) {
        let holder: Address = self.storage.read(&holder_address_key());

        if pwasm_ethereum::sender() != holder {
            panic!("Only the contract holder may acquire the combinator contract.");
        } else if self.combinator.get_combinator_details().acquisition_time != None {
            panic!("The combinator contract cannot be acquired more than once.");
        }

        self.combinator.acquire(pwasm_ethereum::timestamp() as u32, &self.or_choices, &mut self.anytime_acquisition_times);
    }

    // Updates the balances of the holder and counter-party
    fn update(&mut self) {
        // If concluded, can't update.
        if self.get_concluded() {
            panic!("Contract has concluded, nothing more to update.");
        }

        let difference = self.combinator.update(pwasm_ethereum::timestamp() as u32, &self.or_choices, &self.concrete_obs_values, &mut self.anytime_acquisition_times);

        let counter_party_balance = self.storage.read(&counter_party_balance_key());
        self.storage.write(&counter_party_balance_key(), FinancialScContract::safe_add(counter_party_balance, -difference));

        let holder_balance = self.storage.read(&holder_balance_key());
        self.storage.write(&holder_balance_key(), FinancialScContract::safe_add(holder_balance, difference));
    }

    // Acquires an anytime combinator's sub-contract
    fn acquire_anytime_sub_contract(&mut self, anytime_index: u64) {
        let index = anytime_index as usize;
        if index >= self.anytime_acquisition_times.len() {
            panic!("Given anytime index does not exist.");
        }

        let holder: Address = self.storage.read(&holder_address_key());
        if pwasm_ethereum::sender() != holder {
            panic!("Only the contract holder may acquire the combinator contract.");
        }

        let prev_acquisition_time = self.anytime_acquisition_times[anytime_index as usize];
        let new_acquisition_time = pwasm_ethereum::timestamp() as u32;

        if prev_acquisition_time != None && prev_acquisition_time.unwrap() <= new_acquisition_time {
            panic!("Cannot acquire a sub-combinator contract which has already been acquired.");
        }

        self.anytime_acquisition_times[anytime_index as usize] = Some(new_acquisition_time);
    }

    // Stakes Eth with the contract, returns the caller's total balance
    fn stake(&mut self) -> i64 {
        let sender = pwasm_ethereum::sender();
        let stake = pwasm_ethereum::value();
        FinancialScContract::assert_U256_can_be_i64(stake);
        let holder: Address = self.storage.read(&holder_address_key());
        let counter_party: Address = self.storage.read(&counter_party_address_key());
        let key;

        // Check which party is enquiring
        if sender == holder {
            key = holder_balance_key();
        } else if sender == counter_party {
            key = counter_party_balance_key();
        } else {
            panic!("Only the contract holder or the counter-party may stake Ether in the contract.");
        }

        // Get the balance
        let mut balance = self.storage.read(&key);
        balance = FinancialScContract::safe_add(balance, stake.low_u64() as i64);
        self.storage.write(&key, balance);
        balance
    }

    // Withdraws positive Eth balance up to the given amount from the contract (can be called by the holder or counter-party)
    fn withdraw(&mut self, amount: u64) -> bool {
        let sender = pwasm_ethereum::sender();
        let final_amount;
        let original_balance;
        let key;
        let holder: Address = self.storage.read(&holder_address_key());
        let counter_party: Address = self.storage.read(&counter_party_address_key());
        
        // Get the amount to send (clamp at balance amount)
        if sender == holder {
            key = holder_balance_key();
        } else if sender == counter_party {
            key = counter_party_balance_key();
        } else {
            panic!("Only the contract holder or the counter-party may withdraw Ether from the contract.");
        }

        original_balance = self.storage.read(&key);
        final_amount = FinancialScContract::get_withdrawal_amount(amount, original_balance);
        self.storage.write(&key, original_balance - final_amount);

        if final_amount < CALL_GAS {
            return false;
        }

        let mut result = Vec::<u8>::new();
        match pwasm_ethereum::call(CALL_GAS as u64, &sender, U256::from(final_amount - CALL_GAS), &[], &mut result) {
            Ok(_v) => true,
            Err(_e) => {
                // Payment failed, roll-back balance
                self.storage.write(&key, original_balance);
                false
            }
        }
    }
}

// Financial smart contract functions which aren't part of the ABI
impl FinancialScContract {
    // Instantiates a new financial smart contract
    pub fn new() -> FinancialScContract {
        FinancialScContract{
            storage: Storage::new(),
            combinator: Box::new(NullCombinator::new()),
            or_choices: Vec::new(),
            holder_proposed_obs_values: Vec::new(),
            counter_party_proposed_obs_values: Vec::new(),
            concrete_obs_values: Vec::new(),
            anytime_acquisition_times: Vec::new()
        }
    }

    // Constructs the combinators from a serialized combinator contract
    fn set_combinator(&mut self) {
        let (_i, combinator) = self.deserialize_combinator(0);
        self.combinator = combinator;
    }

    // Deserializes a combinator from the given combinator byte vector and index, returns the following index and the boxed combinator
    fn deserialize_combinator(&mut self, i: usize) -> (usize, Box<ContractCombinator>) {
        let serialized_combinators: Vec<i64> = self.storage.read(&serialized_combinator_contract_key());
        if i >= serialized_combinators.len() {
            panic!("Provided combinator contract not valid.");
        }

        match serialized_combinators[i] {
            // zero combinator
            0 => (i + 1, Box::new(ZeroCombinator::new())),

            // one combinator
            1 => (i + 1, Box::new(OneCombinator::new())),

            // and combinator
            2 => {
                // Deserialize sub-combinators
                let (i0, sub_combinator0) = self.deserialize_combinator(i + 1);
                let (i1, sub_combinator1) = self.deserialize_combinator(i0);

                (i1, Box::new(AndCombinator::new(sub_combinator0, sub_combinator1)))
            },

            // or combinator
            3 => {
                // Keep track of or_index and or_choices
                let or_index: usize = self.or_choices.len();
                self.or_choices.push(None);

                // Deserialize sub-combinators
                let (i0, sub_combinator0) = self.deserialize_combinator(i + 1);
                let (i1, sub_combinator1) = self.deserialize_combinator(i0);

                (i1, Box::new(OrCombinator::new(sub_combinator0, sub_combinator1, or_index)))
            },

            // truncate combinator
            4 => {
                // Deserialize timestamp from 4 bytes to 32-bit int
                let mut timestamp: u32 = serialized_combinators[i + 1] as u32;

                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_combinator(i + 2);

                (i0, Box::new(TruncateCombinator::new(sub_combinator, timestamp)))
            },

            // scale combinator
            5 => {
                // Check if observable is provided, if so then deserialize it, otherwise record in obs_values
                let provided: i64 = serialized_combinators[i + 1];
                let mut obs_index: Option<usize>;
                let mut scale_value: Option<i64>;
                let mut i0 = i + 2;

                if provided == 1 {
                    obs_index = None;
                    scale_value = Some(serialized_combinators[i0]);
                    i0 += 1;
                } else {
                    obs_index = Some(self.concrete_obs_values.len());
                    self.concrete_obs_values.push(None);
                    self.holder_proposed_obs_values.push(None);
                    self.counter_party_proposed_obs_values.push(None);
                    scale_value = None;
                }

                // Deserialize sub-contract
                let (i1, sub_combinator) = self.deserialize_combinator(i0);

                (i1, Box::new(ScaleCombinator::new(sub_combinator, obs_index, scale_value)))
            },

            // give combinator
            6 => {
                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_combinator(i + 1);

                (i0, Box::new(GiveCombinator::new(sub_combinator)))
            },

            // then combinator
            7 => {
                // Deserialize sub-combinators
                let (i0, sub_combinator0) = self.deserialize_combinator(i + 1);
                let (i1, sub_combinator1) = self.deserialize_combinator(i0);

                (i1, Box::new(ThenCombinator::new(sub_combinator0, sub_combinator1)))
            },

            // get combinator
            8 => {
                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_combinator(i + 1);

                (i0, Box::new(GetCombinator::new(sub_combinator)))
            },

            // anytime combinator
            9 => {
                // Keep track of anytime_index and anytime_acquisition_times
                let anytime_index = self.anytime_acquisition_times.len();
                self.anytime_acquisition_times.push(None);

                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_combinator(i + 1);

                (i0, Box::new(AnytimeCombinator::new(sub_combinator, anytime_index)))
            },

            // Unrecognised combinator
            _ => panic!("Unrecognised combinator provided.")
        }
    }

    // Add numbers safely to avoid integer overflow/underflow
    fn safe_add(x: i64, y: i64) -> i64 {
        if y > 0 && x > 2_i64.pow(62) - y {
            panic!("Integer overflow.");
        } else if y < 0 && x < -2_i64.pow(62) - y {
            panic!("Integer underflow.");
           
        }
        x + y
    }

    // Checks if a U256 can be converted to an i64 without loss of information
    fn assert_U256_can_be_i64(val: U256) {
        if val > U256::from(2_i64.pow(62)) {
            panic!("Given value is too large to be converted to i64.");
        }
    }

    // Withdraws Ether from the given contract participant, returns the amount to send including gas price
    fn get_withdrawal_amount(amount: u64, balance: i64) -> i64 {
        let final_amount = amount as i64 + CALL_GAS;

        // If the withdrawer can't afford the gas for the transaction, do nothing more
        if balance < CALL_GAS {
            return 0;
        }

        // Clamp withdrawal at balance amount
        if balance < final_amount {
            return balance;
        } else {
            return final_amount as i64;
        }
    }
}

// Unit tests
#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    extern crate pwasm_test;

    use super::{ FinancialScContract, FinancialScInterface };
    use super::pwasm_std::{ Vec, vec, types::{ Address, U256 } };
    use self::pwasm_test::{ ext_reset, ext_update };

    // Initialise a FinancialScContract with the given values (and mock blockchain parameters)
    fn setup_contract(sender: Address, holder: Address, timestamp: u64, serialized_combinator_contract: Vec<i64>) -> FinancialScContract {
        let mut contract = FinancialScContract::new();

        ext_reset(|e| e
            .sender(sender)
            .timestamp(timestamp)
        );
        contract.constructor(serialized_combinator_contract, holder);
        contract
    }

    // The counter-party of the contract is set to the deployer
    #[test]
    fn correct_counter_party() {
        // Mock values and instantiate contract
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = setup_contract(
            sender,
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        // Check counter-party
        assert_eq!(
            contract.get_counter_party(),
            sender,
            "Counter party does not match the sender of the constructor call: {}",
            contract.get_counter_party()
        );
    }

    // The holder of the contract is set to the provided address
    #[test]
    fn correct_holder() {
        // Mock values and instantiate contract
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![0]
        );

        // Check holder
        assert_eq!(
            contract.get_holder(),
            holder,
            "Holder does not match the provided holder of the constructor call: {}",
            contract.get_holder()
        );
    }

    // The serialized combinator contract is set to the provided combinator contract
    #[test]
    fn correct_combinator_contract() {
        // Mock values and instantiate contract
        let combinator_contract = vec![2, 2, 1, 0, 2, 2, 0, 0, 1];
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            combinator_contract.clone()
        );

        // Check that the value is correct
        let registered_combinator_contract = contract.get_contract_definition();
        assert_eq!(
            registered_combinator_contract,
            combinator_contract,
            "Combinator contract does not match provided combinator contract: {:?}",
            registered_combinator_contract
        );
    }

    // The contract doesn't use extraneous combinators in the serialized combinators vector
    #[test]
    fn contract_ignores_extra_combinators_in_serialized_vector() {
        let combinator_contract = vec![0, 2, 1, 1];
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            combinator_contract.clone()
        );

        // Check that the value is correct
        assert_eq!(contract.get_value(), 0);
    }

    // Updating before acquiring the contract does nothing
    #[test]
    fn updating_before_acquiring_does_nothing() {
        let counter_party = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            counter_party,
            holder,
            0,
            vec![1]
        );

        ext_reset(|e| e.sender(holder));
        contract.update();
        assert_eq!(contract.get_balance(), 0);

        ext_update(|e| e.sender(counter_party));
        assert_eq!(contract.get_balance(), 0);
    }

    // Updating after acquiring the contract sets the balance correctly
    #[test]
    fn updating_after_acquiring_updates_balances_correctly () {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let counter_party = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = setup_contract(
            counter_party,
            holder,
            0,
            vec![1]
        );

        ext_reset(|e| e.sender(holder));
        contract.acquire();
        contract.update();
        assert_eq!(contract.get_balance(), 1);

        ext_update(|e| e.sender(counter_party));
        assert_eq!(contract.get_balance(), -1);
    }

    // Staking Eth as the holder stakes the correct amount
    #[test]
    fn holder_balance_updates() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![0]
        );

        // Check that the initial stake is 0
        ext_reset(|e| e.sender(holder));
        assert_eq!(contract.get_balance(), 0);

        // Check that the stake increases when added to
        let new_stake = 10;
        ext_reset(|e| e
            .sender(holder)
            .value(U256::from(new_stake))
        );

        contract.stake();
        assert_eq!(new_stake, contract.get_balance());
        contract.stake();
        assert_eq!(contract.get_balance(), new_stake * 2);
    }

    // Staking Eth as the counter-party stakes the correct amount
    #[test]
    fn counter_party_balance_updates() {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = setup_contract(
            sender,
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        // Check that the initial stake is 0
        ext_reset(|e| e.sender(sender));
        assert_eq!(contract.get_balance(), 0);

        // Check that the stake increases when added to
        let new_stake = 10;
        ext_reset(|e| e
            .sender(sender)
            .value(U256::from(new_stake))
        );

        contract.stake();
        assert_eq!(contract.get_balance(), new_stake);
        contract.stake();
        assert_eq!(contract.get_balance(), new_stake * 2);
    }

    // Withdrawal amount is calculated correctly for a normal withdrawal
    #[test]
    fn get_withdrawal_amount_calculates_correct_normal_amount() {
        let balance = 10000;
        let withdrawal = 5000;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance);

        assert_eq!(amount, withdrawal as i64 + super::CALL_GAS);
    }

    // Withdrawal withdraws balance amount at maximum, even if requested amount is higher
    #[test]
    fn get_withdrawal_amount_clamps_withdrawal_to_balance() {
        let balance = 5000;
        let withdrawal = 10000;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance);

        assert_eq!(balance, amount);
    }

    // Withdrawal does not withdraw anything if the balance is below the required call gas price
    #[test]
    fn withdraw_does_not_withdraw_if_balance_below_gas_price() {
        let balance = (super::CALL_GAS - 1) as i64;
        let withdrawal = (super::CALL_GAS - 1) as u64;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance);

        assert_eq!(amount, 0);
    }

    // Attempting to create a contract with the same holder and counter-party should panic
    #[test]
    #[should_panic(expected = "Holder and counter-party must be different addresses.")]
    fn should_panic_if_holder_equals_counter_party() {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        setup_contract(
            sender,
            sender,
            0,
            vec![0]
        );
    }

    // An empty deserialized combinator vector is not allowed
    #[test]
    #[should_panic(expected = "Provided combinator contract not valid.")]
    fn should_panic_if_no_combinators_given() {
        // Mock values and instantiate contract
        setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![]
        );
    }

    // An undefined combinator vector value is not allowed
    #[test]
    #[should_panic(expected = "Unrecognised combinator provided.")]
    fn should_panic_if_combinator_value_unrecognised() {
        // Mock values and instantiate contract
        setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![255]
        );
    }

    // A non-holder account providing an or-choice is not allowed.
    #[test]
    #[should_panic(expected = "Only the contract holder may set or-choices.")]
    fn should_panic_if_non_holder_provides_or_choice() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        ext_reset(|e| e.sender(Address::zero()));
        contract.set_or_choice(0, true);
    }

    // Providing an or choice for a non-existent or combinator is not allowed
    #[test]
    #[should_panic(expected = "Given or-index does not exist.")]
    fn should_panic_if_non_existent_or_choice_provided() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![0]
        );

        ext_reset(|e| e.sender("25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap()));
        contract.set_or_choice(0, true);
    }

    // Non-holders acquiring the contract is not allowed
    #[test]
    #[should_panic(expected = "Only the contract holder may acquire the combinator contract.")]
    fn should_panic_if_non_holder_acquires() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        ext_reset(|e| e.sender(Address::zero()));
        contract.acquire();
    }

    // Acquiring the contract twice is not allowed
    #[test]
    #[should_panic(expected = "The combinator contract cannot be acquired more than once.")]
    fn should_panic_if_contract_acquired_twice() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![0]
        );

        ext_reset(|e| e.sender(holder));
        contract.acquire();
        contract.acquire();
    }

    // Non-holders acquiring anytime sub-contracts is not allowed
    #[test]
    #[should_panic(expected = "Only the contract holder may acquire the combinator contract.")]
    fn should_panic_if_non_holder_acquires_anytime_sub_contract() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![9, 1]
        );

        ext_reset(|e| e.sender(Address::zero()));
        contract.acquire_anytime_sub_contract(0);
    }

    // Non-holders acquiring anytime sub-contracts is not allowed
    #[test]
    #[should_panic(expected = "Given anytime index does not exist.")]
    fn should_panic_when_acquiring_non_existent_anytime_sub_contract() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![9, 1]
        );

        ext_reset(|e| e
            .sender(holder)
            .timestamp(0)
        );
        contract.acquire();
        contract.acquire_anytime_sub_contract(1);
    }

    // Acquiring anytime sub-contracts twice is not allowed
    #[test]
    #[should_panic(expected = "Cannot acquire a sub-combinator contract which has already been acquired.")]
    fn should_panic_when_acquiring_anytime_sub_contract_twice() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![9, 1]
        );

        ext_reset(|e| e
            .sender(holder)
            .timestamp(0)
        );
        contract.acquire();
        contract.acquire_anytime_sub_contract(0);
        contract.acquire_anytime_sub_contract(0);
    }

    // Overflowing the holder's stake is not allowed
    #[test]
    #[should_panic(expected = "Integer overflow.")]
    fn should_panic_if_holder_balance_overflows() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![0]
        );

        // Set the stake to the maximum i64 value
        ext_reset(|e| e
            .sender(holder)
            .value(U256::from(2_i64.pow(62)))
        );
        contract.stake();

        // Overflow the stake
        ext_reset(|e| e
            .sender(holder)
            .value(U256::from(1))
        );
        contract.stake();
    }

    // Overflowing the counter-party's stake is not allowed
    #[test]
    #[should_panic(expected = "Integer overflow.")]
    fn should_panic_if_counter_party_balance_overflows() {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = setup_contract(
            sender,
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        // Set the stake to the maximum i64 value
        ext_reset(|e| e
            .sender(sender)
            .value(U256::from(2_i64.pow(62)))
        );
        contract.stake();

        // Overflow the stake
        ext_reset(|e| e
            .sender(sender)
            .value(U256::from(1))
        );

        // Check that integer overflow panic is caught
        contract.stake();
    }

    // The holder staking more than the maximum value of an i64 is not allowed
    #[test]
    #[should_panic(expected = "Given value is too large to be converted to i64.")]
    fn should_panic_if_holder_stake_too_large() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![0]
        );

        // Set the stake to the maximum u64 value
        ext_reset(|e| e
            .sender(holder)
            .value(U256::from(2_u64.pow(63)))
        );
        contract.stake();
    }

    // The counter-party staking more than the maximum value of an i64 is not allowed
    #[test]
    #[should_panic(expected = "Given value is too large to be converted to i64.")]
    fn should_panic_if_counter_party_stake_too_large() {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = setup_contract(
            sender,
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        // Set the stake to the maximum u64 value
        ext_reset(|e| e
            .sender(sender)
            .value(U256::from(2_u64.pow(63)))
        );
        contract.stake();
    }

    // An uninvolved user attempting to get their stake is not allowed
    #[test]
    #[should_panic(expected = "Only the contract holder or the counter-party may have stake in the contract.")]
    fn should_panic_if_uninvolved_user_checks_stake() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        // Check state as an uninvolved user
        ext_reset(|e| e.sender(Address::zero()));
        contract.get_balance();
    }

    // An uninvolved user attempting to stake Eth is not allowed
    #[test]
    #[should_panic(expected = "Only the contract holder or the counter-party may stake Ether in the contract.")]
    fn should_panic_if_uninvolved_user_stakes() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        // Check state as an uninvolved user
        ext_reset(|e| e
            .sender(Address::zero())
            .value(U256::from(10))
        );
        contract.stake();
    }
}