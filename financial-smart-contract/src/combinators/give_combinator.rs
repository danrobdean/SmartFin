use super::contract_combinator::{ ContractCombinator, Box, Vec };

// The give combinator
pub struct GiveCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>
}

// Method implementation for the give combinator
impl GiveCombinator {
    pub fn new(sub_combinator: Box<ContractCombinator>) -> GiveCombinator {
        GiveCombinator {
            sub_combinator
        }
    }
}

// Contract combinator implementation for the give combinator
impl ContractCombinator for GiveCombinator {
    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64 {
        -1 * self.sub_combinator.get_value(time, or_choices, obs_values)
    }
}