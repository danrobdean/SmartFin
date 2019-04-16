extern crate pwasm_std;
extern crate pwasm_test;
extern crate financial_smart_contract;

pub use self::financial_smart_contract::{ FinancialScContract, FinancialScInterface };

use self::pwasm_std::{ write_u32, write_u64, types::Address };
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
pub fn setup_contract(deserialized_combinator: Vec<u8>) -> TestContractDetails {
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

    TestContractDetails::new(holder, sender, timestamp, contract)
}

// Converts a 32-bit int to a 4-bit array
pub fn serialize_32_bit_int(val: u32) -> [u8; 4] {
    // Serialize accounting for endianness
    let mut res: [u8; 4] = [0; 4];
    write_u32(&mut res, val);
    res
}

// Converts a signed 64-bit int to an 8-bit array
pub fn serialize_signed_64_bit_int(mut val: i64) -> [u8; 8] {
    // Convert to u64
    let unsigned;
    if val < 0 {
        // Convert from two's complement negative to unsigned
        val += 2_i64.pow(62);
        val += 2_i64.pow(62);
        unsigned = val as u64 + 2_u64.pow(63);
    } else {
        // Signed int is positive, convert straight to unsigned
        unsigned = val as u64;
    }
    
    // Serialize accounting for endianness
    let mut res: [u8; 8] = [0; 8];
    write_u64(&mut res, unsigned);
    res
}