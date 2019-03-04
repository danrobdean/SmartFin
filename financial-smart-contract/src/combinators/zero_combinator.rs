use super::contract_combinator::ContractCombinator;

// The zero combinator
pub struct ZeroCombinator {}

// Method implementation of the zero combinator
impl ZeroCombinator {
    // Constructor
    pub fn new() -> ZeroCombinator {
        ZeroCombinator {}
    }
}

// Contract combinator implementation of the zero combinator
impl ContractCombinator for ZeroCombinator {
    fn acquire(&self, _time: i32) -> i32 {
        0
    }
}