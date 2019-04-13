use super::contract_combinator::{ ContractCombinator, Vec };

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
    fn get_value(&self, _time: u32, _or_choices: &Vec<Option<bool>>, _obs_values: &Vec<Option<i64>>) -> i64 {
        1
    }
}