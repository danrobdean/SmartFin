use super::contract_combinator::{ ContractCombinator, Vec };

// The zero combinator
pub struct ZeroCombinator {}

// Method implementation of the zero combinator
impl ZeroCombinator {
    // Constructor
    pub fn new() -> ZeroCombinator {
        ZeroCombinator {}
    }
}

// Contract combinator implementation of the zero combinator
impl ContractCombinator for ZeroCombinator {
    fn get_value(&self, _time: u32, _or_choices: &Vec<Option<bool>>, _obs_values: &Vec<Option<i64>>) -> i64 {
        0
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, ZeroCombinator };
    use super::super::contract_combinator::{ vec };
    
    // Value is 0
    #[test]
    fn correct_value() {
        // Create combinator zero
        let combinator = ZeroCombinator::new();

        // Check value = 0
        let value = combinator.get_value(0, &vec![], &vec![]);
        assert_eq!(
            value,
            0,
            "Value of 'zero' contract is not equal to 0: {}",
            value
        );
    }

    // Horizon is None
    #[test]
    fn correct_horizon() {
        // Create combinator zero
        let combinator = ZeroCombinator::new();

        // Check horizon = None
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            None,
            "Value of 'zero' contract is not equal to None: {:?}",
            horizon
        );
    }
}