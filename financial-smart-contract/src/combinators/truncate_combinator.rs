use super::contract_combinator::{ ContractCombinator, earliest_horizon, Box, Vec };

// The truncate combinator
pub struct TruncateCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>,
    // The truncated horizon
    truncated_horizon: u32
}

// Method implementation for the truncate combinator
impl TruncateCombinator {
    pub fn new(sub_combinator: Box<ContractCombinator>, truncated_horizon: u32) -> TruncateCombinator {
        TruncateCombinator {
            sub_combinator,
            truncated_horizon
        }
    }
}

// Contract combinator implementation for the truncate combinator
impl ContractCombinator for TruncateCombinator {
    // Returns the latest of the sub-horizon and the truncated horizon
    fn get_horizon(&self) -> Option<u32> {
        earliest_horizon(self.sub_combinator.get_horizon(), Some(self.truncated_horizon))
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64 {
        if self.past_horizon(time) {
            0
        } else {
            self.sub_combinator.get_value(time, or_choices, obs_values)
        }
    }
}