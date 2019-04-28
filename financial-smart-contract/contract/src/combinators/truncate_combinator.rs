use super::contract_combinator::{ Combinator, ContractCombinator, CombinatorDetails, earliest_time, Box, Vec };

// The truncate combinator
pub struct TruncateCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>,

    // The truncated horizon
    truncated_horizon: u32,

    // The common combinator details
    combinator_details: CombinatorDetails
}

// Method implementation for the truncate combinator
impl TruncateCombinator {
    pub fn new(sub_combinator: Box<ContractCombinator>, truncated_horizon: u32) -> TruncateCombinator {
        TruncateCombinator {
            sub_combinator,
            truncated_horizon,
            combinator_details: CombinatorDetails::new()
        }
    }
}

// Contract combinator implementation for the truncate combinator
impl ContractCombinator for TruncateCombinator {
    fn get_combinator_number(&self) -> Combinator {
        Combinator::TRUNCATE
    }

    // Returns the latest of the sub-horizon and the truncated horizon
    fn get_horizon(&self) -> Option<u32> {
        earliest_time(self.sub_combinator.get_horizon(), Some(self.truncated_horizon))
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<Option<u32>>) -> i64 {
        if self.past_horizon(time) {
            0
        } else {
            self.sub_combinator.get_value(time, or_choices, obs_values, anytime_acquisition_times)
        }
    }

    fn get_combinator_details(&self) -> &CombinatorDetails {
        &self.combinator_details
    }

    // Acquires the combinator and acquirable sub-combinators
    fn acquire(&mut self, time: u32, or_choices: &Vec<Option<bool>>, anytime_acquisition_times: &mut Vec<Option<u32>>) {
        if self.past_horizon(time) {
            panic!("Cannot acquire an expired contract.");
        }
        if self.combinator_details.acquisition_time != None {
            panic!("Acquiring a previously-acquired truncate combinator is not allowed.");
        }

        self.sub_combinator.acquire(time, or_choices, anytime_acquisition_times);
        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &mut Vec<Option<u32>>) -> i64 {
        // If not acquired yet or fully updated (no more pending balance), return 0
        if self.combinator_details.acquisition_time == None
            || self.combinator_details.acquisition_time.unwrap() > time
            || self.combinator_details.fully_updated {
            return 0;
        }

        let sub_value = self.sub_combinator.update(time, or_choices, obs_values, anytime_acquisition_times);
        self.combinator_details.fully_updated = self.sub_combinator.get_combinator_details().fully_updated;
        sub_value
    }

    // Serializes this combinator
    fn serialize(&self) -> Vec<i64> {
        let mut serialized = self.serialize_details();

        // Store 0 or 1 then horizon (if defined)
        match self.get_horizon() {
            Some(time) => {
                serialized.push(1);
                serialized.push(time as i64);
            },
            None => serialized.push(0)
        }
        serialized.extend_from_slice(&self.sub_combinator.serialize());
        serialized
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, OneCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, vec };
    
    // Value before expiry is equal to the value of the sub-combinator
    #[test]
    fn correct_value_pre_expiry() {
        // Create truncate 1 one
        let combinator = TruncateCombinator::new(Box::from(OneCombinator::new()), 1);

        // Check value = 1
        let value = combinator.get_value(0, &vec![], &vec![], &vec![]);
        assert_eq!(
            value,
            1,
            "Value of 'truncate 1 one' contract at time = 0 is not equal to 1: {}",
            value
        );
    }
    
    // Value after expiry is 0
    #[test]
    fn correct_value_post_expiry() {
        // Create truncate 1 one
        let combinator = TruncateCombinator::new(Box::from(OneCombinator::new()), 1);

        // Check value = 0
        let value = combinator.get_value(2, &vec![], &vec![], &vec![]);
        assert_eq!(
            value,
            0,
            "Value of 'truncate 1 one' contract at time = 2 is not equal to 0: {}",
            value
        );
    }
    
    // Horizon is correct
    #[test]
    fn correct_horizon() {
        // Create truncate 5 one
        let combinator = TruncateCombinator::new(Box::from(OneCombinator::new()), 5);

        // Check horizon = 5
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            Some(5),
            "Horizon of 'truncate 5 one' contract is not equal to Some(5): {:?}",
            horizon
        );
    }
    
    // Horizon is correct if sub-combinator expires first
    #[test]
    fn correct_horizon_sub_combinator_expires_first() {
        // Create truncate 5 truncate 4 one
        let combinator = TruncateCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                4
            )),
        5);

        // Check horizon = 4
        let horizon = combinator.get_horizon();
        assert_eq!(
            horizon,
            Some(4),
            "Horizon of 'truncate 5 truncate 4 one' contract is not equal to Some(4): {:?}",
            horizon
        );
    }

    // Acquiring give-combinator sets combinator details correctly
    #[test]
    fn acquiring_sets_combinator_details() {
        // Create truncate 2 one
        let mut combinator = TruncateCombinator::new(
            Box::from(OneCombinator::new()),
            2
        );

        // Acquire and check details
        let time: u32 = 1;
        combinator.acquire(time, &vec![], &mut vec![]);
        let combinator_details = combinator.get_combinator_details();

        assert_eq!(
            combinator_details.acquisition_time,
            Some(time),
            "Acquisition time of combinator is not equal to Some(1): {:?}",
            combinator_details.acquisition_time
        );
    }

    // Acquiring and updating combinator returns correct value
    #[test]
    fn acquiring_and_updating_returns_correct_value() {
        // Create truncate 2 one
        let mut combinator = TruncateCombinator::new(
            Box::from(OneCombinator::new()),
            2
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            1,
            "Update value of truncate 2 one is not equal to 1: {}",
            value
        );
    }

    // Acquiring and updating combinator sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated() {
        // Create truncate 2 one
        let mut combinator = TruncateCombinator::new(
            Box::from(OneCombinator::new()),
            2
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.update(0, &vec![], &vec![], &mut vec![]);
        let fully_updated = combinator.get_combinator_details().fully_updated;

        assert!(
            fully_updated,
            "fully_updated is not true: {}",
            fully_updated
        );
    }

    // Acquiring and updating combinator twice returns correct value
    #[test]
    fn acquiring_and_updating_twice_returns_correct_value() {
        // Create truncate 2 one
        let mut combinator = TruncateCombinator::new(
            Box::from(OneCombinator::new()),
            2
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.update(0, &vec![], &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            0,
            "Second update value of truncate 1 one is not equal to 0: {}",
            value
        );
    }

    // Updating before acquiring does not set fully updated, and returns correct value
    #[test]
    fn updating_before_acquiring_does_nothing() {
        // Create truncate 2 one
        let mut combinator = TruncateCombinator::new(
            Box::from(OneCombinator::new()),
            2
        );

        // Update check details
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);
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
        // Create truncate 2 one
        let mut combinator = TruncateCombinator::new(
            Box::from(OneCombinator::new()),
            2
        );

        // Update check details
        combinator.acquire(1, &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);
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

    // Acquiring combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired truncate combinator is not allowed.")]
    fn should_panic_when_acquiring_combinator_twice() {
        // Create truncate 2 one
        let mut combinator = TruncateCombinator::new(
            Box::from(OneCombinator::new()),
            2
        );

        // Acquire twice
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.acquire(0, &vec![], &mut vec![]);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Cannot acquire an expired contract.")]
    fn should_panic_when_acquiring_post_expiry() {
        // Create truncate 2 one
        let mut combinator = TruncateCombinator::new(
            Box::from(OneCombinator::new()),
            2
        );

        // Acquire at time = 3
        combinator.acquire(3, &vec![], &mut vec![]);
    }
}