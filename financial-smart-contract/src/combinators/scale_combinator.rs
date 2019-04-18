use super::contract_combinator::{ ContractCombinator, CombinatorDetails, Box, Vec };

// The scale combinator
pub struct ScaleCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>,

    // The observable index
    obs_index: Option<usize>,

    // The scale value
    scale_value: Option<i64>,

    // The common combinator details
    combinator_details: CombinatorDetails
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
            scale_value,
            combinator_details: CombinatorDetails::new()
        }
    }

    fn get_scale_value(&self, obs_values: &Vec<Option<i64>>) -> Option<i64> {
        match self.scale_value {
            Some(value) => Some(value),
            None => {
                match self.obs_index {
                    Some(index) => {
                        if index >= obs_values.len() {
                            panic!("Attempted to lookup observable which does not exist.")
                        }
                        obs_values[index]
                    },
                    None => panic!("Scale combinator has no scale value or observable index.")
                }
            }
        }
    }
}

// Contract combinator implementation for the scale combinator
impl ContractCombinator for ScaleCombinator {
    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<Option<u32>>) -> i64 {
        let scale_value = self.get_scale_value(obs_values);
        if scale_value == None {
            panic!("Cannot get value of an undefined observable.")
        }

        scale_value.unwrap() * self.sub_combinator.get_value(time, or_choices, obs_values, anytime_acquisition_times)
    }

    fn get_horizon(&self) -> Option<u32> {
        self.sub_combinator.get_horizon()
    }

    fn get_combinator_details(&self) -> &CombinatorDetails {
        &self.combinator_details
    }

    // Acquires the combinator and acquirable sub-combinators
    fn acquire(&mut self, time: u32, or_choices: &Vec<Option<bool>>) {
        if self.past_horizon(time) {
            panic!("Acquiring an expired contract is not allowed.");
        }
        if self.combinator_details.acquisition_time != None {
            panic!("Acquiring a previously-acquired scale combinator is not allowed.");
        }

        self.sub_combinator.acquire(time, or_choices);
        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<Option<u32>>) -> i64 {
        let scale_value = self.get_scale_value(obs_values);

        // If not acquired yet or fully updated (no more pending balance), return 0
        if self.combinator_details.acquisition_time == None
            || self.combinator_details.acquisition_time.unwrap() > time
            || self.combinator_details.fully_updated
            // If no scale value or obs value, don't update
            || scale_value == None {
            return 0;
        }

        let sub_value = self.sub_combinator.update(time, or_choices, obs_values, anytime_acquisition_times);
        self.combinator_details.fully_updated = self.sub_combinator.get_combinator_details().fully_updated;
        scale_value.unwrap() * sub_value
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
        let value = combinator.get_value(0, &vec![], &vec![], &vec![]);
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
        let value = combinator.get_value(0, &vec![], &vec![], &vec![]);
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
        let value = combinator.get_value(0, &vec![], &vec![Some(5)], &vec![]);
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

    // Acquiring give-combinator sets combinator details correctly
    #[test]
    fn acquiring_sets_combinator_details() {
        // Create combinator scale 5 one
        let mut combinator = ScaleCombinator::new(Box::new(OneCombinator::new()), None, Some(5));

        // Acquire and check details
        let time: u32 = 5;
        combinator.acquire(time, &vec![]);
        let combinator_details = combinator.get_combinator_details();

        assert_eq!(
            combinator_details.acquisition_time,
            Some(time),
            "Acquisition time of combinator is not equal to Some(5): {:?}",
            combinator_details.acquisition_time
        );
    }

    // Acquiring and updating combinator returns correct value
    #[test]
    fn acquiring_and_updating_returns_correct_value() {
        // Create combinator scale 5 one
        let mut combinator = ScaleCombinator::new(Box::new(OneCombinator::new()), None, Some(5));

        // Acquire and check value
        combinator.acquire(0, &vec![]);
        let value = combinator.update(0, &vec![], &vec![], &vec![]);

        assert_eq!(
            value,
            5,
            "Update value of scale 5 one is not equal to 5: {}",
            value
        );
    }

    // Acquiring and updating combinator sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated() {
        // Create combinator scale 5 one
        let mut combinator = ScaleCombinator::new(Box::new(OneCombinator::new()), None, Some(5));

        // Acquire and check value
        combinator.acquire(0, &vec![]);
        combinator.update(0, &vec![], &vec![], &vec![]);
        let fully_updated = combinator.get_combinator_details().fully_updated;

        assert_eq!(
            fully_updated,
            true,
            "fully_updated is not true: {}",
            fully_updated
        );
    }

    // Acquiring and updating combinator twice returns correct value
    #[test]
    fn acquiring_and_updating_twice_returns_correct_value() {
        // Create combinator scale 5 one
        let mut combinator = ScaleCombinator::new(Box::new(OneCombinator::new()), None, Some(5));

        // Acquire and check value
        combinator.acquire(0, &vec![]);
        combinator.update(0, &vec![], &vec![], &vec![]);
        let value = combinator.update(0, &vec![], &vec![], &vec![]);

        assert_eq!(
            value,
            0,
            "Second update value of scale 5 one is not equal to 0: {}",
            value
        );
    }

    // Updating before acquiring does not set fully updated, and returns correct value
    #[test]
    fn updating_before_acquiring_does_nothing() {
        // Create combinator scale 5 one
        let mut combinator = ScaleCombinator::new(Box::new(OneCombinator::new()), None, Some(5));

        // Update check details
        let value = combinator.update(0, &vec![], &vec![], &vec![]);
        let combinator_details = combinator.get_combinator_details();

        assert_eq!(
            combinator_details.fully_updated,
            false,
            "fully_updated != false: {}",
            combinator_details.fully_updated
        );

        assert_eq!(
            value,
            0,
            "Value of updating before acquiring != 0: {}",
            value
        )
    }

    // Updating before acquisition time does not set fully updated and returns correct value
    #[test]
    fn updating_before_acquisition_time_does_nothing() {
        // Create combinator scale 5 one
        let mut combinator = ScaleCombinator::new(Box::new(OneCombinator::new()), None, Some(5));

        // Update check details
        combinator.acquire(1, &vec![]);
        let value = combinator.update(0, &vec![], &vec![], &vec![]);
        let combinator_details = combinator.get_combinator_details();

        assert_eq!(
            combinator_details.fully_updated,
            false,
            "fully_updated != false: {}",
            combinator_details.fully_updated
        );

        assert_eq!(
            value,
            0,
            "Value of updating before acquiring != 0: {}",
            value
        )
    }

    // Updating without concrete observable value does not set fully updated and returns correct value
    #[test]
    fn updating_without_concrete_observable_does_nothing() {
        // Create combinator scale obs one
        let mut combinator = ScaleCombinator::new(Box::new(OneCombinator::new()), Some(0), None);

        // Update check details
        combinator.acquire(0, &vec![]);
        let value = combinator.update(0, &vec![], &vec![None], &vec![]);
        let combinator_details = combinator.get_combinator_details();

        assert_eq!(
            combinator_details.fully_updated,
            false,
            "fully_updated != false: {}",
            combinator_details.fully_updated
        );

        assert_eq!(
            value,
            0,
            "Value of updating without concrete observable value != 0: {}",
            value
        )
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
    #[should_panic(expected = "Cannot get value of an undefined observable.")]
    fn should_panic_if_getting_value_without_concrete_observable_value() {
        // Create combinator scale obs one
        let combinator = ScaleCombinator::new(Box::from(OneCombinator::new()), Some(0), None);

        // Get value
        combinator.get_value(0, &vec![], &vec![None], &vec![]);
    }

    // Getting value without the corresponding observable value is not allowed
    #[test]
    #[should_panic(expected = "Attempted to lookup observable which does not exist.")]
    fn should_panic_if_getting_value_without_observable_value_for_index() {
        // Create combinator scale obs one
        let combinator = ScaleCombinator::new(Box::from(OneCombinator::new()), Some(0), None);

        // Get value
        combinator.get_value(0, &vec![], &vec![], &vec![]);
    }

    // Acquiring combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired scale combinator is not allowed.")]
    fn should_panic_when_acquiring_combinator_twice() {
        // Create combinator
        let mut combinator = ScaleCombinator::new(Box::new(OneCombinator::new()), None, Some(5));

        // Acquire twice
        combinator.acquire(0, &vec![]);
        combinator.acquire(0, &vec![]);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Acquiring an expired contract is not allowed.")]
    fn should_panic_when_acquiring_post_expiry() {
        // Create combinator
        let mut combinator = ScaleCombinator::new(
            Box::new(TruncateCombinator::new(
                Box::new(OneCombinator::new()),
                0
            )),
            None,
            Some(5)
        );

        // Acquire at time = 1
        combinator.acquire(1, &vec![]);
    }
}