// The financial smart contract
#![allow(non_snake_case)]

extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate pwasm_abi_derive;

use self::pwasm_std::types::{Address, U256};
use self::pwasm_abi_derive::eth_abi;
use self::pwasm_std::{Box, Vec};

use combinators::contract_combinator::ContractCombinator;
use combinators::zero_combinator::ZeroCombinator;
use combinators::one_combinator::OneCombinator;
use combinators::and_combinator::AndCombinator;

// The financial smart contract interface
#[eth_abi(FinancialScEndpoint)]
pub trait FinancialScInterface {
    // The contract constructor
    fn constructor(&mut self, contract_definition: Vec<u8>);

    // Gets the address of the counter-party
    #[constant]
    fn get_counter_party(&mut self) -> Address;

    // Gets the current value of the contract
    fn get_value(&mut self) -> U256;
}

// The financial smart contract
pub struct FinancialScContract {
    counter_party: Address,
    combinator: Option<Box<ContractCombinator>>
}

// The financial smart contract interface implementation
impl FinancialScInterface for FinancialScContract {
    // The financial smart contract constructor
    fn constructor(&mut self, contract_definition: Vec<u8>) {
        // Store the address of the counter-party
        self.counter_party = pwasm_ethereum::sender();
        self.set_combinator(contract_definition);
    }

    // Gets the address of the counter-party
    fn get_counter_party(&mut self) -> Address {
        self.counter_party
    }

    // Gets the current value of the contract
    fn get_value(&mut self) -> U256 {
        match self.combinator {
            Some(ref combinator) => combinator.acquire(pwasm_ethereum::timestamp()).into(),
            None => panic!("Attempted to get value before combinators initialised")
        }
    }
}

// Financial smart contract functions which aren't part of the ABI
impl FinancialScContract {
    // Instantiates a new financial smart contract
    pub fn new() -> FinancialScContract {
        FinancialScContract{
            counter_party: Address::zero(),
            combinator: None
        }
    }

    // Constructs the combinators from a serialized combinator contract
    fn set_combinator(&mut self, serialized_combinators: Vec<u8>) {
        let (_i, combinator) = FinancialScContract::deserialize_combinator(0, &serialized_combinators);
        self.combinator = Some(combinator);
    }

    // Deserializes a combinator from the given combinator byte vector and index, returns the following index and the boxed combinator
    fn deserialize_combinator(i: usize, serialized_combinators: &[u8]) -> (usize, Box<ContractCombinator>) {
        if i > serialized_combinators.len() {
            panic!("Provided combinator contract not valid.");
        }

        match serialized_combinators[i] {
            0 => (i + 1, Box::new(ZeroCombinator::new())),
            1 => (i + 1, Box::new(OneCombinator::new())),
            2 => {
                let (i0, sub_combinator0) = FinancialScContract::deserialize_combinator(i + 1, serialized_combinators);
                let (i1, sub_combinator1) = FinancialScContract::deserialize_combinator(i0, serialized_combinators);
                (i1, Box::new(AndCombinator::new(sub_combinator0, sub_combinator1)))
            },
            _ => panic!("Unrecognised combinator provided.")
        }
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

    // Setup contract with default values, returns the contract
    fn setup_contract(deserialized_combinator: Vec<u8>) -> FinancialScContract {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = FinancialScContract::new();

        // Mock values
        ext_reset(|e| e
            .sender(sender)
            .timestamp(0)
        );
        contract.constructor(deserialized_combinator);

        contract
    }

    // The counter-party of the contract is set to the deployer
    #[test]
    fn correct_counter_party() {
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        let mut contract = FinancialScContract::new();

        // Mock values
        ext_reset(|e| e
            .sender(sender)
        );
        contract.constructor(vec![0]);

        // Check that counter-party is correct
        let counter_party = contract.get_counter_party();
        assert_eq!(counter_party, sender);
    }

    // The value of the contract is based on the given serialized combinator vector
    #[test]
    fn correct_value_zero() {
        let mut contract = setup_contract(vec![0]);

        // Check that the value is correct
        let value: u64 = contract.get_value().low_u64();
        assert_eq!(value, 0);
    }

    // The value of the contract is based on the given serialized combinator vector
    #[test]
    fn correct_value_one() {
        let mut contract = setup_contract(vec![1]);

        // Check that the value is correct
        let value: u64 = contract.get_value().low_u64();
        assert_eq!(value, 1);
    }

    // The value of the contract is based on the given serialized combinator vector
    #[test]
    fn correct_value_and() {
        let mut contract = setup_contract(vec![2, 1, 1]);

        // Check that the value is correct
        let value: u64 = contract.get_value().low_u64();
        assert_eq!(value, 2);
    }

    // The value of the contract is based on the given serialized combinator vector
    #[test]
    fn contract_ignores_extra_combinators_in_serialized_vector() {
        let mut contract = setup_contract(vec![0, 2, 1, 1]);

        // Check that the value is correct
        let value: u64 = contract.get_value().low_u64();
        assert_eq!(value, 0);
    }
    
    // Test that an empty deserialized combinator vector is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_no_combinators_given() {
        setup_contract(vec![]);
    }

    // Test that an undefined combinator vector value is not allowed
    #[test]
    #[should_panic]
    fn should_panic_if_combinator_value_unrecognised() {
        setup_contract(vec![255]);
    }
}