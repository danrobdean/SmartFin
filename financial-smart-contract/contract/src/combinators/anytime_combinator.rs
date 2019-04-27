use super::contract_combinator::{ ContractCombinator, CombinatorDetails, Box, Vec };

// The anytime combinator
pub struct AnytimeCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>,

    // The anytime-index
    anytime_index: usize,

    // The common combinator details
    combinator_details: CombinatorDetails
}

// Method implementation for the anytime combinator
impl AnytimeCombinator {
    pub fn new(sub_combinator: Box<ContractCombinator>, anytime_index: usize) -> AnytimeCombinator {
        AnytimeCombinator {
            sub_combinator,
            anytime_index,
            combinator_details: CombinatorDetails::new()
        }
    }
}

// Contract combinator implementation for the anytime combinator
impl ContractCombinator for AnytimeCombinator {
    // Returns the sub-horizon
    fn get_horizon(&self) -> Option<u32> {
        self.sub_combinator.get_horizon()
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<Option<u32>>) -> i64 {
        let mut acquisition_time = anytime_acquisition_times[self.anytime_index];

        // If no acquisition time set, or acquisition time set after the sub-combinator's horizon
        if acquisition_time == None || self.sub_combinator.past_horizon(acquisition_time.unwrap()) {
            acquisition_time = self.sub_combinator.get_horizon();
        }

        // If current concrete acquisition time is None or not reached yet, value is 0, otherwise value of sub-combinator
        if acquisition_time == None || time < acquisition_time.unwrap() {
            0
        } else {
            self.sub_combinator.get_value(acquisition_time.unwrap(), or_choices, obs_values, anytime_acquisition_times)
        }
    }

    fn get_combinator_details(&self) -> &CombinatorDetails {
        &self.combinator_details
    }

    // Acquires the combinator and acquirable sub-combinators
    fn acquire(&mut self, time: u32, _or_choices: &Vec<Option<bool>>, anytime_acquisition_times: &mut Vec<Option<u32>>) {
        if self.past_horizon(time) {
            panic!("Cannot acquire an expired contract.");
        }
        if self.combinator_details.acquisition_time != None {
            panic!("Acquiring a previously-acquired anytime combinator is not allowed.");
        }

        anytime_acquisition_times[self.anytime_index] = self.sub_combinator.get_horizon();
        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, setting the acquisition time, and returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &mut Vec<Option<u32>>) -> i64 {
        // If not acquired yet or fully updated (no more pending balance), return 0
        if self.combinator_details.acquisition_time == None
            || self.combinator_details.acquisition_time.unwrap() > time
            || self.combinator_details.fully_updated {
            return 0;
        }

        // Check if sub-combinator acquisition time already set
        let mut acquisition_time = self.sub_combinator.get_combinator_details().acquisition_time;

        // If not, check if provided
        if acquisition_time == None && anytime_acquisition_times[self.anytime_index] != None {
            acquisition_time = anytime_acquisition_times[self.anytime_index];

            // If sub-horizon is before the given acquisition time, use sub-horizon as acquisition time
            if self.sub_combinator.past_horizon(acquisition_time.unwrap()) {
                acquisition_time = self.sub_combinator.get_horizon();
            }

            if self.combinator_details.acquisition_time.unwrap() > acquisition_time.unwrap() {
                panic!("Cannot acquire anytime sub-combinator before the anytime combinator is acquired.");
            }

            // If acquisition time has been passed then acquire sub-contract, otherwise do nothing more
            if acquisition_time.unwrap() <= time {
                self.sub_combinator.acquire(acquisition_time.unwrap(), or_choices, anytime_acquisition_times);
            }
        }

        let sub_value = self.sub_combinator.update(time, or_choices, obs_values, anytime_acquisition_times);
        self.combinator_details.fully_updated = self.sub_combinator.get_combinator_details().fully_updated;
        sub_value
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, AnytimeCombinator, OneCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, vec };
    
    // Value is sub-combinator's value
    #[test]
    fn correct_value_after_acquisition_time() {
        // Create combinator anytime truncate 1 one
        let combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Check value = 1
        let value = combinator.get_value(2, &vec![], &vec![], &vec![Some(0)]);
        assert_eq!(
            value,
            1,
            "Value of 'anytime truncate 1 one' at time = 2 with acquisition time 0 is not equal to 1: {}",
            value
        );
    }
    
    // Value is 0 before horizon
    #[test]
    fn correct_value_before_acquisition_time() {
        // Create combinator anytime truncate 1 one
        let combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Check value = 0
        let value = combinator.get_value(0, &vec![], &vec![], &vec![Some(1)]);
        assert_eq!(
            value,
            0,
            "Value of 'anytime truncate 1 one' with acquisition time 1 at time = 0 is not equal to 0: {}",
            value
        );
    }
    
    // Value is 1 after horizon with no acquisition time
    #[test]
    fn correct_value_after_horizon_no_acquisition_time() {
        // Create combinator anytime truncate 1 one
        let combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Check value = 0
        let value = combinator.get_value(2, &vec![], &vec![], &vec![None]);
        assert_eq!(
            value,
            1,
            "Value of 'anytime truncate 1 one' with no acquisition time at time = 2 is not equal to 1: {}",
            value
        );
    }
    
    // Value is 0 if the sub-combinator has no horizon or acquisition time
    #[test]
    fn correct_value_no_horizon_no_acquisition_time() {
        // Create combinator anytime one
        let combinator = AnytimeCombinator::new(
            Box::from(OneCombinator::new()),
            0
        );

        // Check value = 0
        let value = combinator.get_value(0, &vec![], &vec![], &vec![None]);
        assert_eq!(
            value,
            0,
            "Value of 'anytime one' at time = 0 is not equal to 0: {}",
            value
        );
    }

    // Horizon is equal to sub-combinator's horizon
    #[test]
    fn horizon_equals_sub_combinator_horizon() {
        // Create combinator anytime truncate 1 one
        let combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Check horizon
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            Some(1),
            "Horizon of combinator 'anytime truncate 1 one' is not equal to Some(1): {:?}",
            horizon
        );
    }

    // Acquiring combinator sets combinator details correctly
    #[test]
    fn acquiring_sets_combinator_details() {
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Acquire and check details
        let time: u32 = 1;
        let mut acquisition_times = vec![None];
        combinator.acquire(time, &vec![], &mut acquisition_times);
        let combinator_details = combinator.get_combinator_details();

        assert_eq!(
            combinator_details.acquisition_time,
            Some(time),
            "Acquisition time of combinator is not equal to Some(1): {:?}",
            combinator_details.acquisition_time
        );
    }

    // Acquiring and updating combinator returns correct value before the horizon
    #[test]
    fn acquiring_and_updating_returns_correct_value_before_acquisition_time() {
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        let value = combinator.update(0, &vec![], &vec![], &mut acquisition_times);

        assert_eq!(
            value,
            0,
            "Update value of anytime truncate 1 one with no acquisition time at time = 0 is not equal to 0: {}",
            value
        );
    }

    // Acquiring and updating combinator returns correct value after the horizon
    #[test]
    fn acquiring_and_updating_returns_correct_value_after_horizon() {
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        assert_eq!(acquisition_times[0], Some(1));
        let value = combinator.update(2, &vec![], &vec![], &mut acquisition_times);

        assert_eq!(
            value,
            1,
            "Update value of anytime truncate 1 one with no acquisition time at time = 2 is not equal to 1: {}",
            value
        );
    }

    // Acquiring and updating combinator returns correct value after the horizon but before the given acquisition time
    #[test]
    fn acquiring_and_updating_returns_correct_value_after_horizon_before_acquisition_time() {
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        acquisition_times[0] = Some(3);
        let value = combinator.update(2, &vec![], &vec![], &mut acquisition_times);

        assert_eq!(
            value,
            1,
            "Update value of anytime truncate 1 one with no acquisition time at time = 2 with acquisition time 3 is not equal to 1: {}",
            value
        );
    }

    // Acquiring and updating combinator before acquisition time sets fully updated to false
    #[test]
    fn acquiring_and_updating_does_not_set_fully_updated_before_acquisition_time() {
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        acquisition_times[0] = Some(1);
        combinator.update(0, &vec![], &vec![], &mut acquisition_times);
        let fully_updated = combinator.get_combinator_details().fully_updated;

        assert!(
            !fully_updated,
            "fully_updated is not false: {}",
            fully_updated
        );
    }

    // Acquiring and updating combinator after acquisition time sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated_after_acquisition_time() {
        // Create combinator anytime truncate 2 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                2
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        acquisition_times[0] = Some(0);
        combinator.update(1, &vec![], &vec![], &mut acquisition_times);
        let fully_updated = combinator.get_combinator_details().fully_updated;

        assert!(
            fully_updated,
            "fully_updated is not true: {}",
            fully_updated
        );
    }

    // Acquiring and updating combinator after horizon with no given acquisition time sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated_after_horizon_no_acquisition_time() {
        // Create combinator anytime truncate 2 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                2
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        combinator.update(3, &vec![], &vec![], &mut acquisition_times);
        let fully_updated = combinator.get_combinator_details().fully_updated;

        assert!(
            fully_updated,
            "fully_updated is not true: {}",
            fully_updated
        );
    }

    // Acquiring and updating combinator after acquisition time returns correct value
    #[test]
    fn acquiring_and_updating_after_acquisition_time_returns_correct_value() {
        // Create combinator anytime truncate 2 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                2
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        acquisition_times[0] = Some(1);
        let value = combinator.update(1, &vec![], &vec![], &mut acquisition_times);

        assert_eq!(
            value,
            1,
            "Update value of anytime truncate 2 one at time == acquisition time is not equal to 1: {}",
            value
        );
    }

    // Acquiring and updating combinator twice returns correct value
    #[test]
    fn acquiring_and_updating_twice_after_acquisition_time_returns_correct_value() {
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        combinator.update(2, &vec![], &vec![], &mut acquisition_times);
        let value = combinator.update(2, &vec![], &vec![], &mut acquisition_times);

        assert_eq!(
            value,
            0,
            "Second update value of anytime truncate 1 one at time = 2 is not equal to 0: {}",
            value
        );
    }

    // Acquiring and updating combinator twice returns correct value with first update before acquisition time
    #[test]
    fn acquiring_and_updating_twice_with_one_update_before_horizon_returns_correct_value() {
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        combinator.update(0, &vec![], &vec![], &mut acquisition_times);
        let value = combinator.update(2, &vec![], &vec![], &mut acquisition_times);

        assert_eq!(
            value,
            1,
            "Second update value of anytime truncate 1 one after horizon is not equal to 1: {}",
            value
        );
    }

    // Updating before acquiring does not set fully updated, and returns correct value
    #[test]
    fn updating_before_acquiring_does_nothing() {
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Update check details
        let mut acquisition_times = vec![None];
        let value = combinator.update(2, &vec![], &vec![], &mut acquisition_times);
        let combinator_details = combinator.get_combinator_details();

        assert!(
            !combinator_details.fully_updated,
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
        // Create combinator anytime truncate 1 one
        let mut combinator = AnytimeCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            )),
            0
        );

        // Update check details
        let mut acquisition_times = vec![None];
        combinator.acquire(1, &vec![], &mut acquisition_times);
        let value = combinator.update(0, &vec![], &vec![], &mut acquisition_times);
        let combinator_details = combinator.get_combinator_details();

        assert!(
            !combinator_details.fully_updated,
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

    // Updating without horizon/acquisition time does not set fully updated and returns correct value
    #[test]
    fn acquiring_and_updating_with_no_horizon_or_acquisition_time_does_nothing() {
        // Create combinator anytime one
        let mut combinator = AnytimeCombinator::new(
            Box::from(OneCombinator::new()),
            0
        );

        // Update check details
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        let value = combinator.update(10, &vec![], &vec![], &mut acquisition_times);
        let combinator_details = combinator.get_combinator_details();

        assert!(
            !combinator_details.fully_updated,
            "fully_updated != false: {}",
            combinator_details.fully_updated
        );

        assert_eq!(
            value,
            0,
            "Value of updating with no horizon != 0: {}",
            value
        )
    }

    // Acquiring combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired anytime combinator is not allowed.")]
    fn should_panic_when_acquiring_combinator_twice() {
        // Create combinator
        let mut combinator = AnytimeCombinator::new(Box::new(OneCombinator::new()), 0);

        // Acquire twice
        let mut acquisition_times = vec![None];
        combinator.acquire(0, &vec![], &mut acquisition_times);
        combinator.acquire(0, &vec![], &mut acquisition_times);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Cannot acquire an expired contract.")]
    fn should_panic_when_acquiring_post_expiry() {
        // Create combinator
        let mut combinator = AnytimeCombinator::new(
            Box::new(TruncateCombinator::new(
                Box::new(OneCombinator::new()),
                0
            )),
            0
        );

        // Acquire at time = 1
        let mut acquisition_times = vec![None];
        combinator.acquire(1, &vec![], &mut acquisition_times);
    }

    // Acquiring and updating combinator with invalid acquisition time is not allowed
    #[test]
    #[should_panic(expected = "Cannot acquire anytime sub-combinator before the anytime combinator is acquired.")]
    fn should_panic_when_acquiring_and_updating_with_acquisition_time_before_self_acquisition() {
        // Create combinator anytime one
        let mut combinator = AnytimeCombinator::new(
            Box::from(OneCombinator::new()),
            0
        );

        // Acquire and check value
        let mut acquisition_times = vec![None];
        combinator.acquire(1, &vec![], &mut acquisition_times);
        acquisition_times[0] = Some(0);
        combinator.update(2, &vec![], &vec![], &mut acquisition_times);
    }
}