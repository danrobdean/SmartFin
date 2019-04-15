use super::contract_combinator::{ ContractCombinator, latest_horizon, Box, Vec };

// The then combinator
pub struct ThenCombinator {
    // The first sub-combinator
    sub_combinator0: Box<ContractCombinator>,
    // The second sub-combinator
    sub_combinator1: Box<ContractCombinator>
}

// Method implementation for the then combinator
impl ThenCombinator {
    pub fn new(sub_combinator0: Box<ContractCombinator>, sub_combinator1: Box<ContractCombinator>) -> ThenCombinator {
        ThenCombinator {
            sub_combinator0,
            sub_combinator1
        }
    }
}

// Contract combinator implementation for the then combinator
impl ContractCombinator for ThenCombinator {
    // Returns the latest of the two sub-horizons
    fn get_horizon(&self) -> Option<u32> {
        latest_horizon(self.sub_combinator0.get_horizon(), self.sub_combinator1.get_horizon())
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64 {
        if !self.sub_combinator0.past_horizon(time) {
            self.sub_combinator0.get_value(time, or_choices, obs_values)
        } else {
            self.sub_combinator1.get_value(time, or_choices, obs_values)
        }
    }
}