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

    fn get_horizon(&self) -> Option<u32> {
        self.sub_combinator.get_horizon()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, GiveCombinator, OneCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, vec };
    
    // Value is negation of sub-combinator's value
    #[test]
    fn correct_value() {
        // Create combinator give one
        let combinator = GiveCombinator::new(Box::from(OneCombinator::new()));

        // Check value = -1
        let value = combinator.get_value(0, &vec![], &vec![]);
        assert_eq!(
            value,
            -1,
            "Value of 'give one' contract is not equal to -1: {}",
            value
        );
    }

    // Horizon is equal to sub-combinator's horizon
    #[test]
    fn horizon_equals_sub_combinator_horizon() {
        // Create combinator give truncate 1 one
        let combinator = GiveCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Check horizon
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            Some(1),
            "Horizon of combinator 'give truncate 1 one' is not equal to Some(1): {:?}",
            horizon
        );
    }
}