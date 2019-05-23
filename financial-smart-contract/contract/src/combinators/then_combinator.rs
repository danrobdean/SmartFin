use super::contract_combinator::{ Combinator, ContractCombinator, CombinatorDetails, latest_time, deserialize_combinator, Box, Vec };

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

    // Deserialize
    pub fn deserialize(index: usize, serialized_combinator: &Vec<i64>) -> (usize, Box<ContractCombinator>) {
        if index + 1 >= serialized_combinator.len() {
            panic!("Attempted to deserialize ill-formed serialized ThenCombinator.")
        }
        let (index0, sub_combinator0) = deserialize_combinator(index + 2, serialized_combinator);
        let (index1, sub_combinator1) = deserialize_combinator(index0, serialized_combinator);

        (
            index1,
            Box::new(ThenCombinator {
                sub_combinator0,
                sub_combinator1,
                combinator_details: CombinatorDetails::deserialize([serialized_combinator[index], serialized_combinator[index + 1]])
            })
        )
    }
}

// Contract combinator implementation for the then combinator
impl ContractCombinator for ThenCombinator {
    fn get_combinator_number(&self) -> Combinator {
        Combinator::THEN
    }

    // Returns the latest of the two sub-horizons
    fn get_horizon(&self) -> Option<u32> {
        latest_time(self.sub_combinator0.get_horizon(), self.sub_combinator1.get_horizon())
    }

    fn get_combinator_details(&self) -> &CombinatorDetails {
        &self.combinator_details
    }

    // Acquires the combinator and acquirable sub-combinators
    fn acquire(&mut self, time: u32, or_choices: &Vec<Option<bool>>, anytime_acquisition_times: &mut Vec<(bool, Option<u32>)>) {
        if self.past_horizon(time) {
            panic!("Cannot acquire an expired contract.");
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

        sub_combinator.acquire(time, or_choices, anytime_acquisition_times);
        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &mut Vec<(bool, Option<u32>)>) -> i64 {
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

        let sub_value = sub_combinator.update(time, or_choices, obs_values, anytime_acquisition_times);
        self.combinator_details.fully_updated = sub_combinator.get_combinator_details().fully_updated;
        sub_value
    }

    // Serializes this combinator
    fn serialize(&self) -> Vec<i64> {
        let mut serialized = self.serialize_details();
        serialized.extend_from_slice(&self.sub_combinator0.serialize());
        serialized.extend_from_slice(&self.sub_combinator1.serialize());
        serialized
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, Combinator, ThenCombinator, TruncateCombinator, ZeroCombinator, OneCombinator };
    use super::super::contract_combinator::{ Box, vec };

    // Combinator number is correct
    #[test]
    fn correct_combinator_number() {
        let combinator = ThenCombinator::new(Box::new(OneCombinator::new()), Box::new(OneCombinator::new()));
        assert_eq!(combinator.get_combinator_number(), Combinator::THEN);
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
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Acquire and check value
        combinator.acquire(2, &vec![], &mut vec![]);
        let value = combinator.update(2, &vec![], &vec![], &mut vec![]);

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
        combinator.acquire(2, &vec![], &mut vec![]);
        combinator.update(2, &vec![], &vec![], &mut vec![]);
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
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.update(2, &vec![], &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);

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
        // Create combinator then truncate 1 zero one
        let mut combinator = ThenCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(ZeroCombinator::new()),
                1
            )),
            Box::from(OneCombinator::new())
        );

        // Update check details
        combinator.acquire(3, &vec![], &mut vec![]);
        let value = combinator.update(1, &vec![], &vec![], &mut vec![]);
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
        combinator.acquire(0, &vec![], &mut vec![]);
        let value = combinator.update(2, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            0,
            "Value of updating later than acquiring != 0: {}",
            value
        )
    }

    // Serializing then-combinator is correct
    #[test]
    fn serialization_correct() {
        let sub_combinator0 = OneCombinator::new();
        let sub_combinator1 = ZeroCombinator::new();
        let mut sub_combinators_serialized = sub_combinator0.serialize();
        sub_combinators_serialized.extend_from_slice(&sub_combinator1.serialize()[..]);
        let combinator = ThenCombinator::new(Box::new(sub_combinator0), Box::new(sub_combinator1));
        let serialized = combinator.serialize();
        assert_eq!(serialized[0..3], combinator.serialize_details()[..]);
        assert_eq!(serialized[3..], sub_combinators_serialized[..]);
    }

    // Deserializing then-combinator is correct
    #[test]
    fn deserialization_correct() {
        let mut combinator = ThenCombinator::new(Box::new(OneCombinator::new()), Box::new(ZeroCombinator::new()));
        combinator.acquire(1, &vec![], &mut vec![]);
        combinator.update(2, &vec![], &vec![], &mut vec![]);
        let serialized = combinator.serialize();
        let deserialized = ThenCombinator::deserialize(1, &serialized).1;
        assert_eq!(deserialized.serialize(), serialized);
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
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.acquire(2, &vec![], &mut vec![]);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Cannot acquire an expired contract.")]
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
        combinator.acquire(3, &vec![], &mut vec![]);
    }
}