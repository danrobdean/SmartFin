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
pub mod storage;

use pwasm_abi::eth::EndpointInterface;
use pwasm_std::{ Box, Vec, types::{ Address, U256, H256 } };
use pwasm_abi_derive::eth_abi;
use combinators::*;
use storage::*;

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

// The storage key for whether or not to use gas
fn use_gas_key() -> H256 {
    H256::from([4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
}

// The storage key for the last-updated time
fn last_updated_key() -> H256 {
    H256::from([5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
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

// The observable values storage key (first slot is length, the following are elements)
fn obs_entries_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3])
}

// The anytime acquisition times key storage key (first slot is length, the following are elements)
fn anytime_acquisition_times_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4])
}

// The serialized combinator contract (obtained locally) storage key (first slot is length, the following are elements)
fn serialized_local_combinator_contract_key() -> H256 {
    // Store in own memory namespace as Vec storage size is not constant
    H256::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5])
}

// The financial smart contract interface
#[eth_abi(FinancialScEndpoint)]
pub trait FinancialScInterface {
    // The contract constructor, takes the combinator contract definition (serialized) and the holder address
    fn constructor(&mut self, contract_definition: Vec<i64>, holder: Address, use_gas: bool);

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

    // Gets the current balance of the given party (true is holder, false counter-party)
    #[constant]
    fn get_balance(&mut self, holderBalance: bool) -> i64;

    // Gets whether or not the contract has concluded all operation (i.e. updating will never change the balance).
    #[constant]
    fn get_concluded(&mut self) -> bool;

    // Gets whether or not the contract allocates gas fees upon withdrawal.
    #[constant]
    fn get_use_gas(&mut self) -> bool;

    // Gets the last-updated time.
    #[constant]
    fn get_last_updated(&mut self) -> i64;

    // Gets the contract acquisition times (top level acquisition time and anytime acquisition times)
    #[constant]
    fn get_acquisition_times(&mut self) -> Vec<i64>;

    // Gets the or choices
    #[constant]
    fn get_or_choices(&mut self) -> Vec<u8>;

    // Gets the concrete observable values
    #[constant]
    fn get_obs_entries(&mut self) -> Vec<i64>;

    // Sets the preference of the given or combinator's sub-combinators
    fn set_or_choice(&mut self, or_index: u64, choice: bool);

    // Sets a value for the given observable
    fn set_obs_value(&mut self, obs_index: u64, value: i64);

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
    fn withdraw(&mut self, amount: u64) ;
}

// The financial smart contract
pub struct FinancialScContract {
    // The contract storage table
    storage: Storage,
}

// The financial smart contract interface implementation
impl FinancialScInterface for FinancialScContract {
    // The financial smart contract constructor
    fn constructor(&mut self, contract_definition: Vec<i64>, holder: Address, use_gas: bool) {
        if holder == pwasm_ethereum::sender() {
            panic!("Holder and counter-party must be different addresses.");
        }
        if contract_definition.len() == 0 {
            panic!("Provided combinator contract not valid.");
        }

        // Initialise storage
        self.storage.write(&holder_address_key(), holder);
        self.storage.write(&counter_party_address_key(), pwasm_ethereum::sender());
        self.storage.write(&holder_balance_key(), 0_i64);
        self.storage.write(&counter_party_balance_key(), 0_i64);
        self.storage.write(&use_gas_key(), use_gas);
        self.storage.write(&last_updated_key(), pwasm_ethereum::timestamp() as i64);
        self.storage.write_ref(&serialized_remote_combinator_contract_key() , &contract_definition);

        self.set_remote_combinator();
    }

    // Gets the address of the holder
    fn get_holder(&mut self) -> Address {
        self.storage.read(&holder_address_key()).0
    }

    // Gets the address of the counter-party
    fn get_counter_party(&mut self) -> Address {
        self.storage.read(&counter_party_address_key()).0
    }

    // Gets the combinator contract definition (serialized)
    fn get_contract_definition(&mut self) -> Vec<i64> {
        self.storage.read_ref(&serialized_remote_combinator_contract_key()).0
    }

    // Gets the current value of the contract
    fn get_value(&mut self) -> i64 {
        let or_choices: Vec<Option<bool>> = self.storage.read_ref(&or_choices_key()).0;
        let obs_values: Vec<Option<i64>> = self.get_obs_values();
        let anytime_acquisition_times: Vec<(bool, Option<u32>)> = self.storage.read_ref(&anytime_acquisition_times_key()).0;

        self.get_combinator().get_value(pwasm_ethereum::timestamp() as u32, &or_choices, &obs_values, &anytime_acquisition_times)
    }

    // Gets the total balance of the caller
    fn get_balance(&mut self, holderBalance: bool) -> i64 {
        if holderBalance {
            self.storage.read(&holder_balance_key()).0
        } else {
            self.storage.read(&counter_party_balance_key()).0
        }
    }

    // Gets whether or not the contract has concluded.
    fn get_concluded(&mut self) -> bool {
        let combinator = self.get_combinator();
        let combinator_details = combinator.get_combinator_details();
        combinator_details.fully_updated
            || combinator_details.acquisition_time == None && combinator.past_horizon(pwasm_ethereum::timestamp() as u32)
    }

    // Gets whether or not the contract allocates gas fees upon withdrawal.
    fn get_use_gas(&mut self) -> bool {
        self.storage.read(&use_gas_key()).0
    }

    // Gets the last-updated time.
    fn get_last_updated(&mut self) -> i64 {
        self.storage.read(&last_updated_key()).0
    }

    // Gets the contract acquisition times (top level acquisition time and anytime acquisition times)
    fn get_acquisition_times(&mut self) -> Vec<i64> {
        let acquisition_time: Option<u32> = self.get_combinator().get_combinator_details().acquisition_time;
        let anytime_acquisition_times_full: Vec<(bool, Option<u32>)> = self.storage.read_ref(&anytime_acquisition_times_key()).0;

        let mut serialized_acquisition_times: Vec<i64> = Vec::new();
        serialized_acquisition_times.push(if acquisition_time == None { -1 } else { acquisition_time.unwrap() as i64 });

        let anytime_acquisition_times: Vec<i64> =
            anytime_acquisition_times_full.into_iter().map(|e| if e.1 == None { -1 } else { e.1.unwrap() as i64 }).collect();

        serialized_acquisition_times.extend_from_slice(&anytime_acquisition_times[..]);
        serialized_acquisition_times
    }

    // Gets the or choices
    fn get_or_choices(&mut self) -> Vec<u8> {
        let or_choices: Vec<Option<bool>> = self.storage.read_ref(&or_choices_key()).0;

        // Convert None into 2, Some(true) into 1, Some(false) into 0
        or_choices.into_iter().map(|e|
            if e == None {
                2
            } else if e.unwrap() {
                1
            } else {
                0
            }
        ).collect()
    }

    // Gets the concrete observable values
    fn get_obs_entries(&mut self) -> Vec<i64> {
        let obs_entries: Vec<(Address, Option<i64>)> = self.storage.read_ref(&obs_entries_key()).0;
        let mut serialized_obs_entries: Vec<i64> = Vec::new();

        // Serialize obs values
        for value in obs_entries {
            // Convert address to four i64s and then store

            // Convert to i64s
            let address: [i64; 4] = address_to_i64(value.0);
            for i in 0..4 {
                serialized_obs_entries.push(address[i]);
            }

            // Push value
            if value.1 == None {
                serialized_obs_entries.push(-1);
            } else {
                serialized_obs_entries.push(0);
                serialized_obs_entries.push(value.1.unwrap());
            }
        }

        serialized_obs_entries
    }

    // Sets the given or combinator's preference between its sub-combinators
    fn set_or_choice(&mut self, or_index: u64, prefer_first: bool) {
        let holder: Address = self.storage.read(&holder_address_key()).0;
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

    // Sets the given observable's value
    fn set_obs_value(&mut self, obs_index: u64, value: i64) {
        let mut obs_entries: Vec<(Address, Option<i64>)> = self.storage.read_ref(&obs_entries_key()).0;

        // Check index in bounds
        let index: usize = obs_index as usize;
        if index >= obs_entries.len() {
            panic!("Given observable-index does not exist.");
        }

        // Check sender
        let arbiter: Address = obs_entries[index].0;
        let sender: Address = pwasm_ethereum::sender();
        if sender != arbiter {
            panic!("Sender cannot set value for given observable-index.");
        }

        // Set the value
        obs_entries[index] = (arbiter, Some(value));
        self.storage.write_ref(&obs_entries_key(), &obs_entries);
    }

    // Acquires the combinator contract at the current block-time (when called by the holder)
    fn acquire(&mut self) {
        let mut combinator = self.get_combinator();
        let holder: Address = self.storage.read(&holder_address_key()).0;

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

        self.update();
    }

    // Updates the balances of the holder and counter-party
    fn update(&mut self) {
        // If concluded, can't update.
        if self.get_concluded() {
            panic!("Contract has concluded, nothing more to update.");
        }

        // Set the last-updated time
        self.storage.write(&last_updated_key(), pwasm_ethereum::timestamp() as i64);

        // Update combinators
        let mut combinator = self.get_combinator();
        let or_choices: Vec<Option<bool>> = self.storage.read_ref(&or_choices_key()).0;
        let obs_values: Vec<Option<i64>> = self.get_obs_values();
        let mut anytime_acquisition_times: Vec<(bool, Option<u32>)> = self.storage.read_ref(&anytime_acquisition_times_key()).0;

        let difference = combinator.update(pwasm_ethereum::timestamp() as u32, &or_choices, &obs_values, &mut anytime_acquisition_times);

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

        let holder: Address = self.storage.read(&holder_address_key()).0;
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

        self.update();
    }

    // Stakes Eth with the contract, returns the caller's total balance
    fn stake(&mut self) -> i64 {
        let sender = pwasm_ethereum::sender();
        let stake = pwasm_ethereum::value();
        FinancialScContract::assert_U256_can_be_i64(stake);
        let holder: Address = self.storage.read(&holder_address_key()).0;
        let counter_party: Address = self.storage.read(&counter_party_address_key()).0;
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
    fn withdraw(&mut self, amount: u64) {
        let sender = pwasm_ethereum::sender();
        let final_amount;
        let original_balance;
        let key;
        let holder_balance = self.storage.read(&holder_balance_key()).0;
        let counter_party_balance = self.storage.read(&counter_party_balance_key()).0;
        let holder: Address = self.storage.read(&holder_address_key()).0;
        let counter_party: Address = self.storage.read(&counter_party_address_key()).0;
        let use_gas = self.storage.read(&use_gas_key()).0;
        
        // Get the amount to send (clamp at balance amount)
        if sender == holder {
            key = holder_balance_key();
            original_balance = holder_balance;
        } else if sender == counter_party {
            key = counter_party_balance_key();
            original_balance = counter_party_balance;
        } else {
            panic!("Only the contract holder or the counter-party may withdraw Ether from the contract.");
        }

        let funds = holder_balance + counter_party_balance;
        final_amount = FinancialScContract::get_withdrawal_amount(amount, original_balance, funds, use_gas);

        if use_gas && final_amount < CALL_GAS {
            panic!("Not enough funds to pay the gas price while withdrawing.");
        }

        self.storage.write(&key, original_balance - final_amount);
        let mut result = Vec::<u8>::new();

        let gas_cost = if use_gas { CALL_GAS } else { 0 };
        let withdraw_amount = final_amount - gas_cost;

        match pwasm_ethereum::call(gas_cost as u64, &sender, U256::from(withdraw_amount), &[], &mut result) {
            Ok(_v) => return,
            Err(_e) => {
                // Payment failed, roll-back balance
                self.storage.write(&key, original_balance);
                panic!("Payment failed");
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
        self.storage.write_ref(&obs_entries_key(), &Vec::<(Address, Option<i64>)>::new());
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
                // Check if observable is provided, if so then deserialize it, otherwise record in obs_entries
                let provided: i64 = serialized_combinators[i + 1];
                let mut obs_index: Option<usize>;
                let mut scale_value: Option<i64>;
                let mut i0 = i + 2;

                if provided == 1 {
                    obs_index = None;
                    scale_value = Some(serialized_combinators[i0]);
                    i0 += 1;
                } else {
                    let mut obs_entries: Vec<(Address, Option<i64>)> = self.storage.read_ref(&obs_entries_key()).0;

                    // Deserialize arbiter address
                    let mut serialized_address: [i64; 4] = [0; 4];
                    serialized_address.copy_from_slice(&serialized_combinators[(i + 2)..(i + 6)]);
                    let address = i64_to_address(serialized_address);

                    obs_index = Some(obs_entries.len());
                    obs_entries.push((address, None));

                    self.storage.write_ref(&obs_entries_key(), &obs_entries);
                    scale_value = None;
                    i0 += 4;
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

    // Gets the observable values
    fn get_obs_values(&mut self) -> Vec<Option<i64>> {
        let obs_entries: Vec<(Address, Option<i64>)> = self.storage.read_ref(&obs_entries_key()).0;
        obs_entries.into_iter().map(|e| e.1).collect()
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
    fn get_withdrawal_amount(amount: u64, balance: i64, funds: i64, use_gas: bool) -> i64 {
        let mut final_amount = amount as i64;
        if use_gas {
            final_amount = final_amount + CALL_GAS;
        }

        // If the withdrawer or contract can't afford the gas for the transaction, do nothing more
        if use_gas && (balance < CALL_GAS || funds < CALL_GAS) {
            return 0;
        }

        // Clamp withdrawal at balance and fund amount
        if balance < final_amount {
            final_amount = balance;
        }
        if funds < final_amount {
            final_amount = funds;
        }

        return final_amount as i64;
    }
}

// Unit tests
#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    extern crate pwasm_test;

    use super::{ FinancialScContract, FinancialScInterface, Storage, Stores, StoresRef };
    use storage::{ address_to_i64, i64_to_address };
    use super::pwasm_std::{ Vec, vec, types::{ Address, U256, H256 } };
    use self::pwasm_test::{ ext_reset, ext_update };

    // Initialise a FinancialScContract with the given values (and mock blockchain parameters)
    fn setup_contract(sender: Address, holder: Address, timestamp: u64, serialized_combinator_contract: Vec<i64>) -> FinancialScContract {
        let mut contract = FinancialScContract::new();

        ext_reset(|e| e
            .sender(sender)
            .timestamp(timestamp)
        );
        contract.constructor(serialized_combinator_contract, holder, true);
        contract
    }

    // Initialise a FinancialScContract with the given values (and mock blockchain parameters)
    fn setup_contract_no_gas(sender: Address, holder: Address, timestamp: u64, serialized_combinator_contract: Vec<i64>) -> FinancialScContract {
        let mut contract = FinancialScContract::new();

        ext_reset(|e| e
            .sender(sender)
            .timestamp(timestamp)
        );
        contract.constructor(serialized_combinator_contract, holder, false);
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

        contract.update();
        assert_eq!(contract.get_balance(true), 0);

        assert_eq!(contract.get_balance(false), 0);
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
        assert_eq!(contract.get_balance(true), 1);

        assert_eq!(contract.get_balance(false), -1);
    }

    #[test]
    fn updating_sets_last_updated_time() {
        let initial_time = 2;
        let update_time = 3;
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            initial_time,
            vec![0]
        );

        // Check the initial last-updated time is correct
        assert_eq!(contract.get_last_updated() as u64, initial_time);

        // Update
        ext_update(|e| e.timestamp(update_time));
        contract.update();

        // Check last-updated time is correct
        assert_eq!(contract.get_last_updated() as u64, update_time);
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
        assert_eq!(contract.get_balance(true), 0);

        // Check that the stake increases when added to
        let new_stake = 10;
        ext_update(|e| e
            .sender(holder)
            .value(U256::from(new_stake))
        );

        contract.stake();
        assert_eq!(new_stake, contract.get_balance(true));
        contract.stake();
        assert_eq!(contract.get_balance(true), new_stake * 2);
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
        assert_eq!(contract.get_balance(false), 0);

        // Check that the stake increases when added to
        let new_stake = 10;
        ext_update(|e| e
            .sender(sender)
            .value(U256::from(new_stake))
        );

        contract.stake();
        assert_eq!(contract.get_balance(false), new_stake);
        contract.stake();
        assert_eq!(contract.get_balance(false), new_stake * 2);
    }

    // Acquisition times returned correctly
    #[test]
    fn get_acquisition_times_returns_correct_times() {
        let combinator_contract = vec![9, 9, 9, 9, 9, 1];
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            combinator_contract.clone()
        );

        ext_update(|e| e
            .sender(holder)
            .timestamp(0)
        );
        contract.acquire();

        ext_update(|e| e.timestamp(1));
        contract.acquire_anytime_sub_contract(0);

        ext_update(|e| e.timestamp(2));
        contract.acquire_anytime_sub_contract(1);

        ext_update(|e| e.timestamp(3));
        contract.acquire_anytime_sub_contract(2);

        ext_update(|e| e.timestamp(4));
        contract.acquire_anytime_sub_contract(3);

        assert_eq!(contract.get_acquisition_times(), vec![0, 1, 2, 3, 4, -1]);
    }

    // Or choices returned correctly
    #[test]
    fn get_or_choices_returns_correct_values() {
        let combinator_contract = vec![3, 3, 3, 1, 0, 1, 0];
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            combinator_contract.clone()
        );

        ext_update(|e| e.sender(holder));

        contract.set_or_choice(0, true);
        contract.set_or_choice(1, false);

        assert_eq!(contract.get_or_choices(), vec![1, 0, 2]);
    }

    // Obs entries returned correctly
    #[test]
    fn get_obs_entries_returns_correct_entries() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let counter_party = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let arbiter: Address = "3D04E16e08E4c1c7fa8fC5A386237669341EaAcE".parse().unwrap();
        let arbiter_serialized: [i64; 4] = address_to_i64(arbiter);

        let combinator_contract = vec![
            5, -1, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
            5, -1, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
            5, -1, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
            1
        ];
        let mut contract = setup_contract(
            counter_party,
            holder,
            0,
            combinator_contract.clone()
        );

        ext_update(|e| e.sender(arbiter));
        contract.set_obs_value(0, 1);
        contract.set_obs_value(2, -1);

        assert_eq!(contract.get_obs_entries(), vec![
            arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3], 0, 1,
            arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3], -1,
            arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3], 0, -1
        ]);
    }

    // Obs values returned correctly
    #[test]
    fn get_obs_values_returns_correct_values() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let counter_party = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let arbiter: Address = "3D04E16e08E4c1c7fa8fC5A386237669341EaAcE".parse().unwrap();
        let arbiter_serialized: [i64; 4] = address_to_i64(arbiter);

        let combinator_contract = vec![
            5, -1, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
            5, -1, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
            5, -1, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
            1
        ];
        let mut contract = setup_contract(
            counter_party,
            holder,
            0,
            combinator_contract.clone()
        );

        ext_update(|e| e.sender(arbiter));
        contract.set_obs_value(0, 1);
        contract.set_obs_value(2, -1);

        assert_eq!(contract.get_obs_values(), vec![Some(1), None, Some(-1)]);
    }

    // Withdrawal amount is calculated correctly for a normal withdrawal
    #[test]
    fn get_withdrawal_amount_calculates_correct_normal_amount() {
        let balance = 10000;
        let withdrawal = 5000;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance, balance as i64, true);

        assert_eq!(amount, withdrawal as i64 + super::CALL_GAS);
    }

    // Withdrawal amount is calculated correctly for a normal withdrawal when not using gas
    #[test]
    fn get_withdrawal_amount_calculates_correct_normal_amount_no_gas() {
        let balance = 10000;
        let withdrawal = 5000;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance, balance as i64, false);

        assert_eq!(amount, withdrawal as i64);
    }

    // Withdrawal withdraws balance amount at maximum, even if requested amount is higher
    #[test]
    fn get_withdrawal_amount_clamps_withdrawal_to_balance() {
        let balance = 5000;
        let withdrawal = 10000;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance, balance as i64, true);

        assert_eq!(balance, amount);
    }

    // Withdrawal withdraws funds amount at maximum, even if requested amount is higher
    #[test]
    fn get_withdrawal_amount_clamps_withdrawal_to_funds() {
        let balance = 5000;
        let withdrawal = 10000;
        let funds = 2500;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance, funds, true);

        assert_eq!(funds, amount);
    }

    // Withdrawal does not withdraw anything if the balance is below the required call gas price
    #[test]
    fn withdraw_does_not_withdraw_if_balance_below_gas_price() {
        let balance = (super::CALL_GAS - 1) as i64;
        let withdrawal = 1 as u64;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance, 10000, true);

        assert_eq!(amount, 0);
    }

    // Withdrawal does not withdraw anything if the funds are below the required call gas price
    #[test]
    fn withdraw_does_not_withdraw_if_funds_below_gas_price() {
        let balance = 10000 as i64;
        let withdrawal = 1 as u64;
        let funds = super::CALL_GAS - 1;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance, funds as i64, true);

        assert_eq!(amount, 0);
    }

    // Withdrawal withdraws even if the balance is below the required call gas price when not using gas
    #[test]
    fn withdraw_withdraws_if_balance_below_gas_price_when_not_using_gas() {
        let balance = (super::CALL_GAS - 1) as i64;
        let withdrawal = 1 as u64;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance, 10000, false);

        assert_eq!(amount, 1);
    }

    // Withdrawal withdraws if the funds are below the required call gas price when not using gas
    #[test]
    fn withdraw_withdraws_if_funds_below_gas_price_when_not_using_gas() {
        let balance = 10000 as i64;
        let withdrawal = 1 as u64;
        let funds = super::CALL_GAS - 1;
        let amount = FinancialScContract::get_withdrawal_amount(withdrawal, balance, funds as i64, false);

        assert_eq!(amount, 1);
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