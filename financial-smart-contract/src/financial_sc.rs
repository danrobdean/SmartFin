// The financial smart contract
#![allow(non_snake_case)]

extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate pwasm_abi_derive;

use self::pwasm_std::types::Address;
use self::pwasm_abi_derive::eth_abi;

use combinators::contract_combinator::ContractCombinator;
use combinators::zero_combinator::ZeroCombinator;

// The financial smart contract interface
#[eth_abi(FinancialScEndpoint)]
pub trait FinancialScInterface {
    // The contract constructor
    fn constructor(&mut self);

    // Gets the address of the counter-party
    #[constant]
    fn get_counter_party(&mut self) -> Address;
}

// The financial smart contract
pub struct FinancialScContract {
    counter_party: Address,
    combinator: ContractCombinator
}

// The financial smart contract interface implementation
impl FinancialScInterface for FinancialScContract {
    // The financial smart contract constructor
    fn constructor(&mut self) {
        // Store the address of the counter-party
        self.counter_party = pwasm_ethereum::sender();
    }

    // Gets the address of the counter-party
    fn get_counter_party(&mut self) -> Address {
        return self.counter_party;
    }
}

// Financial smart contract functions which aren't part of the ABI
impl FinancialScContract {
    // Instantiates a new financial smart contract
    pub fn new() -> FinancialScContract {
        FinancialScContract{
            counter_party: Address::zero(),
            combinator: ZeroCombinator::new()
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    extern crate std;
    use super::*;
    use self::pwasm_test::{ext_reset};

    // The counter-party of the contract is set to the deployer
    #[test]
    fn correct_counter_party() {
        let mut contract = FinancialScContract::new();

        // Mock sender address
        let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
        ext_reset(|e| e.sender(sender));

        // Call contract functions
        contract.constructor();
        let counter_party = contract.get_counter_party();

        // Check that counter-party is correct
        assert_eq!(counter_party, sender);
    }
}