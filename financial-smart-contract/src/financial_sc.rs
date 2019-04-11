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

    // Stakes Eth with the contract (can be called by the holder or counter-party), returns the caller's total stake
    #[payable]
    fn stake(&mut self) -> U256;
}

// The financial smart contract
pub struct FinancialScContract {
    holder: Address,
    counter_party: Address,
    serialized_combinators: Vec<u8>,
    combinator: Box<ContractCombinator>,
    counter_party_stake: U256,
    holder_stake: U256
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
        self.combinator.get_value(pwasm_ethereum::timestamp()).into()
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
            holder_stake: 0.into()
        }
    }

    // Constructs the combinators from a serialized combinator contract
    fn set_combinator(&mut self) {
        let (_i, combinator) = FinancialScContract::deserialize_combinator(0, &self.serialized_combinators);
        self.combinator = combinator;
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

    // Add numbers safely to avoid integer overflow
    fn safe_add(x: U256, y: U256) -> U256 {
        let z = x + y;
        if z < x {
            panic!("Integer overflow.")
        }
        z
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