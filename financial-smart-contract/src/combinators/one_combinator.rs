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

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, OneCombinator };
    use super::super::contract_combinator::{ vec };
    
    // Value is 1
    #[test]
    fn correct_value() {
        // Create combinator one
        let combinator = OneCombinator::new();

        // Check value = 1
        let value = combinator.get_value(0, &vec![], &vec![]);
        assert_eq!(
            value,
            1,
            "Value of 'one' contract is not equal to 1: {}",
            value
        );
    }

    // Horizon is None
    #[test]
    fn correct_horizon() {
        // Create combinator one
        let combinator = OneCombinator::new();

        // Check horizon = None
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            None,
            "Value of 'one' contract is not equal to None: {:?}",
            horizon
        );
    }
}