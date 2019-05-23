extern crate pwasm_std;
extern crate pwasm_test;
extern crate financial_smart_contract;

pub use self::financial_smart_contract::{ FinancialScContract, FinancialScInterface, address_to_i64 };

use self::pwasm_std::{ types::Address };
use self::pwasm_test::ext_reset;

// The details of a contract used for testing
pub struct TestContractDetails {
    pub holder: Address,
    pub counter_party: Address,
    pub timestamp: u64,
    pub contract: FinancialScContract
}

// Method implementation for the details of a testing contract
impl TestContractDetails {
    // Instantiates a new set of contract details
    fn new(holder: Address, counter_party: Address, timestamp: u64, contract: FinancialScContract) -> TestContractDetails {
        TestContractDetails {
            holder,
            counter_party,
            timestamp,
            contract
        }
    }
}

// Setup contract with default values, returns the contract
pub fn setup_contract(deserialized_combinator: Vec<i64>) -> TestContractDetails {
    let sender = "1818909b947a9FA7f5Fe42b0DD1b2f9E9a4F903f".parse().unwrap();
    let holder = "25248F6f32B37f69A92dAf05d5647981b58Aaec4".parse().unwrap();
    let timestamp = 0;
    let mut contract = FinancialScContract::new();

    // Mock values
    ext_reset(|e| e
        .sender(sender)
        .timestamp(timestamp)
    );
    contract.constructor(deserialized_combinator, holder, true);

    TestContractDetails::new(holder, sender, timestamp, contract)
}