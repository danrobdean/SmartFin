use super::contract_combinator::{ ContractCombinator, latest_horizon, Box, Vec };

// The or combinator
pub struct OrCombinator {
    // The first sub-combinator
    sub_combinator0: Box<ContractCombinator>,
    // The second sub-combinator
    sub_combinator1: Box<ContractCombinator>,
    // The index of this or combinator in the contract with reference to all or combinators
    or_index: usize
}

// Method implementation for the or combinator
impl OrCombinator {
    pub fn new(sub_combinator0: Box<ContractCombinator>, sub_combinator1: Box<ContractCombinator>, or_index: usize) -> OrCombinator {
        OrCombinator {
            sub_combinator0,
            sub_combinator1,
            or_index
        }
    }
}

// Contract combinator implementation for the or combinator
impl ContractCombinator for OrCombinator {
    // Returns the latest of the two sub-horizons
    fn get_horizon(&self) -> Option<u32> {
        latest_horizon(self.sub_combinator0.get_horizon(), self.sub_combinator1.get_horizon())
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64 {
        // If one sub-combinator has expired, choose the other
        if self.sub_combinator0.past_horizon(time) {
            self.sub_combinator1.get_value(time, or_choices, obs_values)
        } else if self.sub_combinator1.past_horizon(time) {
            self.sub_combinator0.get_value(time, or_choices, obs_values)
        } else {
            // If both sub-combinators can be acquired, use the provided choice, or panic if no choice has been provided
            match or_choices[self.or_index] {
                Some(true) => self.sub_combinator0.get_value(time, or_choices, obs_values),
                Some(false) => self.sub_combinator1.get_value(time, or_choices, obs_values),
                None => panic!("Cannot get value of OR combinator when neither sub-combinator has been chosen or has expired.")
            }
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, OrCombinator, OneCombinator, ZeroCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, vec };
    
    // Value is value of left sub-combinator when left or-choice is made and no sub-combinators have expired
    #[test]
    fn correct_value_left_no_expiry() {
        // Create combinator or zero one
        let combinator = OrCombinator::new(Box::from(ZeroCombinator::new()), Box::from(OneCombinator::new()), 0);

        // Check value = 0
        let value = combinator.get_value(0, &vec![Some(true)], &vec![]);
        assert_eq!(
            value,
            0,
            "Value of 'or zero one' contract with or-choice 0 = left is not equal to 0: {}",
            value
        );
    }
    
    // Value is value of right sub-combinator when right or-choice is made and no sub-combinators have expired
    #[test]
    fn correct_value_right_no_expiry() {
        // Create combinator or zero one
        let combinator = OrCombinator::new(Box::from(ZeroCombinator::new()), Box::from(OneCombinator::new()), 0);

        // Check value = 0
        let value = combinator.get_value(0, &vec![Some(false)], &vec![]);
        assert_eq!(
            value,
            1,
            "Value of 'or zero one' contract with or-choice 0 = right is not equal to 1: {}",
            value
        );
    }
    
    // Value is value of right sub-combinator when left or-choice is made and left sub-combinator is expired
    #[test]
    fn correct_value_left_expired() {
        // Create combinator or truncate 1 zero one
        let combinator = OrCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new()),
            0
        );

        // Check value = 1 at time = 2
        let value = combinator.get_value(2, &vec![Some(true)], &vec![]);
        assert_eq!(
            value,
            1,
            "Value of 'or truncate zero one' contract with expired left combinator is not 1: {}",
            value
        );
    }
    
    // Value is value of left sub-combinator when right or-choice is made and right sub-combinator is expired
    #[test]
    fn correct_value_right_expired() {
        // Create combinator or one truncate 1 zero
        let combinator = OrCombinator::new(
            Box::from(OneCombinator::new()),
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            0
        );

        // Check value = 1 at time = 2
        let value = combinator.get_value(2, &vec![Some(true)], &vec![]);
        assert_eq!(
            value,
            1,
            "Value of 'or one truncate zero' contract with expired right combinator is not 1: {}",
            value
        );
    }
    
    // Horizon is latest of sub-combinators' horizons with the left combinator truncated
    #[test]
    fn correct_horizon_with_left_combinator_truncated() {
        // Create combinator or truncate 1 one one
        let combinator = OrCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new()),
            0
        );

        // Check horizon == None
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            None,
            "Value of 'or truncate 1 one one' contract is not equal to None: {:?}",
            horizon
        );
    }
    
    // Horizon is latest of sub-combinators' horizons with the right combinator truncated
    #[test]
    fn correct_horizon_with_right_combinator_truncated() {
        // Create combinator or one truncate 1 one
        let combinator = OrCombinator::new(
            Box::from(OneCombinator::new()),
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Check horizon == None
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            None,
            "Value of 'or one truncate 1 one' contract is not equal to None: {:?}",
            horizon
        );
    }

    // Getting value if bot sub-combinators non-expired and or-choice not made is not allowed
    #[test]
    #[should_panic(expected = "Cannot get value of OR combinator when neither sub-combinator has been chosen or has expired.")]
    fn should_panic_if_getting_value_with_both_sub_combinators_non_expired_and_no_or_choice() {
        // Create combinator or one one
        let combinator = OrCombinator::new(
            Box::from(OneCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Get value at time = 0 with no or-choice
        combinator.get_value(2, &vec![None], &vec![]);
    }
}