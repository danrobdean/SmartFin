use super::contract_combinator::{ ContractCombinator, Box, Vec };

// The scale combinator
pub struct ScaleCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>,
    // The observable index
    obs_index: Option<usize>,
    // The scale value
    scale_value: Option<i64>
}

// Method implementation for the scale combinator
impl ScaleCombinator {
    pub fn new(sub_combinator: Box<ContractCombinator>, obs_index: Option<usize>, scale_value: Option<i64>) -> ScaleCombinator {
        if obs_index == None && scale_value == None {
            panic!("Scale combinator cannot be instantiated without a concrete observable index or scale value.")
        }

        ScaleCombinator {
            sub_combinator,
            obs_index,
            scale_value
        }
    }
}

// Contract combinator implementation for the scale combinator
impl ContractCombinator for ScaleCombinator {
    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64 {
        match self.scale_value {
            Some(value) => value* self.sub_combinator.get_value(time, or_choices, obs_values),
            None => {
                match self.obs_index {
                    Some(index) => {
                        if index >= obs_values.len() {
                            panic!("Attempted to lookup observable which does not exist.")
                        }
                        let obs_value = obs_values[index];
                        match obs_value {
                            Some(value) => value * self.sub_combinator.get_value(time, or_choices, obs_values),
                            None => panic!("Cannot get value of scale combinator with an undefined observable.")
                        }
                    },
                    None => panic!("Scale combinator has no scale value or observable index.")
                }
            }
        }
    }

    fn get_horizon(&self) -> Option<u32> {
        self.sub_combinator.get_horizon()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, ScaleCombinator, OneCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, vec };
    
    // Value with provided positive scale value is correct
    #[test]
    fn correct_value_scale_value_positive() {
        // Create combinator scale 5 one
        let combinator = ScaleCombinator::new(Box::from(OneCombinator::new()), None, Some(5));

        // Check value = 5
        let value = combinator.get_value(0, &vec![], &vec![]);
        assert_eq!(
            value,
            5,
            "Value of 'scale 5 one' contract is not equal to 5: {}",
            value
        );
    }
    
    // Value with provided negative scale value is correct
    #[test]
    fn correct_value_scale_value_negative() {
        // Create combinator scale -5 one
        let combinator = ScaleCombinator::new(Box::from(OneCombinator::new()), None, Some(-5));

        // Check value = -5
        let value = combinator.get_value(0, &vec![], &vec![]);
        assert_eq!(
            value,
            -5,
            "Value of 'scale -5 one' contract is not equal to -5: {}",
            value
        );
    }
    
    // Value with observable value is correct
    #[test]
    fn correct_value_observable() {
        // Create combinator scale obs one
        let combinator = ScaleCombinator::new(Box::from(OneCombinator::new()), Some(0), None);

        // Check value = 5
        let value = combinator.get_value(0, &vec![], &vec![Some(5)]);
        assert_eq!(
            value,
            5,
            "Value of 'scale obs one' contract with observable 5 is not equal to 5: {}",
            value
        );
    }

    // Horizon is equal to sub-combinator's horizon
    #[test]
    fn horizon_equals_sub_combinator_horizon() {
        // Create combinator scale 1 truncate 1 one
        let combinator = ScaleCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            None,
            Some(1)
        );

        // Check horizon
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            Some(1),
            "Horizon of combinator 'scale 1 truncate 1 one' is not equal to Some(1): {:?}",
            horizon
        );
    }
    
    // Scale combinator being instantiated without an observable index or scale value is not allowed
    #[test]
    #[should_panic(expected = "Scale combinator cannot be instantiated without a concrete observable index or scale value.")]
    fn should_panic_if_instantiated_without_obs_index_or_scale_value() {
        // Create combinator scale <> one
        ScaleCombinator::new(Box::from(OneCombinator::new()), None, None);
    }

    // Getting value without a concrete observable value is not allowed
    #[test]
    #[should_panic(expected = "Cannot get value of scale combinator with an undefined observable.")]
    fn should_panic_if_getting_value_without_concrete_observable_value() {
        // Create combinator scale obs one
        let combinator = ScaleCombinator::new(Box::from(OneCombinator::new()), Some(0), None);

        // Get value
        combinator.get_value(0, &vec![], &vec![None]);
    }

    // Getting value without the corresponding observable value is not allowed
    #[test]
    #[should_panic(expected = "Attempted to lookup observable which does not exist.")]
    fn should_panic_if_getting_value_without_observable_value_for_index() {
        // Create combinator scale obs one
        let combinator = ScaleCombinator::new(Box::from(OneCombinator::new()), Some(0), None);

        // Get value
        combinator.get_value(0, &vec![], &vec![]);
    }
}