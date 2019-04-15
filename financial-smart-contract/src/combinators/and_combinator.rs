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

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64 {
        self.sub_combinator0.get_value(time, or_choices, obs_values) + self.sub_combinator1.get_value(time, or_choices, obs_values)
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, AndCombinator, OneCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, vec };
    
    // Value is sum of sub-combinators' values
    #[test]
    fn and_combinator_correct_value() {
        // Create combinator and one one
        let combinator = AndCombinator::new(Box::from(OneCombinator::new()), Box::from(OneCombinator::new()));

        // Check value = 2
        let value = combinator.get_value(0, &vec![], &vec![]);
        assert_eq!(
            value,
            2,
            "Value of 'and one one' contract is not equal to 2: {}",
            value
        );
    }
    
    // Horizon is latest of sub-combinators' horizons with the left combinator truncated
    #[test]
    fn correct_horizon_with_left_combinator_truncated() {
        // Create combinator and truncate 1 one one
        let combinator = AndCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Check horizon == None
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            None,
            "Value of 'and truncate 1 one one' contract is not equal to None: {:?}",
            horizon
        );
    }
    
    // Horizon is latest of sub-combinators' horizons with the right combinator truncated
    #[test]
    fn correct_horizon_with_right_combinator_truncated() {
        // Create combinator and one truncate 1 one
        let combinator = AndCombinator::new(
            Box::from(OneCombinator::new()),
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Check horizon == None
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            None,
            "Value of 'and one truncate 1 one' contract is not equal to None: {:?}",
            horizon
        );
    }
}