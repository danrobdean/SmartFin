use super::contract_combinator::{ ContractCombinator, latest_horizon, Box, Vec };

// The and combinator
pub struct AndCombinator {
    // The first sub-combinator
    sub_combinator0: Box<ContractCombinator>,
    // The second sub-combinator
    sub_combinator1: Box<ContractCombinator>
}

// Method implementation for the and combinator
impl AndCombinator {
    pub fn new(sub_combinator0: Box<ContractCombinator>, sub_combinator1: Box<ContractCombinator>) -> AndCombinator {
        AndCombinator {
            sub_combinator0,
            sub_combinator1
        }
    }
}

// Contract combinator implementation for the and combinator
impl ContractCombinator for AndCombinator {
    // Returns the latest of the two sub-horizons
    fn get_horizon(&self) -> Option<u32> {
        latest_horizon(self.sub_combinator0.get_horizon(), self.sub_combinator1.get_horizon())
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>) -> u64 {
        self.sub_combinator0.get_value(time, or_choices) + self.sub_combinator1.get_value(time, or_choices)
    }
}