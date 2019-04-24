use super::contract_combinator::{ ContractCombinator, CombinatorDetails, latest_time, Box, Vec };

// The or combinator
pub struct OrCombinator {
    // The first sub-combinator
    sub_combinator0: Box<ContractCombinator>,

    // The second sub-combinator
    sub_combinator1: Box<ContractCombinator>,

    // The index of this or combinator in the contract with reference to all or combinators
    or_index: usize,

    // The common combinator details
    combinator_details: CombinatorDetails
}

// Method implementation for the or combinator
impl OrCombinator {
    pub fn new(sub_combinator0: Box<ContractCombinator>, sub_combinator1: Box<ContractCombinator>, or_index: usize) -> OrCombinator {
        OrCombinator {
            sub_combinator0,
            sub_combinator1,
            or_index,
            combinator_details: CombinatorDetails::new()
        }
    }

    // Returns whether the current or-choice is the first sub-combinator
    fn get_or_choice(&self, time: u32, or_choices: &Vec<Option<bool>>) -> Option<bool> {
        if self.sub_combinator0.past_horizon(time) {
            Some(false)
        } else if self.sub_combinator1.past_horizon(time) {
            Some(true)
        } else {
            match or_choices[self.or_index] {
                Some(true) => Some(true),
                Some(false) => Some(false),
                None => None
            }
        }
    }
}

// Contract combinator implementation for the or combinator
impl ContractCombinator for OrCombinator {
    // Returns the latest of the two sub-horizons
    fn get_horizon(&self) -> Option<u32> {
        latest_time(self.sub_combinator0.get_horizon(), self.sub_combinator1.get_horizon())
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<Option<u32>>) -> i64 {
        if self.get_or_choice(time, or_choices) == None {
            panic!("Cannot get OR choice when neither sub-combinator has been chosen or has expired.");
        }

        if self.get_or_choice(time, or_choices).unwrap() {
            self.sub_combinator0.get_value(time, or_choices, obs_values, anytime_acquisition_times)
        } else {
            self.sub_combinator1.get_value(time, or_choices, obs_values, anytime_acquisition_times)
        }
    }

    fn get_combinator_details(&self) -> &CombinatorDetails {
        &self.combinator_details
    }

    // Acquires the combinator and acquirable sub-combinators
    fn acquire(&mut self, time: u32, or_choices: &Vec<Option<bool>>, anytime_acquisition_times: &mut Vec<Option<u32>>) {
        if self.past_horizon(time) {
            panic!("Acquiring an expired contract is not allowed.");
        }
        if self.combinator_details.acquisition_time != None {
            panic!("Acquiring a previously-acquired or combinator is not allowed.");
        }

        // Check which sub-combinator to acquire. If ambiguous, acquire both branches.
        if self.get_or_choice(time, or_choices) == Some(true) {
            self.sub_combinator0.acquire(time, or_choices, anytime_acquisition_times);
        } else if self.get_or_choice(time, or_choices) == Some(false) {
            self.sub_combinator1.acquire(time, or_choices, anytime_acquisition_times);
        } else {
            self.sub_combinator0.acquire(time, or_choices, anytime_acquisition_times);
            self.sub_combinator1.acquire(time, or_choices, anytime_acquisition_times);
        }

        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &mut Vec<Option<u32>>) -> i64 {
        let or_choice = self.get_or_choice(time, or_choices);
        // If not acquired yet or fully updated (no more pending balance), return 0
        if self.combinator_details.acquisition_time == None
            || self.combinator_details.acquisition_time.unwrap() > time
            || self.combinator_details.fully_updated
            // If ambiguous or choice, don't update
            || or_choice == None {
            return 0;
        }

        let sub_combinator;
        if or_choice.unwrap() {
            sub_combinator = &mut self.sub_combinator0;
        } else {
            sub_combinator = &mut self.sub_combinator1;
        }
        let sub_value = sub_combinator.update(time, or_choices, obs_values, anytime_acquisition_times);
        self.combinator_details.fully_updated = sub_combinator.get_combinator_details().fully_updated;
        sub_value
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
        let value = combinator.get_value(0, &vec![Some(true)], &vec![], &vec![]);
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
        let value = combinator.get_value(0, &vec![Some(false)], &vec![], &vec![]);
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
        let value = combinator.get_value(2, &vec![Some(true)], &vec![], &vec![]);
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
        let value = combinator.get_value(2, &vec![Some(true)], &vec![], &vec![]);
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

    // Acquiring give-combinator sets combinator details correctly
    #[test]
    fn acquiring_sets_combinator_details() {
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(ZeroCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Acquire and check details
        let time: u32 = 5;
        combinator.acquire(time, &vec![Some(true)], &mut vec![]);
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
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(ZeroCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Acquire and check value
        combinator.acquire(0, &vec![Some(false)], &mut vec![]);
        let value = combinator.update(0, &vec![Some(false)], &vec![], &mut vec![]);

        assert_eq!(
            value,
            1,
            "Update value of or zero one is not equal to 1 with right or-choice: {}",
            value
        );
    }

    // Acquiring and updating combinator sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated() {
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(ZeroCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Acquire and check value
        combinator.acquire(0, &vec![Some(false)], &mut vec![]);
        combinator.update(0, &vec![Some(false)], &vec![], &mut vec![]);
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
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(ZeroCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Acquire and check value
        combinator.acquire(0, &vec![Some(false)], &mut vec![]);
        combinator.update(0, &vec![Some(false)], &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![Some(false)], &vec![], &mut vec![]);

        assert_eq!(
            value,
            0,
            "Second update value of or zero one is not equal to 0: {}",
            value
        );
    }

    // Updating before acquiring does not set fully updated, and returns correct value
    #[test]
    fn updating_before_acquiring_does_nothing() {
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(ZeroCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Update check details
        let value = combinator.update(0, &vec![Some(false)], &vec![], &mut vec![]);
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
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(ZeroCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Update check details
        combinator.acquire(1, &vec![Some(false)], &mut vec![]);
        let value = combinator.update(0, &vec![Some(false)], &vec![], &mut vec![]);
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

    // Updating with ambiguous or choice does not set fully updated and returns correct value
    #[test]
    fn updating_ambiguous_or_choice_does_nothing() {
        // Create combinator or one one
        let mut combinator = OrCombinator::new(
            Box::from(OneCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Update check details
        combinator.acquire(0, &vec![None], &mut vec![]);
        let value = combinator.update(0, &vec![None], &vec![], &mut vec![]);
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
            "Value of updating ambiguous or choice != 0: {}",
            value
        )
    }

    // Updating before acquisition time does not set fully updated and returns correct value
    #[test]
    fn updating_with_different_time_to_acquisition_time_returns_correct_value() {
        // Create combinator or truncate 1 zero one
        let mut combinator = OrCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new()),
            0
        );

        // Update check details
        combinator.acquire(0, &vec![Some(true)], &mut vec![]);
        let value = combinator.update(2, &vec![Some(true)], &vec![], &mut vec![]);

        assert_eq!(
            value,
            0,
            "Value of updating later than acquiring != 0: {}",
            value
        )
    }

    // Getting value if bot sub-combinators non-expired and or-choice not made is not allowed
    #[test]
    #[should_panic(expected = "Cannot get OR choice when neither sub-combinator has been chosen or has expired.")]
    fn should_panic_if_getting_value_with_both_sub_combinators_non_expired_and_no_or_choice() {
        // Create combinator or one one
        let combinator = OrCombinator::new(
            Box::from(OneCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Get value at time = 0 with no or-choice
        combinator.get_value(2, &vec![None], &vec![], &vec![]);
    }

    // Acquiring combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired or combinator is not allowed.")]
    fn should_panic_when_acquiring_combinator_twice() {
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(ZeroCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Acquire twice
        combinator.acquire(0, &vec![Some(false)], &mut vec![]);
        combinator.acquire(0, &vec![Some(false)], &mut vec![]);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Acquiring an expired contract is not allowed.")]
    fn should_panic_when_acquiring_post_expiry() {
        // Create combinator or truncate 0 zero truncate 0 one
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::new(ZeroCombinator::new()),
                0
            )),
            Box::from(TruncateCombinator::new(
                Box::new(OneCombinator::new()),
                0
            )),
            0
        );

        // Acquire at time = 1
        combinator.acquire(1, &vec![], &mut vec![]);
    }
}