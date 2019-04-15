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

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, ThenCombinator, TruncateCombinator, ZeroCombinator, OneCombinator };
    use super::super::contract_combinator::{ Box, vec };
    
    // Value with left sub-combinator non-expired is correct
    #[test]
    fn correct_value_left_sub_combinator_non_expired() {
        // Create combinator then zero one
        let combinator = ThenCombinator::new(Box::from(ZeroCombinator::new()), Box::from(OneCombinator::new()));

        // Check value = 0
        let value = combinator.get_value(0, &vec![], &vec![]);
        assert_eq!(
            value,
            0,
            "Value of 'then zero one' contract is not equal to 0: {}",
            value
        );
    }
    
    // Value with left sub-combinator expired is correct
    #[test]
    fn correct_value_left_sub_combinator_expired() {
        // Create combinator then truncate 0 zero one
        let combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                0,
            )),
            Box::from(OneCombinator::new())
        );

        // Check value = 1 at time = 1
        let value = combinator.get_value(1, &vec![], &vec![]);
        assert_eq!(
            value,
            1,
            "Value of 'then truncate 0 zero one' contract at time = 1 is not equal to 1: {}",
            value
        );
    }

    // Test that contract horizon is correct
    #[test]
    fn correct_horizon() {
        // Create combinator then truncate 1 zero truncate 2 one
        let combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                2
            ))
        );

        // Check horizon = 2
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            Some(2),
            "Horizon of 'then truncate 1 zero truncate 2 one' is not equal to Some(2): {:?}",
            horizon
        );
    }
}