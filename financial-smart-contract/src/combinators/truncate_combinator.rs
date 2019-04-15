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

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, OneCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, vec };
    
    // Value before expiry is equal to the value of the sub-combinator
    #[test]
    fn correct_value_pre_expiry() {
        // Create truncate 1 one
        let combinator = TruncateCombinator::new(Box::from(OneCombinator::new()), 1);

        // Check value = 1
        let value = combinator.get_value(0, &vec![], &vec![]);
        assert_eq!(
            value,
            1,
            "Value of 'truncate 1 one' contract at time = 0 is not equal to 1: {}",
            value
        );
    }
    
    // Value after expiry is 0
    #[test]
    fn correct_value_post_expiry() {
        // Create truncate 1 one
        let combinator = TruncateCombinator::new(Box::from(OneCombinator::new()), 1);

        // Check value = 0
        let value = combinator.get_value(2, &vec![], &vec![]);
        assert_eq!(
            value,
            0,
            "Value of 'truncate 1 one' contract at time = 2 is not equal to 0: {}",
            value
        );
    }
    
    // Horizon is correct
    #[test]
    fn correct_horizon() {
        // Create truncate 5 one
        let combinator = TruncateCombinator::new(Box::from(OneCombinator::new()), 5);

        // Check horizon = 5
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            Some(5),
            "Horizon of 'truncate 5 one' contract is not equal to Some(5): {:?}",
            horizon
        );
    }
    
    // Horizon is correct if sub-combinator expires first
    #[test]
    fn correct_horizon_sub_combinator_expires_first() {
        // Create truncate 5 truncate 4 one
        let combinator = TruncateCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                4
            )),
        5);

        // Check horizon = 4
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            Some(4),
            "Horizon of 'truncate 5 truncate 4 one' contract is not equal to Some(4): {:?}",
            horizon
        );
    }
}