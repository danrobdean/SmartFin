use super::contract_combinator::{ ContractCombinator, Vec };

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
    fn get_value(&self, _time: u32, _or_choices: &Vec<Option<bool>>, _obs_values: &Vec<Option<i64>>) -> i64 {
        0
    }
}