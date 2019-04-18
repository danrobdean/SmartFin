use super::contract_combinator::{ ContractCombinator, CombinatorDetails, latest_horizon, Box, Vec };

// The then combinator
pub struct ThenCombinator {
    // The first sub-combinator
    sub_combinator0: Box<ContractCombinator>,

    // The second sub-combinator
    sub_combinator1: Box<ContractCombinator>,

    // The common combinator details
    combinator_details: CombinatorDetails
}

// Method implementation for the then combinator
impl ThenCombinator {
    pub fn new(sub_combinator0: Box<ContractCombinator>, sub_combinator1: Box<ContractCombinator>) -> ThenCombinator {
        ThenCombinator {
            sub_combinator0,
            sub_combinator1,
            combinator_details: CombinatorDetails::new()
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

    fn get_combinator_details(&self) -> &CombinatorDetails {
        &self.combinator_details
    }

    // Acquires the combinator and acquirable sub-combinators
    fn acquire(&mut self, time: u32, or_choices: &Vec<Option<bool>>) {
        if self.past_horizon(time) {
            panic!("Acquiring an expired contract is not allowed.");
        }
        if self.combinator_details.acquisition_time != None {
            panic!("Acquiring a previously-acquired then combinator is not allowed.");
        }

        let sub_combinator;
        if !self.sub_combinator0.past_horizon(time) {
            sub_combinator = &mut self.sub_combinator0;
        } else {
            sub_combinator = &mut self.sub_combinator1;
        }

        sub_combinator.acquire(time, or_choices);
        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64 {
        // If not acquired yet or fully updated (no more pending balance), return 0
        if self.combinator_details.acquisition_time == None
            || self.combinator_details.acquisition_time.unwrap() > time
            || self.combinator_details.fully_updated {
            return 0;
        }

        let sub_combinator;
        if !self.sub_combinator0.past_horizon(self.combinator_details.acquisition_time.unwrap()) {
            sub_combinator = &mut self.sub_combinator0;
        } else {
            sub_combinator = &mut self.sub_combinator1;
        }

        let sub_value = sub_combinator.update(time, or_choices, obs_values);
        self.combinator_details.fully_updated = sub_combinator.get_combinator_details().fully_updated;
        sub_value
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

    // Acquiring give-combinator sets combinator details correctly
    #[test]
    fn acquiring_sets_combinator_details() {
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

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
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Acquire and check value
        combinator.acquire(2, &vec![]);
        let value = combinator.update(2, &vec![], &vec![]);

        assert_eq!(
            value,
            1,
            "Update value of then truncate 1 zero one at time = 2 is not equal to 1: {}",
            value
        );
    }

    // Acquiring and updating combinator sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated() {
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Acquire and check value
        combinator.acquire(2, &vec![]);
        combinator.update(2, &vec![], &vec![]);
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
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Acquire and check value
        combinator.acquire(0, &vec![]);
        combinator.update(2, &vec![], &vec![]);
        let value = combinator.update(0, &vec![], &vec![]);

        assert_eq!(
            value,
            0,
            "Second update value of truncate 1 zero one is not equal to 0: {}",
            value
        );
    }

    // Updating before acquiring does not set fully updated, and returns correct value
    #[test]
    fn updating_before_acquiring_does_nothing() {
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Update check details
        let value = combinator.update(0, &vec![], &vec![]);
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
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Update check details
        combinator.acquire(3, &vec![]);
        let value = combinator.update(1, &vec![], &vec![]);
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
    fn updating_with_different_time_to_acquisition_time_returns_correct_value() {
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Update check details
        combinator.acquire(0, &vec![]);
        let value = combinator.update(2, &vec![], &vec![]);

        assert_eq!(
            value,
            0,
            "Value of updating later than acquiring != 0: {}",
            value
        )
    }

    // Acquiring combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired then combinator is not allowed.")]
    fn should_panic_when_acquiring_combinator_twice() {
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Acquire twice
        combinator.acquire(0, &vec![]);
        combinator.acquire(2, &vec![]);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Acquiring an expired contract is not allowed.")]
    fn should_panic_when_acquiring_post_expiry() {
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                2
            ))
        );

        // Acquire at time = 3
        combinator.acquire(3, &vec![]);
    }
}