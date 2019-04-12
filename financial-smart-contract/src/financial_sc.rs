// The financial smart contract
#![allow(non_snake_case)]

extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate pwasm_abi_derive;

use self::pwasm_std::types::{Address, U256};
use self::pwasm_abi_derive::eth_abi;
use self::pwasm_std::{Box, Vec};

use combinators::*;

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
    fn get_value(&mut self) -> U256;

    // Gets the current stake of the caller (if called by the holder or counter-party)
    #[constant]
    fn get_stake(&mut self) -> U256;

    // Sets the preference of the given or combinator's sub-combinators
    fn set_or_choice(&mut self, or_index: U256, choice: bool);

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

    // The values of each observable
    obs_values: Vec<Option<u64>>
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
    fn get_value(&mut self) -> U256 {
        self.combinator.get_value(pwasm_ethereum::timestamp() as u32, &self.or_choices, &self.obs_values).into()
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
    fn set_or_choice(&mut self, or_index: U256, prefer_first: bool) {
        if or_index >= U256::from(self.or_choices.len()) {
            panic!("Given or-index does not exist.");
        }
        self.or_choices[or_index.low_u64() as usize] = Some(prefer_first);
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
            obs_values: Vec::new()
        }
    }

    // Constructs the combinators from a serialized combinator contract
    fn set_combinator(&mut self) {
        let (_i, combinator) = self.deserialize_combinator(0);
        self.combinator = combinator;
    }

    // Deserializes a combinator from the given combinator byte vector and index, returns the following index and the boxed combinator
    fn deserialize_combinator(&mut self, i: usize) -> (usize, Box<ContractCombinator>) {
        if i > self.serialized_combinators.len() {
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
                let mut scale_value: Option<u64>;
                let mut i0 = i + 2;

                if provided == 1 {
                    obs_index = None;
                    scale_value = Some(FinancialScContract::deserialize_64_bit_int(&self.serialized_combinators, i0));
                    i0 += 8;
                } else {
                    obs_index = Some(self.obs_values.len());
                    self.obs_values.push(None);
                    scale_value = None;
                }

                // Deserialize sub-contract
                let (i1, sub_combinator) = self.deserialize_combinator(i0);

                (i1, Box::new(ScaleCombinator::new(sub_combinator, obs_index, scale_value)))
            }

            // Unrecognised combinator
            _ => panic!("Unrecognised combinator provided.")
        }
    }

    // Add numbers safely to avoid integer overflow
    fn safe_add(x: U256, y: U256) -> U256 {
        let z = x + y;
        if z < x {
            panic!("Integer overflow.")
        }
        z
    }

    // Deserialize a 32-bit integer
    fn deserialize_32_bit_int(serialized_data: &Vec<u8>, start_i: usize) -> u32 {
        let mut int: u32 = 0;
        let mut byte = 0;

        while byte < 4 {
            int = (int * 256) + (serialized_data[start_i + byte] as u32);
            byte += 1;
        }

        int
    }

    // Deserialize a 64-bit integer
    fn deserialize_64_bit_int(serialized_data: &Vec<u8>, start_i: usize) -> u64 {
        let mut int: u64 = 0;
        let mut byte = 0;

        while byte < 8 {
            int = (int * 256) + serialized_data[start_i + byte] as u64;
            byte += 1;
        }

        int
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    extern crate pwasm_std;

    use super::*;
    use self::pwasm_test::ext_reset;
    use self::pwasm_std::vec;

    // The details of a contract used for testing
    struct TestContractDetails {
        holder: Address,
        counter_party: Address,
        timestamp: U256,
        contract: FinancialScContract
    }

    // Method implementation for the details of a testing contract
    impl TestContractDetails {
        // Instantiates a new set of contract details
        fn new(holder: Address, counter_party: Address, timestamp: U256, contract: FinancialScContract) -> TestContractDetails {
            TestContractDetails {
                holder,
                counter_party,
                timestamp,
                contract
            }
        }
    }

    // Setup contract with default values, returns the contract
    fn setup_contract(deserialized_combinator: Vec<u8>) -> TestContractDetails {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
        let timestamp = 0;
        let mut contract = FinancialScContract::new();

        // Mock values
        ext_reset(|e| e
            .sender(sender)
            .timestamp(timestamp)
        );
        contract.constructor(deserialized_combinator, holder);

        TestContractDetails::new(holder, sender, timestamp.into(), contract)
    }

    // Converts a 32-bit int to a 4-bit array
    fn serialize_32_bit_int(mut int: u32) -> [u8; 4] {
        let mut serialized: [u8; 4] = [0; 4];
        let mut byte: i32 = 3;
        while byte >= 0 {
            serialized[byte as usize] = (int % 256) as u8;
            int /= 256;
            byte -= 1;
        }

        serialized
    }

    // Converts a 64-bit int to an 8-bit array
    fn serialize_64_bit_int(mut int: u64) -> [u8; 8] {
        let mut serialized: [u8; 8] = [0; 8];
        let mut byte: i32 = 7;
        while byte >= 0 {
            serialized[byte as usize] = (int % 256) as u8;
            int /= 256;
            byte -= 1;
        }

        serialized
    }

    // The counter-party of the contract is set to the deployer
    #[test]
    fn correct_counter_party() {
        let mut contract_details = setup_contract(vec![0]);

        // Check that holder is correct
        let counter_party = contract_details.contract.get_counter_party();
        assert_eq!(contract_details.counter_party, counter_party);
    }

    // The holder of the contract is set to the provided address
    #[test]
    fn correct_holder() {
        let mut contract_details = setup_contract(vec![0]);

        // Check that holder is correct
        let holder = contract_details.contract.get_holder();
        assert_eq!(contract_details.holder, holder);
    }

    // The serialized combinator contract is set to the provided combinator contract
    #[test]
    fn correct_combinator_contract() {
        let combinator_contract = vec![2, 2, 1, 0, 2, 2, 0, 0, 1];
        let mut contract = setup_contract(combinator_contract.clone()).contract;

        // Check that the value is correct
        let registered_combinator_contract = contract.get_contract_definition();
        assert_eq!(registered_combinator_contract, combinator_contract);
    }

    // The value of the contract is based on the given serialized combinator vector
    #[test]
    fn correct_value_zero() {
        let mut contract = setup_contract(vec![0]).contract;

        // Check that the value is correct
        let value: u64 = contract.get_value().low_u64();
        assert_eq!(value, 0);
    }

    // The value of the contract is based on the given serialized combinator vector
    #[test]
    fn correct_value_one() {
        let mut contract = setup_contract(vec![1]).contract;

        // Check that the value is correct
        let value: u64 = contract.get_value().low_u64();
        assert_eq!(value, 1);
    }

    // The value of the contract is based on the given serialized combinator vector
    #[test]
    fn correct_value_and() {
        let mut contract = setup_contract(vec![2, 1, 1]).contract;

        // Check that the value is correct
        let value: u64 = contract.get_value().low_u64();
        assert_eq!(value, 2);
    }

    // The value of the or combinator is correct given a left or choice
    #[test]
    fn correct_value_or_left() {
        let mut contract = setup_contract(vec![3, 0, 1]).contract;
        
        // Set the or choice and check the value
        contract.set_or_choice(U256::from(0), true);
        assert_eq!(contract.get_value(), U256::from(0));
    }

    // The value of the or combinator is correct given a right or choice
    #[test]
    fn correct_value_or_right() {
        let mut contract = setup_contract(vec![3, 0, 1]).contract;
        
        // Set the or choice and check the value
        contract.set_or_choice(U256::from(0), false);
        assert_eq!(contract.get_value(), U256::from(1));
    }

    // The value of an expired truncated contract is 0
    #[test]
    fn expired_truncate_worth_0() {
        // Create contract truncate 0 one
        let mut timestamp = serialize_32_bit_int(0).to_vec();
        let mut combinator_contract = vec![4];
        combinator_contract.append(&mut timestamp);
        combinator_contract.append(&mut vec![1]);
        let mut contract = setup_contract(combinator_contract).contract;

        // Check that contract value is 0 at timestamp 1
        ext_reset(|e| e.timestamp(1));
        assert_eq!(contract.get_value(), U256::from(0));
    }

    // The value of a non-expired truncated contract is correct
    #[test]
    fn non_expired_truncate_value_correct() {
        // Create contract truncate 1 one
        let mut timestamp = serialize_32_bit_int(1).to_vec();
        let mut combinator_contract = vec![4];
        combinator_contract.append(&mut timestamp);
        combinator_contract.append(&mut vec![1]);
        let mut contract = setup_contract(combinator_contract).contract;

        // Check that contract value is 1 at timestamp 0
        ext_reset(|e| e.timestamp(0));
        assert_eq!(contract.get_value(), U256::from(1));
    }

    // The value of and with one expired sub-contract is correct
    #[test]
    fn expired_and_correct() {
        // Create contract and truncate 0 one one
        let mut timestamp = serialize_32_bit_int(0).to_vec();
        let mut combinator_contract = vec![3, 4];
        combinator_contract.append(&mut timestamp);
        combinator_contract.append(&mut vec![1, 1]);
        let mut contract = setup_contract(combinator_contract).contract;

        // Check that contract value is 1 at timestamp 1
        ext_reset(|e| e.timestamp(1));
        assert_eq!(contract.get_value(), U256::from(1));
    }

    // The value of or with one expired sub-contract is correct
    #[test]
    fn expired_or_correct() {
        // Create contract or truncate 0 one zero
        let mut timestamp = serialize_32_bit_int(0).to_vec();
        let mut combinator_contract = vec![3, 4];
        combinator_contract.append(&mut timestamp);
        combinator_contract.append(&mut vec![1, 0]);
        let mut contract = setup_contract(combinator_contract).contract;

        // Check that contract value is 0 at timestamp 1 with no or-choice
        ext_reset(|e| e.timestamp(1));
        assert_eq!(contract.get_value(), U256::from(0));
    }

    // The value of or with one expired sub-contract and a conflicting or-choice is correct
    #[test]
    fn expired_or_ignores_choice() {
        // Create contract or truncate 0 one zero
        let mut timestamp = serialize_32_bit_int(0).to_vec();
        let mut combinator_contract = vec![3, 4];
        combinator_contract.append(&mut timestamp);
        combinator_contract.append(&mut vec![1, 0]);
        let mut contract = setup_contract(combinator_contract).contract;

        // Check that contract value is 0 at timestamp 1 with left or-choice
        contract.set_or_choice(U256::from(0), true);
        ext_reset(|e| e.timestamp(1));
        assert_eq!(contract.get_value(), U256::from(0));
    }

    // The value of a scale combinator with the scale value provided is correct
    #[test]
    fn scale_with_provided_scale_value_has_correct_value() {
        // Create contract or scale 2 one
        let mut scale_value = serialize_64_bit_int(2).to_vec();
        let mut combinator_contract = vec![5, 1];
        combinator_contract.append(&mut scale_value);
        combinator_contract.append(&mut vec![1]);
        let mut contract = setup_contract(combinator_contract).contract;

        // Check that contract value is 2
        assert_eq!(contract.get_value(), U256::from(2));
    }

    // The value of the contract is based on the given serialized combinator vector
    #[test]
    fn contract_ignores_extra_combinators_in_serialized_vector() {
        let mut contract = setup_contract(vec![0, 2, 1, 1]).contract;

        // Check that the value is correct
        let value: u64 = contract.get_value().low_u64();
        assert_eq!(value, 0);
    }

    // Staking Eth as the holder stakes the correct amount
    #[test]
    fn holder_stake_updates() {
        let mut contract_details = setup_contract(vec![0]);

        // Check that the initial stake is 0
        ext_reset(|e| e.sender(contract_details.holder));
        assert_eq!(U256::from(0), contract_details.contract.get_stake());

        // Check that the stake increases when added to
        let new_stake = U256::from(10);
        ext_reset(|e| e
            .sender(contract_details.holder)
            .value(new_stake)
        );

        contract_details.contract.stake();
        assert_eq!(new_stake, contract_details.contract.get_stake());
        contract_details.contract.stake();
        assert_eq!(new_stake * U256::from(2), contract_details.contract.get_stake());
    }

    // Staking Eth as the counter-party stakes the correct amount
    #[test]
    fn counter_party_stake_updates() {
        let mut contract_details = setup_contract(vec![0]);

        // Check that the initial stake is 0
        ext_reset(|e| e.sender(contract_details.counter_party));
        assert_eq!(U256::from(0), contract_details.contract.get_stake());

        // Check that the stake increases when added to
        let new_stake = U256::from(10);
        ext_reset(|e| e
            .sender(contract_details.counter_party)
            .value(new_stake)
        );

        contract_details.contract.stake();
        assert_eq!(new_stake, contract_details.contract.get_stake());
        contract_details.contract.stake();
        assert_eq!(new_stake * U256::from(2), contract_details.contract.get_stake());
    }

    // Attempting to get the value of a contract before calling the constructor is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_not_initialised() {
        let mut contract = FinancialScContract::new();
        contract.get_value();
        contract.set_combinator();
    }

    // Attempting to create a contract with the same holder and counter-party should panic
    #[test]
    #[should_panic]
    fn should_panic_if_holder_equals_counter_party() {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = FinancialScContract::new();

        // Mock values
        ext_reset(|e| e
            .sender(sender)
        );
        contract.constructor(vec![0], sender);
    }

    // An empty deserialized combinator vector is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_no_combinators_given() {
        setup_contract(vec![]);
    }

    // An undefined combinator vector value is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_combinator_value_unrecognised() {
        setup_contract(vec![255]);
    }

    // Evaluating a contract with an ambiguous or choice is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_ambiguous_or_choice() {
        let mut contract = setup_contract(vec![3, 1, 0]).contract;
        contract.get_value();
    }

    // Providing an or choice for a non-existent or combinator is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_non_existent_or_choice_provided() {
        let mut contract = setup_contract(vec![0]).contract;
        contract.set_or_choice(U256::from(0), true);
    }

    // Getting the value of a contract with an undefined observable is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_getting_value_with_undefined_observable() {
        let mut contract = setup_contract(vec![5, 0, 1]).contract;
        contract.get_value();
    }

    // Overflowing the holder's stake is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_holder_stake_overflows() {
        let mut contract_details = setup_contract(vec![0]);

        // Set the stake to the maximum U256 value
        ext_reset(|e| e
            .sender(contract_details.holder)
            .value(U256::MAX)
        );
        contract_details.contract.stake();

        // Overflow the stake
        ext_reset(|e| e
            .sender(contract_details.holder)
            .value(U256::from(1))
        );
        contract_details.contract.stake();
    }

    // Overflowing the counter-party's stake is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_counter_party_stake_overflows() {
        let mut contract_details = setup_contract(vec![0]);

        // Set the stake to the maximum U256 value
        ext_reset(|e| e
            .sender(contract_details.counter_party)
            .value(U256::MAX)
        );
        contract_details.contract.stake();

        // Overflow the stake
        ext_reset(|e| e
            .sender(contract_details.counter_party)
            .value(U256::from(1))
        );
        contract_details.contract.stake();
    }

    // An uninvolved user attempting to get their stake is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_uninvolved_user_checks_stake() {
        let mut contract = setup_contract(vec![0]).contract;

        // Check state as an uninvolved user
        ext_reset(|e| e.sender(Address::zero()));
        contract.get_stake();
    }

    // An uninvolved user attempting to stake Eth is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_uninvolved_user_stakes() {
        let mut contract = setup_contract(vec![0]).contract;

        // Check state as an uninvolved user
        ext_reset(|e| e
            .sender(Address::zero())
            .value(U256::from(10))
        );
        contract.stake();
    }
}