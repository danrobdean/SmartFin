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
use pwasm_std::{ Box, Vec, types::{ Address, U256 }, read_u32, read_u64 };
use pwasm_abi_derive::eth_abi;
use combinators::*;

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

// The financial smart contract interface
#[eth_abi(FinancialScEndpoint)]
pub trait FinancialScInterface {
    // The contract constructor, takes the combinator contract definition (serialized) and the holder address
    fn constructor(&mut self, contract_definition: Vec<u8>, holder: Address);

    // Gets the address of the contract holder
    #[constant]
    fn get_holder(&mut self) -> Address;

    // Gets the address of the counter-party
    #[constant]
    fn get_counter_party(&mut self) -> Address;

    // Gets the combinator contract definition, returns the combinator contract serialized
    #[constant]
    fn get_contract_definition(&mut self) -> Vec<u8>;

    // Gets the current value of the contract (TODO: for dev purposes)
    #[constant]
    fn get_value(&mut self) -> i64;

    // Gets the current stake of the caller (if called by the holder or counter-party)
    #[constant]
    fn get_stake(&mut self) -> U256;

    // Sets the preference of the given or combinator's sub-combinators
    fn set_or_choice(&mut self, or_index: u64, choice: bool);

    // Proposes a value for the given observable
    fn propose_obs_value(&mut self, obs_index: u64, value: i64);

    // Stakes Eth with the contract (can be called by the holder or counter-party), returns the caller's total stake
    #[payable]
    fn stake(&mut self) -> U256;
}

// The financial smart contract
pub struct FinancialScContract {
    // The contract holder
    holder: Address,

    // The contract's counter-party (the author)
    counter_party: Address,

    // The serialized combinator contract
    serialized_combinators: Vec<u8>,

    // The combinator contract
    combinator: Box<ContractCombinator>,

    // The counter-party's current stake
    counter_party_stake: U256,

    // The holder's current stake
    holder_stake: U256,

    // The choices for each or combinator
    or_choices: Vec<Option<bool>>,

    // The values of each observable proposed by the holder
    holder_proposed_obs_values: Vec<Option<i64>>,

    // The values of each observable proposed by the counter-party
    counter_party_proposed_obs_values: Vec<Option<i64>>,

    // The concrete values of each observable
    concrete_obs_values: Vec<Option<i64>>
}

// The financial smart contract interface implementation
impl FinancialScInterface for FinancialScContract {
    // The financial smart contract constructor
    fn constructor(&mut self, contract_definition: Vec<u8>, holder: Address) {
        if holder == pwasm_ethereum::sender() {
            panic!("Holder and counter-party must be different addresses.")
        }

        self.holder = holder;
        self.counter_party = pwasm_ethereum::sender();
        self.serialized_combinators = contract_definition;
        self.set_combinator();
    }

    // Gets the address of the holder
    fn get_holder(&mut self) -> Address {
        self.holder
    }

    // Gets the address of the counter-party
    fn get_counter_party(&mut self) -> Address {
        self.counter_party
    }

    // Gets the combinator contract definition (serialized)
    fn get_contract_definition(&mut self) -> Vec<u8> {
        self.serialized_combinators.clone()
    }

    // Gets the current value of the contract
    fn get_value(&mut self) -> i64 {
        self.combinator.get_value(pwasm_ethereum::timestamp() as u32, &self.or_choices, &self.concrete_obs_values)
    }

    // Gets the total stake of the caller
    fn get_stake(&mut self) -> U256 {
        let sender = pwasm_ethereum::sender();

        if sender == self.holder {
            self.holder_stake
        } else if sender == self.counter_party {
            self.counter_party_stake
        } else {
            panic!("Only the contract holder or the counter-party may have stake in the contract.")
        }
    }

    // Sets the given or combinator's preference between its sub-combinators
    fn set_or_choice(&mut self, or_index: u64, prefer_first: bool) {
        if pwasm_ethereum::sender() != self.holder {
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
            panic!("Given observable-index does not exist.")
        }

        // Set the proposed value for the sender
        let sender = pwasm_ethereum::sender();

        if sender == self.holder {
            self.holder_proposed_obs_values[index] = Some(value);
        } else if sender == self.counter_party {
            self.counter_party_proposed_obs_values[index] = Some(value);
        } else {
            panic!("Only the holder or counter-party set the value of observables.");
        }

        // Check if proposed values match
        if self.holder_proposed_obs_values[index] == self.counter_party_proposed_obs_values[index] {
            self.concrete_obs_values[index] = self.holder_proposed_obs_values[index];
        }
    }

    // Stakes Eth with the contract, returns the caller's total stake
    fn stake(&mut self) -> U256 {
        let sender = pwasm_ethereum::sender();

        if sender == self.holder {
            self.holder_stake = FinancialScContract::safe_add(self.holder_stake, pwasm_ethereum::value());
            self.holder_stake
        } else if sender == self.counter_party {
            self.counter_party_stake = FinancialScContract::safe_add(self.counter_party_stake, pwasm_ethereum::value());
            self.counter_party_stake
        } else {
            panic!("Only the contract holder or the counter-party may stake ether in the contract.")
        }
    }
}

// Financial smart contract functions which aren't part of the ABI
impl FinancialScContract {
    // Instantiates a new financial smart contract
    pub fn new() -> FinancialScContract {
        FinancialScContract{
            holder: Address::zero(),
            counter_party: Address::zero(),
            serialized_combinators: Vec::new(),
            combinator: Box::new(NullCombinator::new()),
            counter_party_stake: 0.into(),
            holder_stake: 0.into(),
            or_choices: Vec::new(),
            holder_proposed_obs_values: Vec::new(),
            counter_party_proposed_obs_values: Vec::new(),
            concrete_obs_values: Vec::new()
        }
    }

    // Constructs the combinators from a serialized combinator contract
    fn set_combinator(&mut self) {
        let (_i, combinator) = self.deserialize_combinator(0);
        self.combinator = combinator;
    }

    // Deserializes a combinator from the given combinator byte vector and index, returns the following index and the boxed combinator
    fn deserialize_combinator(&mut self, i: usize) -> (usize, Box<ContractCombinator>) {
        if i >= self.serialized_combinators.len() {
            panic!("Provided combinator contract not valid.");
        }

        match self.serialized_combinators[i] {
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
                let mut timestamp: u32 = FinancialScContract::deserialize_32_bit_int(&self.serialized_combinators, i + 1);

                // Deserialize sub-combinator
                let (i0, sub_combinator) = self.deserialize_combinator(i + 5);

                (i0, Box::new(TruncateCombinator::new(sub_combinator, timestamp)))
            },

            // scale combinator
            5 => {
                // Check if observable is provided, if so then deserialize it, otherwise record in obs_values
                let provided: u8 = self.serialized_combinators[i + 1];
                let mut obs_index: Option<usize>;
                let mut scale_value: Option<i64>;
                let mut i0 = i + 2;

                if provided == 1 {
                    obs_index = None;
                    scale_value = Some(FinancialScContract::deserialize_signed_64_bit_int(&self.serialized_combinators, i0));
                    i0 += 8;
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
            }

            // Unrecognised combinator
            _ => panic!("Unrecognised combinator provided.")
        }
    }

    // Add numbers safely to avoid integer overflow
    fn safe_add(x: U256, y: U256) -> U256 {
        if x > U256::MAX - y {
            panic!("Integer overflow.")
        }
        x + y
    }

    // Deserialize a 32-bit integer
    fn deserialize_32_bit_int(serialized_data: &Vec<u8>, start_i: usize) -> u32 {
        // Deserialize accounting for endianness
        read_u32(&serialized_data[start_i..start_i+4])
    }

    // Deserialize a signed 64-bit integer
    fn deserialize_signed_64_bit_int(serialized_data: &Vec<u8>, start_i: usize) -> i64 {
        // Deserialize as u64 accounting for endianness
        let unsigned = read_u64(&serialized_data[start_i..start_i + 8]);

        // Convert to i64 from u64
        let mut signed: i64;
        if unsigned > 2_u64.pow(63) {
            // Most significant bit is 1, convert to two's complement negative
            signed = (unsigned - 2_u64.pow(63)) as i64; // Allowed as unsigned > 2^63, so unsigned - 2^63 > 0

            // Subtract 2^63, using 2 * 2^62 (as i64::MAX < 2^63)
            signed -= 2_i64.pow(62);
            signed -= 2_i64.pow(62);
        } else {
            // Most significant bit is 0, no conversion to negative needed
            signed = unsigned as i64;
        }
        signed
    }
}

// Unit tests
#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    extern crate pwasm_test;

    use super::{ FinancialScContract, FinancialScInterface };
    use super::pwasm_std::{ Vec, vec, types::{ Address, U256 } };
    use self::pwasm_test::ext_reset;

    // Initialise a FinancialScContract with the given values (and mock blockchain parameters)
    fn setup_contract(sender: Address, holder: Address, timestamp: u64, serialized_combinator_contract: Vec<u8>) -> FinancialScContract {
        let mut contract = FinancialScContract::new();

        ext_reset(|e| e
            .sender(sender)
            .timestamp(timestamp)
        );
        contract.constructor(serialized_combinator_contract, holder);
        contract
    }

    // 32-bit integers are deserialized correctly
    #[test]
    fn deserialize_32_bit_integer_correct() {
        let deserialized_data: Vec<u8> = vec![2, 2, 99, 16, 127, 79, 2];

        let int = FinancialScContract::deserialize_32_bit_int(&deserialized_data, 2);
        let expected = deserialized_data[2] as u32 * 2_u32.pow(0)
            + deserialized_data[3] as u32 * 2_u32.pow(8)
            + deserialized_data[4] as u32 * 2_u32.pow(16)
            + deserialized_data[5] as u32 * 2_u32.pow(24);

        assert_eq!(
            expected,
            int,
            "Serialized u32 {:?} is not deserialized correctly: {}",
            &deserialized_data[2..6],
            int
        );
    }

    // Signed 64-bit integers are deserialized correctly
    #[test]
    fn deserialize_signed_64_bit_integer_correct() {
        let deserialized_data: Vec<u8> = vec![2, 2, 129, 16, 220, 79, 189, 129, 5, 129, 2];

        let int = FinancialScContract::deserialize_signed_64_bit_int(&deserialized_data, 2);
        let mut expected = deserialized_data[2] as i64 * 2_i64.pow(0)
            + deserialized_data[3] as i64 * 2_i64.pow(8)
            + deserialized_data[4] as i64 * 2_i64.pow(16)
            + deserialized_data[5] as i64 * 2_i64.pow(24)
            + deserialized_data[6] as i64 * 2_i64.pow(32)
            + deserialized_data[7] as i64 * 2_i64.pow(40)
            + deserialized_data[8] as i64 * 2_i64.pow(48);

        // Handle most significant byte to enable negativity
        if deserialized_data[9] > 2_u8.pow(7) {
            expected += (deserialized_data[9] as i64 - 2_i64.pow(7)) * 2_i64.pow(56);
            expected -= 2_i64.pow(62);
            expected -= 2_i64.pow(62);
        } else {
            expected += deserialized_data[9] as i64 * 2_i64.pow(56);
        }

        assert_eq!(
            expected,
            int,
            "Serialized i64 {:?} is not deserialized correctly: {}",
            &deserialized_data[2..10],
            int
        );
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

    // Staking Eth as the holder stakes the correct amount
    #[test]
    fn holder_stake_updates() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![0]
        );

        // Check that the initial stake is 0
        ext_reset(|e| e.sender(holder));
        assert_eq!(contract.get_stake(), U256::from(0));

        // Check that the stake increases when added to
        let new_stake = U256::from(10);
        ext_reset(|e| e
            .sender(holder)
            .value(new_stake)
        );

        contract.stake();
        assert_eq!(new_stake, contract.get_stake());
        contract.stake();
        assert_eq!(contract.get_stake(), new_stake * U256::from(2));
    }

    // Staking Eth as the counter-party stakes the correct amount
    #[test]
    fn counter_party_stake_updates() {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = setup_contract(
            sender,
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        // Check that the initial stake is 0
        ext_reset(|e| e.sender(sender));
        assert_eq!(contract.get_stake(), U256::from(0));

        // Check that the stake increases when added to
        let new_stake = U256::from(10);
        ext_reset(|e| e
            .sender(sender)
            .value(new_stake)
        );

        contract.stake();
        assert_eq!(contract.get_stake(), new_stake);
        contract.stake();
        assert_eq!(contract.get_stake(), new_stake * U256::from(2));
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

    // Overflowing the holder's stake is not allowed
    #[test]
    #[should_panic(expected = "Integer overflow.")]
    fn should_panic_if_holder_stake_overflows() {
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let mut contract = setup_contract(
            "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap(),
            holder,
            0,
            vec![0]
        );

        // Set the stake to the maximum U256 value
        ext_reset(|e| e
            .sender(holder)
            .value(U256::MAX)
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
    fn should_panic_if_counter_party_stake_overflows() {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = setup_contract(
            sender,
            "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap(),
            0,
            vec![0]
        );

        // Set the stake to the maximum U256 value
        ext_reset(|e| e
            .sender(sender)
            .value(U256::MAX)
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
        contract.get_stake();
    }

    // An uninvolved user attempting to stake Eth is not allowed
    #[test]
    #[should_panic(expected = "Only the contract holder or the counter-party may stake ether in the contract.")]
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