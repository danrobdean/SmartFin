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
    // Reads a value of the given type from storage, returns the value and the last used address (storage is done sequentially)
    fn read(&mut self, key: &H256) -> (T, H256);

    // Writes a value of the given type to storage, returns the last used address (storage is done sequentially)
    fn write(&mut self, key: &H256, value: T) -> H256;
}

// The implementing struct can store values of the given type (passed by reference).
trait StoresRef<T> {
    // Reads a value of the given type from storage, returns the value and the last used address (storage is done sequentially)
    fn read_ref(&mut self, key: &H256) -> (T, H256);

    // Writes a value of the given type to storage (from reference), returns the last used address (storage is done sequentially)
    fn write_ref(&mut self, key: &H256, value: &T) -> H256;
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

    // Write the length of a vector to storage, uses 1 slot in storage
    fn write_len(&mut self, key: &H256, len: i64) {
        pwasm_ethereum::write(key, &Storage::from_i64(len));
    }

    // Read whether an Option has Some or None
    fn read_some(&mut self, key: &H256) -> bool {
        Storage::to_bool(&pwasm_ethereum::read(key))
    }

    // Write whether an Option has Some or None
    fn write_some(&mut self, key: &H256, some: bool) {
        pwasm_ethereum::write(key, &Storage::from_bool(some));
    }
    
    // Convert a stored value into an address
    fn to_address(value: &[u8; 32]) -> Address {
        H256::from(value).into()
    }

    // Convert an address into a storable value
    fn from_address(value: &Address) -> [u8; 32] {
        H256::from(value.clone()).into()
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

    // Converts a storable value into a bool
    fn to_bool(value: &[u8; 32]) -> bool {
        H256::from(value) != H256::zero()
    }

    // Converts a bool into a storable value
    fn from_bool(value: bool) -> [u8; 32] {
        let mut res: [u8; 32];
        if value {
            res= [0; 32];
            res[0] = 1;
        } else {
            res = H256::zero().into();
        }
        res
    }
}

impl StoresRef<[u8; 32]> for Storage {
    // Read a value from storage, store locally if not already
    fn read_ref(&mut self, key: &H256) -> ([u8; 32], H256) {
        for entry in &self.table {
            if entry.key == *key {
                return (entry.value.clone(), *key);
            }
        }

        let value = pwasm_ethereum::read(key);
        self.table.push(Entry {
            key: key.clone(),
            value: value.clone()
        });
        (value, *key)
    }

    // Write a value to storage and store locally
    fn write_ref(&mut self, key: &H256, value: &[u8; 32]) -> H256 {
        pwasm_ethereum::write(key, &value);

        for entry in &mut self.table {
            if entry.key == *key {
                entry.value = value.clone();
                return *key;
            }
        }

        self.table.push(Entry {
            key: key.clone(),
            value: value.clone()
        });
        *key
    }
}

impl StoresRef<Address> for Storage {
    fn read_ref(&mut self, key: &H256) -> (Address, H256) {
        let (value, last_used): ([u8; 32], H256) = self.read_ref(key);
        (Storage::to_address(&value), last_used)
    }

    fn write_ref(&mut self, key: &H256, value: &Address) -> H256 {
        self.write_ref(key, &Storage::from_address(value));
        *key
    }
}

impl Stores<i64> for Storage {
    fn read(&mut self, key: &H256) -> (i64, H256) {
        let (value, last_used): ([u8; 32], H256) = self.read_ref(key);
        (Storage::to_i64(&value), last_used)
    }

    fn write(&mut self, key: &H256, value: i64) -> H256 {
        self.write_ref(key, &Storage::from_i64(value));
        *key
    }
}

impl Stores<u32> for Storage {
    fn read(&mut self, key: &H256) -> (u32, H256) {
        let (value, last_used): ([u8; 32], H256) = self.read_ref(key);
        // Converts from an i64, works as long as the stored value is actually a u32 (should always be the case)
        (Storage::to_i64(&value) as u32, last_used)
    }

    fn write(&mut self, key: &H256, value: u32) -> H256 {
        // Converts to an i64, works as long as the value being stores is 0 < val < 2^63-1, we have 0 < val_u32 < 2^32-1
        self.write_ref(key, &Storage::from_i64(value as i64));
        *key
    }
}

impl Stores<bool> for Storage {
    fn read(&mut self, key: &H256) -> (bool, H256) {
        let (value, last_used): ([u8; 32], H256) = self.read_ref(key);
        (Storage::to_bool(&value), last_used)
    }

    fn write(&mut self, key: &H256, value: bool) -> H256 {
        self.write_ref(key, &Storage::from_bool(value))
    }
}

// Vectors are stored sequentially
impl<T> StoresRef<Vec<T>> for Storage where Storage: Stores<T>, Vec<T>: core::clone::Clone {
    // Reads vector sequentially from storage
    fn read_ref(&mut self, key: &H256) -> (Vec<T>, H256) {
        let length: i64 = self.read_len(key);
        let mut current = add_to_key(*key, 1);
        let mut res: Vec<T> = Vec::new();        

        let mut last_used: H256 = *key;
        for _ in 0..length {
            let (value, end): (T, H256) = self.read(&current);
            last_used = end;
            res.push(value);
            current = add_to_key(end, 1);
        }

        (res, last_used)
    }

    fn write_ref(&mut self, key: &H256, value: &Vec<T>) -> H256 {
        let length = value.len();
        self.write_len(key, length as i64);
        let mut last_used = *key;
        let mut clone = value.clone();

        // Consume list clone, writing each element to storage
        for _ in 0..length {
            last_used = self.write(&add_to_key(last_used, 1 as u64), clone.remove(0));
        }
        last_used
    }
}

impl<T> Stores<Option<T>> for Storage where Storage: Stores<T> {
    fn read(&mut self, key: &H256) -> (Option<T>, H256) {
        let some: bool = self.read_some(key);
        if some {
            let (value, last_used) = self.read(&add_to_key(*key, 1));
            (Some(value), last_used)
        } else {
            (None, *key)
        }
    }

    fn write(&mut self, key: &H256, value: Option<T>) -> H256 {
        match value {
            Some(v) => {
                self.write_some(key, true);
                self.write(&add_to_key(*key, 1), v)
            },
            None => {
                self.write_some(key, false);
                return *key
            }
        }
    }
}

impl<T, U> Stores<(T, U)> for Storage where Storage: Stores<T> + Stores<U> {
    fn read(&mut self, key: &H256) -> ((T, U), H256) {
        let (first, key0): (T, H256) = self.read(key);
        let (second, key1): (U, H256) = self.read(&add_to_key(key0, 1));
        ((first, second), key1)
    }

    fn write(&mut self, key: &H256, value: (T, U)) -> H256 {
        let key0 = self.write(&key, value.0);
        self.write(&add_to_key(key0, 1), value.1)
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

        // Only allow modification of the first 31 bytes of the address, the last byte separates memory namespaces and cannot be crossed
        if i >= 31 {
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

// The serialized combinator contract (obtained remotely) storage key (first slot is length, the following are elements)
fn serialized_remote_combinator_contract_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1])
}

// The or choices storage key (first slot is length, the following are elements)
fn or_choices_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2])
}

// The holder's proposed observable values storage key (first slot is length, the following are elements)
fn holder_proposed_obs_values_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3])
}

// The counter-party's proposed observable values storage key (first slot is length, the following are elements)
fn counter_party_proposed_obs_values_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4])
}

// The concrete observable values storage key (first slot is length, the following are elements)
fn concrete_obs_values_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5])
}

// The anytime acquisition times key storage key (first slot is length, the following are elements)
fn anytime_acquisition_times_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6])
}

// The serialized combinator contract (obtained locally) storage key (first slot is length, the following are elements)
fn serialized_local_combinator_contract_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7])
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
        self.storage.write_ref(&holder_address_key(), &holder);
        self.storage.write_ref(&counter_party_address_key(), &pwasm_ethereum::sender());
        self.storage.write(&holder_balance_key(), 0_i64);
        self.storage.write(&counter_party_balance_key(), 0_i64);
        self.storage.write_ref(&serialized_remote_combinator_contract_key() , &contract_definition);

        self.set_remote_combinator();
    }

    // Gets the address of the holder
    fn get_holder(&mut self) -> Address {
        self.storage.read_ref(&holder_address_key()).0
    }

    // Gets the address of the counter-party
    fn get_counter_party(&mut self) -> Address {
        self.storage.read_ref(&counter_party_address_key()).0
    }

    // Gets the combinator contract definition (serialized)
    fn get_contract_definition(&mut self) -> Vec<i64> {
        self.storage.read_ref(&serialized_remote_combinator_contract_key()).0
    }

    // Gets the current value of the contract
    fn get_value(&mut self) -> i64 {
        let or_choices: Vec<Option<bool>> = self.storage.read_ref(&or_choices_key()).0;
        let concrete_obs_values: Vec<Option<i64>> = self.storage.read_ref(&concrete_obs_values_key()).0;
        let anytime_acquisition_times: Vec<(bool, Option<u32>)> = self.storage.read_ref(&anytime_acquisition_times_key()).0;

        self.get_combinator().get_value(pwasm_ethereum::timestamp() as u32, &or_choices, &concrete_obs_values, &anytime_acquisition_times)
    }

    // Gets the total balance of the caller
    fn get_balance(&mut self) -> i64 {
        let sender = pwasm_ethereum::sender();
        let holder: Address = self.storage.read_ref(&holder_address_key()).0;
        let counter_party: Address = self.storage.read_ref(&counter_party_address_key()).0;

        if sender == holder {
            self.storage.read(&holder_balance_key()).0
        } else if sender == counter_party {
            self.storage.read(&counter_party_balance_key()).0
        } else {
            panic!("Only the contract holder or the counter-party may have stake in the contract.");
        }
    }

    // Gets whether or not the contract has concluded.
    fn get_concluded(&mut self) -> bool {
        let combinator = self.get_combinator();
        let combinator_details = combinator.get_combinator_details();
        combinator_details.fully_updated
            || combinator_details.acquisition_time == None && combinator.past_horizon(pwasm_ethereum::timestamp() as u32)
    }

    // Sets the given or combinator's preference between its sub-combinators
    fn set_or_choice(&mut self, or_index: u64, prefer_first: bool) {
        let holder: Address = self.storage.read_ref(&holder_address_key()).0;
        if pwasm_ethereum::sender() != holder {
            panic!("Only the contract holder may set or-choices.");
        }

        let index = or_index as usize;
        let mut or_choices: Vec<Option<bool>> = self.storage.read_ref(&or_choices_key()).0;
        if index >= or_choices.len() {
            panic!("Given or-index does not exist.");
        }

        or_choices[index as usize] = Some(prefer_first);
        self.storage.write_ref(&or_choices_key(), &or_choices);
    }

    // Proposes the given observable's value
    fn propose_obs_value(&mut self, obs_index: u64, value: i64) {
        // Check that index is within bounds
        let index: usize = obs_index as usize;
        let mut holder_proposed_obs_values: Vec<Option<i64>> = self.storage.read_ref(&holder_proposed_obs_values_key()).0;
        let mut counter_party_proposed_obs_values: Vec<Option<i64>> = self.storage.read_ref(&counter_party_proposed_obs_values_key()).0;
        if index >= holder_proposed_obs_values.len() {
            panic!("Given observable-index does not exist.");
        }

        // Set the proposed value for the sender
        let sender = pwasm_ethereum::sender();
        let holder: Address = self.storage.read_ref(&holder_address_key()).0;
        let counter_party: Address = self.storage.read_ref(&counter_party_address_key()).0;

        if sender == holder {
            holder_proposed_obs_values[index] = Some(value);
            self.storage.write_ref(&holder_proposed_obs_values_key(), &holder_proposed_obs_values);
        } else if sender == counter_party {
            counter_party_proposed_obs_values[index] = Some(value);
            self.storage.write_ref(&counter_party_proposed_obs_values_key(), &counter_party_proposed_obs_values);
        } else {
            panic!("Only the holder or counter-party set the value of observables.");
        }

        // Check if proposed values match
        if holder_proposed_obs_values[index] == counter_party_proposed_obs_values[index] {
            let mut concrete_obs_values: Vec<Option<i64>> = self.storage.read_ref(&concrete_obs_values_key()).0;
            concrete_obs_values[index] = Some(value);
            self.storage.write_ref(&concrete_obs_values_key(), &concrete_obs_values);
        }
    }

    // Acquires the combinator contract at the current block-time (when called by the holder)
    fn acquire(&mut self) {
        let mut combinator = self.get_combinator();
        let holder: Address = self.storage.read_ref(&holder_address_key()).0;

        if pwasm_ethereum::sender() != holder {
            panic!("Only the contract holder may acquire the combinator contract.");
        } else if combinator.get_combinator_details().acquisition_time != None {
            panic!("The combinator contract cannot be acquired more than once.");
        }

        let or_choices: Vec<Option<bool>> = self.storage.read_ref(&or_choices_key()).0;
        let mut anytime_acquisition_times: Vec<(bool, Option<u32>)> = self.storage.read_ref(&anytime_acquisition_times_key()).0;

        combinator.acquire(pwasm_ethereum::timestamp() as u32, &or_choices, &mut anytime_acquisition_times);

        self.set_combinator(combinator);
        self.storage.write_ref(&anytime_acquisition_times_key(), &anytime_acquisition_times);
    }

    // Updates the balances of the holder and counter-party
    fn update(&mut self) {
        // If concluded, can't update.
        if self.get_concluded() {
            panic!("Contract has concluded, nothing more to update.");
        }

        // Update combinators
        let mut combinator = self.get_combinator();
        let or_choices: Vec<Option<bool>> = self.storage.read_ref(&or_choices_key()).0;
        let concrete_obs_values: Vec<Option<i64>> = self.storage.read_ref(&concrete_obs_values_key()).0;
        let mut anytime_acquisition_times: Vec<(bool, Option<u32>)> = self.storage.read_ref(&anytime_acquisition_times_key()).0;

        let difference = combinator.update(pwasm_ethereum::timestamp() as u32, &or_choices, &concrete_obs_values, &mut anytime_acquisition_times);

        self.set_combinator(combinator);
        self.storage.write_ref(&anytime_acquisition_times_key(), &anytime_acquisition_times);

        // Adjust balances
        let counter_party_balance = self.storage.read(&counter_party_balance_key()).0;
        self.storage.write(&counter_party_balance_key(), FinancialScContract::safe_add(counter_party_balance, -difference));

        let holder_balance = self.storage.read(&holder_balance_key()).0;
        self.storage.write(&holder_balance_key(), FinancialScContract::safe_add(holder_balance, difference));
    }

    // Acquires an anytime combinator's sub-contract
    fn acquire_anytime_sub_contract(&mut self, anytime_index: u64) {
        let index = anytime_index as usize;
        let mut anytime_acquisition_times: Vec<(bool, Option<u32>)> = self.storage.read_ref(&anytime_acquisition_times_key()).0;
        if index >= anytime_acquisition_times.len() {
            panic!("Given anytime index does not exist.");
        }

        if !anytime_acquisition_times[index].0 {
            panic!("Given anytime combinator has not been acquired.");
        }

        let holder: Address = self.storage.read_ref(&holder_address_key()).0;
        if pwasm_ethereum::sender() != holder {
            panic!("Only the contract holder may acquire the combinator contract.");
        }

        let prev_acquisition_time = anytime_acquisition_times[anytime_index as usize].1;
        let new_acquisition_time = pwasm_ethereum::timestamp() as u32;

        if prev_acquisition_time != None && prev_acquisition_time.unwrap() <= new_acquisition_time {
            panic!("Cannot acquire a sub-combinator contract which has already been acquired.");
        }

        anytime_acquisition_times[anytime_index as usize] = (true, Some(new_acquisition_time));
        self.storage.write_ref(&anytime_acquisition_times_key(), &anytime_acquisition_times);
    }

    // Stakes Eth with the contract, returns the caller's total balance
    fn stake(&mut self) -> i64 {
        let sender = pwasm_ethereum::sender();
        let stake = pwasm_ethereum::value();
        FinancialScContract::assert_U256_can_be_i64(stake);
        let holder: Address = self.storage.read_ref(&holder_address_key()).0;
        let counter_party: Address = self.storage.read_ref(&counter_party_address_key()).0;
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
        let mut balance = self.storage.read(&key).0;
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
        let holder: Address = self.storage.read_ref(&holder_address_key()).0;
        let counter_party: Address = self.storage.read_ref(&counter_party_address_key()).0;
        
        // Get the amount to send (clamp at balance amount)
        if sender == holder {
            key = holder_balance_key();
        } else if sender == counter_party {
            key = counter_party_balance_key();
        } else {
            panic!("Only the contract holder or the counter-party may withdraw Ether from the contract.");
        }

        original_balance = self.storage.read(&key).0;
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
            storage: Storage::new()
        }
    }

    // Constructs the combinators from a serialized combinator contract
    fn set_remote_combinator(&mut self) {
        self.storage.write_ref(&or_choices_key(), &Vec::<Option<bool>>::new());
        self.storage.write_ref(&holder_proposed_obs_values_key(), &Vec::<Option<i64>>::new());
        self.storage.write_ref(&counter_party_proposed_obs_values_key(), &Vec::<Option<i64>>::new());
        self.storage.write_ref(&concrete_obs_values_key(), &Vec::<Option<i64>>::new());
        self.storage.write_ref(&anytime_acquisition_times_key(), &Vec::<Option<u32>>::new());

        let (_, combinator) = self.deserialize_remote_combinator(0);

        self.set_combinator(combinator);
    }

    // Deserializes a combinator from the given combinator byte vector (obtained remotely) and index, returns the following index and the boxed combinator
    fn deserialize_remote_combinator(&mut self, i: usize)-> (usize, Box<ContractCombinator>) {
        let serialized_combinators: Vec<i64> = self.storage.read_ref(&serialized_remote_combinator_contract_key()).0;
        if i >= serialized_combinators.len() {
            panic!("Provided combinator contract not valid.");
        }

        match Combinator::from(serialized_combinators[i]) {
            // zero combinator
            Combinator::ZERO => (i + 1, Box::new(ZeroCombinator::new())),

            // one combinator
            Combinator::ONE => (i + 1, Box::new(OneCombinator::new())),

            // and combinator
            Combinator::AND => {
                // Deserialize sub-combinators
                let (i0, sub_combinator0) = self.deserialize_remote_combinator(i + 1);
                let (i1, sub_combinator1) = self.deserialize_remote_combinator(i0);

                (i1, Box::new(AndCombinator::new(sub_combinator0, sub_combinator1)))
            },

            // or combinator
            Combinator::OR => {
                // Keep track of or_index and or_choices
                let mut or_choices: Vec<Option<bool>> = self.storage.read_ref(&or_choices_key()).0;
                let or_index: usize = or_choices.len();
                or_choices.push(None);
                self.storage.write_ref(&or_choices_key(), &or_choices);

                // Deserialize sub-combinators
                let (i0, sub_combinator0) = self.deserialize_remote_combinator(i + 1);
                let (i1, sub_combinator1) = self.deserialize_remote_combinator(i0);

                (i1, Box::new(OrCombinator::new(sub_combinator0, sub_combinator1, or_index)))
            },

            // truncate combinator
            Combinator::TRUNCATE => {
                // Deserialize timestamp from 4 bytes to 32-bit int
                let mut timestamp: u32 = serialized_combinators[i + 1] as u32;

                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_remote_combinator(i + 2);

                (i0, Box::new(TruncateCombinator::new(sub_combinator, timestamp)))
            },

            // scale combinator
            Combinator::SCALE => {
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
                    let mut holder_proposed_obs_values: Vec<Option<i64>> = self.storage.read_ref(&holder_proposed_obs_values_key()).0;
                    let mut counter_party_proposed_obs_values: Vec<Option<i64>> = self.storage.read_ref(&counter_party_proposed_obs_values_key()).0;
                    let mut concrete_obs_values: Vec<Option<i64>> = self.storage.read_ref(&concrete_obs_values_key()).0;

                    obs_index = Some(concrete_obs_values.len());
                    holder_proposed_obs_values.push(None);
                    counter_party_proposed_obs_values.push(None);
                    concrete_obs_values.push(None);

                    self.storage.write_ref(&holder_proposed_obs_values_key(), &holder_proposed_obs_values);
                    self.storage.write_ref(&counter_party_proposed_obs_values_key(), &counter_party_proposed_obs_values);
                    self.storage.write_ref(&concrete_obs_values_key(), &concrete_obs_values);
                    scale_value = None;
                }

                // Deserialize sub-contract
                let (i1, sub_combinator) = self.deserialize_remote_combinator(i0);

                (i1, Box::new(ScaleCombinator::new(sub_combinator, obs_index, scale_value)))
            },

            // give combinator
            Combinator::GIVE => {
                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_remote_combinator(i + 1);

                (i0, Box::new(GiveCombinator::new(sub_combinator)))
            },

            // then combinator
            Combinator::THEN => {
                // Deserialize sub-combinators
                let (i0, sub_combinator0) = self.deserialize_remote_combinator(i + 1);
                let (i1, sub_combinator1) = self.deserialize_remote_combinator(i0);

                (i1, Box::new(ThenCombinator::new(sub_combinator0, sub_combinator1)))
            },

            // get combinator
            Combinator::GET => {
                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_remote_combinator(i + 1);

                (i0, Box::new(GetCombinator::new(sub_combinator)))
            },

            // anytime combinator
            Combinator::ANYTIME => {
                // Keep track of anytime_index and anytime_acquisition_times
                let mut anytime_acquisition_times: Vec<(bool, Option<u32>)> = self.storage.read_ref(&anytime_acquisition_times_key()).0;
                let anytime_index = anytime_acquisition_times.len();
                anytime_acquisition_times.push((false, None));
                self.storage.write_ref(&anytime_acquisition_times_key(), &anytime_acquisition_times);

                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_remote_combinator(i + 1);

                (i0, Box::new(AnytimeCombinator::new(sub_combinator, anytime_index)))
            }
        }
    }

    // Gets and deserializes the ContractCombinator from storage
    fn get_combinator(&mut self) -> Box<ContractCombinator> {
        let serialized = self.storage.read_ref(&serialized_local_combinator_contract_key()).0;
        deserialize_combinator(0, &serialized).1
    }

    // Serializes and stores the ContractCombinator
    fn set_combinator(&mut self, combinator: Box<ContractCombinator>) {
        let serialized = combinator.serialize();
        self.storage.write_ref(&serialized_local_combinator_contract_key(), &serialized);
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

    use super::{ FinancialScContract, FinancialScInterface, Storage, Stores, StoresRef, add_to_key };
    use super::pwasm_std::{ Vec, vec, types::{ Address, U256, H256 } };
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

    // Storage of an i64 works correctly
    #[test]
    fn stores_and_retrieves_i64_correctly() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        let value: i64 = 14436934069;
        contract.storage.write(&H256::zero(), value);
        assert_eq!(contract.storage.read(&H256::zero()), (value, H256::zero()));
    }

    // Storage of a u32 works correctly
    #[test]
    fn stores_and_retrieves_u32_correctly() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        let value: u32 = 1443693;
        contract.storage.write(&H256::zero(), value);
        assert_eq!(contract.storage.read(&H256::zero()), (value, H256::zero()));
    }

    // Storage of a bool works correctly
    #[test]
    fn stores_and_retrieves_bool_correctly() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        let value0: bool = true;
        contract.storage.write(&H256::zero(), value0);
        assert_eq!(contract.storage.read(&H256::zero()), (value0, H256::zero()));

        let value1: bool = false;
        contract.storage.write(&H256::zero(), value1);
        assert_eq!(contract.storage.read(&H256::zero()), (value1, H256::zero()));
    }

    // Storage of an Address works correctly
    #[test]
    fn stores_and_retrieves_address_correctly() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        let value: Address = Address::from([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]);
        contract.storage.write_ref(&H256::zero(), &value);
        assert_eq!(contract.storage.read_ref(&H256::zero()), (value, H256::zero()));
    }

    // Storage of an Option works correctly
    #[test]
    fn stores_and_retrieves_option_correctly() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        let value0: Option<i64> = Some(123);
        contract.storage.write(&H256::zero(), value0);
        assert_eq!(contract.storage.read(&H256::zero()), (value0, add_to_key(H256::zero(), 1)));

        let value1: Option<i64> = None;
        contract.storage.write(&H256::zero(), value1);
        assert_eq!(contract.storage.read(&H256::zero()), (value1, H256::zero()));
    }

    // Storage of a tuple works correctly
    #[test]
    fn stores_and_retrieves_tuple_correctly() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        let value0: Option<i64> = Some(123);
        let value1: i64 = 10;
        contract.storage.write(&H256::zero(), (value0, value1));
        assert_eq!(contract.storage.read(&H256::zero()), ((value0, value1), add_to_key(H256::zero(), 2)));
    }

    // Storage of a multi-slot typed Vec works correctly
    #[test]
    fn stores_and_retrieves_single_slot_type_vec_correctly() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        let value: Vec<i64> = vec![15425, 15436136, 1346134, 123093, 132523];
        contract.storage.write_ref(&H256::zero(), &value);
        assert_eq!(contract.storage.read_ref(&H256::zero()), (value, add_to_key(H256::zero(), 5)));
    }

    // Storage of a multi-slot typed Vec works correctly
    #[test]
    fn stores_and_retrieves_multi_slot_type_vec_correctly() {
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        let value: Vec<Option<i64>> = vec![Some(15425), None, Some(1346134), Some(123093), None];
        contract.storage.write_ref(&H256::zero(), &value);
        assert_eq!(contract.storage.read_ref(&H256::zero()), (value, add_to_key(H256::zero(), 8)));
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

        ext_update(|e| e.sender(holder));
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

        ext_update(|e| e.sender(holder));
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
        ext_update(|e| e.sender(holder));
        assert_eq!(contract.get_balance(), 0);

        // Check that the stake increases when added to
        let new_stake = 10;
        ext_update(|e| e
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
        ext_update(|e| e.sender(sender));
        assert_eq!(contract.get_balance(), 0);

        // Check that the stake increases when added to
        let new_stake = 10;
        ext_update(|e| e
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
    #[should_panic(expected = "Unrecognised combinator.")]
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

        ext_update(|e| e.sender(Address::zero()));
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

        ext_update(|e| e.sender("25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap()));
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

        ext_update(|e| e.sender(Address::zero()));
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

        ext_update(|e| e.sender(holder));
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
        ext_update(|e| e.sender(holder));
        contract.acquire();

        ext_update(|e| e.sender(Address::zero()));
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

        ext_update(|e| e
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

        ext_update(|e| e
            .sender(holder)
            .timestamp(0)
        );
        contract.acquire();
        contract.acquire_anytime_sub_contract(0);
        contract.acquire_anytime_sub_contract(0);
    }

    // Acquiring anytime sub-contracts before the parent contract is not allowed
    #[test]
    #[should_panic(expected = "Given anytime combinator has not been acquired.")]
    fn should_panic_if_acquiring_anytime_sub_contract_before_parent_contract() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![9, 1]
        );
        ext_update(|e| e.sender(holder));
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
        ext_update(|e| e
            .sender(holder)
            .value(U256::from(2_i64.pow(62)))
        );
        contract.stake();

        // Overflow the stake
        ext_update(|e| e
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
        ext_update(|e| e
            .sender(sender)
            .value(U256::from(2_i64.pow(62)))
        );
        contract.stake();

        // Overflow the stake
        ext_update(|e| e
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
        ext_update(|e| e
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
        ext_update(|e| e
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
        ext_update(|e| e.sender(Address::zero()));
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
        ext_update(|e| e
            .sender(Address::zero())
            .value(U256::from(10))
        );
        contract.stake();
    }
}