use super::contract_combinator::{ ContractCombinator, CombinatorDetails, Box, Vec };

// The give combinator
pub struct GiveCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>,

    // The common combinator details
    combinator_details: CombinatorDetails
}

// Method implementation for the give combinator
impl GiveCombinator {
    pub fn new(sub_combinator: Box<ContractCombinator>) -> GiveCombinator {
        GiveCombinator {
            sub_combinator,
            combinator_details: CombinatorDetails::new()
        }
    }
}

// Contract combinator implementation for the give combinator
impl ContractCombinator for GiveCombinator {
    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<Option<u32>>) -> i64 {
        -1 * self.sub_combinator.get_value(time, or_choices, obs_values, anytime_acquisition_times)
    }

    fn get_horizon(&self) -> Option<u32> {
        self.sub_combinator.get_horizon()
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
            panic!("Acquiring a previously-acquired give combinator is not allowed.");
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
        -1 * sub_value
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
        let value = combinator.get_value(0, &vec![], &vec![], &vec![]);
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

    // Acquiring give-combinator sets combinator details correctly
    #[test]
    fn acquiring_sets_combinator_details() {
        // Create combinator give one
        let mut combinator = GiveCombinator::new(Box::new(OneCombinator::new()));

        // Acquire and check details
        let time: u32 = 5;
        combinator.acquire(time, &vec![], &mut vec![]);
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
        // Create combinator
        let mut combinator = GiveCombinator::new(Box::new(OneCombinator::new()));

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            -1,
            "Update value of give one is not equal to -1: {}",
            value
        );
    }

    // Acquiring and updating combinator sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated() {
        // Create combinator
        let mut combinator = GiveCombinator::new(Box::new(OneCombinator::new()));

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.update(0, &vec![], &vec![], &mut vec![]);
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
        // Create combinator
        let mut combinator = GiveCombinator::new(Box::new(OneCombinator::new()));

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.update(0, &vec![], &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            0,
            "Second update value of give one is not equal to 0: {}",
            value
        );
    }

    // Updating before acquiring does not set fully updated, and returns correct value
    #[test]
    fn updating_before_acquiring_does_nothing() {
        // Create combinator give one
        let mut combinator = GiveCombinator::new(Box::new(OneCombinator::new()));

        // Update check details
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);
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
        // Create combinator give one
        let mut combinator = GiveCombinator::new(Box::new(OneCombinator::new()));

        // Update check details
        combinator.acquire(1, &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);
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

    // Acquiring combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired give combinator is not allowed.")]
    fn should_panic_when_acquiring_combinator_twice() {
        // Create combinator
        let mut combinator = GiveCombinator::new(Box::new(OneCombinator::new()));

        // Acquire twice
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.acquire(0, &vec![], &mut vec![]);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Acquiring an expired contract is not allowed.")]
    fn should_panic_when_acquiring_post_expiry() {
        // Create combinator
        let mut combinator = GiveCombinator::new(
            Box::new(TruncateCombinator::new(
                Box::new(OneCombinator::new()),
                0
            ))
        );

        // Acquire at time = 1
        combinator.acquire(1, &vec![], &mut vec![]);
    }
}