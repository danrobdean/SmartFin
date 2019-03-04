use super::contract_combinator::ContractCombinator;

// The one combinator
pub struct OneCombinator {}

// Method implementation of the one combinator
impl OneCombinator {
    // Constructor
    pub fn new() -> OneCombinator {
        OneCombinator {}
    }
}

// Contract combinator implementation of the one combinator
impl ContractCombinator for OneCombinator {
    fn acquire(&self, _time: i32) -> i32 {
        1
    }
}