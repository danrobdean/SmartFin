use super::contract_combinator::{ Combinator, ContractCombinator, CombinatorDetails, deserialize_combinator, Box, Vec };

// The get combinator
pub struct GetCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>,

    // The common combinator details
    combinator_details: CombinatorDetails
}

// Method implementation for the get combinator
impl GetCombinator {
    pub fn new(sub_combinator: Box<ContractCombinator>) -> GetCombinator {
        GetCombinator {
            sub_combinator,
            combinator_details: CombinatorDetails::new()
        }
    }

    // Deserialize
    pub fn deserialize(index: usize, serialized_combinator: &Vec<i64>) -> (usize, Box<ContractCombinator>) {
        if index + 1 >= serialized_combinator.len() {
            panic!("Attempted to deserialize ill-formed serialized GetCombinator.")
        }
        let (index0, sub_combinator) = deserialize_combinator(index + 2, serialized_combinator);

        (
            index0,
            Box::new(GetCombinator {
                sub_combinator,
                combinator_details: CombinatorDetails::deserialize([serialized_combinator[index], serialized_combinator[index + 1]])
            })
        )
    }
}

// Contract combinator implementation for the get combinator
impl ContractCombinator for GetCombinator {
    fn get_combinator_number(&self) -> Combinator {
        Combinator::GET
    }

    // Returns the sub-horizon
    fn get_horizon(&self) -> Option<u32> {
        self.sub_combinator.get_horizon()
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<(bool, Option<u32>)>) -> i64 {
        if self.past_horizon(time) {
            self.sub_combinator.get_value(self.get_horizon().unwrap(), or_choices, obs_values, anytime_acquisition_times)
        } else {
            0
        }
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
            panic!("Acquiring a previously-acquired get combinator is not allowed.");
        }

        // If sub-combinator will expire, set its acquisition time to its horizon, otherwise it can never be acquired
        if self.sub_combinator.get_horizon() != None {
            let horizon = self.sub_combinator.get_horizon().unwrap();
            self.sub_combinator.acquire(horizon, or_choices, anytime_acquisition_times);
        } else {
            self.combinator_details.fully_updated = true;
        }

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

        let sub_value = self.sub_combinator.update(time, or_choices, obs_values, anytime_acquisition_times);
        self.combinator_details.fully_updated = self.sub_combinator.get_combinator_details().fully_updated;
        sub_value
    }

    // Serializes this combinator
    fn serialize(&self) -> Vec<i64> {
        let mut serialized = self.serialize_details();
        serialized.extend_from_slice(&self.sub_combinator.serialize());
        serialized
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, Combinator, GetCombinator, OneCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, vec };

    // Combinator number is correct
    #[test]
    fn correct_combinator_number() {
        let combinator = GetCombinator::new(Box::new(OneCombinator::new()));
        assert_eq!(combinator.get_combinator_number(), Combinator::GET);
    }
    
    // Value is sub-combinator's value
    #[test]
    fn correct_value_after_horizon() {
        // Create combinator get truncate 1 one
        let combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Check value = 1
        let value = combinator.get_value(2, &vec![], &vec![], &vec![]);
        assert_eq!(
            value,
            1,
            "Value of 'get truncate 1 one' at time = 2 is not equal to 1: {}",
            value
        );
    }
    
    // Value is 0 before horizon
    #[test]
    fn correct_value_before_horizon() {
        // Create combinator get truncate 1 one
        let combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Check value = 0
        let value = combinator.get_value(0, &vec![], &vec![], &vec![]);
        assert_eq!(
            value,
            0,
            "Value of 'get truncate 1 one' at time = 0 is not equal to 0: {}",
            value
        );
    }
    
    // Value is 0 if the sub-combinator has no horizon
    #[test]
    fn correct_value_no_horizon() {
        // Create combinator get truncate 1 one
        let combinator = GetCombinator::new(
            Box::from(OneCombinator::new())
        );

        // Check value = 0
        let value = combinator.get_value(0, &vec![], &vec![], &vec![]);
        assert_eq!(
            value,
            0,
            "Value of 'get one' at time = 0 is not equal to 0: {}",
            value
        );
    }

    // Horizon is equal to sub-combinator's horizon
    #[test]
    fn horizon_equals_sub_combinator_horizon() {
        // Create combinator get truncate 1 one
        let combinator = GetCombinator::new(
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
            "Horizon of combinator 'get truncate 1 one' is not equal to Some(1): {:?}",
            horizon
        );
    }

    // Acquiring get-combinator sets combinator details correctly
    #[test]
    fn acquiring_sets_combinator_details() {
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
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

    // Acquiring and updating combinator returns correct value before the horizon
    #[test]
    fn acquiring_and_updating_returns_correct_value_before_horizon() {
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            0,
            "Update value of get truncate 1 one at time = 0 is not equal to 0: {}",
            value
        );
    }

    // Acquiring and updating combinator returns correct value after the horizon
    #[test]
    fn acquiring_and_updating_returns_correct_value_after_horizon() {
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        let value = combinator.update(2, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            1,
            "Update value of get truncate 1 one at time = 2 is not equal to 1: {}",
            value
        );
    }

    // Acquiring and updating combinator before horizon sets fully updated to false
    #[test]
    fn acquiring_and_updating_does_not_set_fully_updated_before_horizon() {
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.update(0, &vec![], &vec![], &mut vec![]);
        let fully_updated = combinator.get_combinator_details().fully_updated;

        assert!(
            !fully_updated,
            "fully_updated is not false: {}",
            fully_updated
        );
    }

    // Acquiring and updating combinator after horizon sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated_after_horizon() {
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
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
    fn acquiring_and_updating_twice_after_horizon_returns_correct_value() {
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.update(2, &vec![], &vec![], &mut vec![]);
        let value = combinator.update(2, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            0,
            "Second update value of get truncate 1 one is not equal to 0: {}",
            value
        );
    }

    // Acquiring and updating combinator twice returns correct value with first update before horizon
    #[test]
    fn acquiring_and_updating_twice_with_one_update_before_horizon_returns_correct_value() {
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.update(0, &vec![], &vec![], &mut vec![]);
        let value = combinator.update(2, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            1,
            "Second update value of get truncate 1 one after horizon is not equal to 1: {}",
            value
        );
    }

    // Updating before acquiring does not set fully updated, and returns correct value
    #[test]
    fn updating_before_acquiring_does_nothing() {
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
        );

        // Update check details
        let value = combinator.update(2, &vec![], &vec![], &mut vec![]);
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
        // Create combinator get truncate 1 one
        let mut combinator = GetCombinator::new(
            Box::from(TruncateCombinator::new(
                Box::from(OneCombinator::new()),
                1
            ))
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

    // Acquiring with no horizon sets fully_updated to true, and value to 0
    #[test]
    fn acquiring_and_updating_with_no_horizon_does_nothing() {
        // Create combinator get one
        let mut combinator = GetCombinator::new(
            Box::from(OneCombinator::new())
        );

        // Update check details
        combinator.acquire(0, &vec![], &mut vec![]);
        let value = combinator.update(10, &vec![], &vec![], &mut vec![]);
        let combinator_details = combinator.get_combinator_details();

        assert!(
            combinator_details.fully_updated,
            "fully_updated != true: {}",
            combinator_details.fully_updated
        );

        assert_eq!(
            value,
            0,
            "Value of updating with no horizon != 0: {}",
            value
        )
    }

    // Serializing get-combinator is correct
    #[test]
    fn serialization_correct() {
        let sub_combinator = OneCombinator::new();
        let sub_combinator_serialized = sub_combinator.serialize();
        let combinator = GetCombinator::new(Box::new(sub_combinator));
        let serialized = combinator.serialize();
        assert_eq!(serialized[0..3], combinator.serialize_details()[..]);
        assert_eq!(serialized[3..], sub_combinator_serialized[..]);
    }

    // Deserializing get-combinator is correct
    #[test]
    fn deserialization_correct() {
        let mut combinator = GetCombinator::new(Box::new(OneCombinator::new()));
        combinator.acquire(1, &vec![], &mut vec![]);
        combinator.update(2, &vec![], &vec![], &mut vec![]);
        let serialized = combinator.serialize();
        let deserialized = GetCombinator::deserialize(1, &serialized).1;
        assert_eq!(deserialized.serialize(), serialized);
    }

    // Acquiring combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired get combinator is not allowed.")]
    fn should_panic_when_acquiring_combinator_twice() {
        // Create combinator
        let mut combinator = GetCombinator::new(Box::new(OneCombinator::new()));

        // Acquire twice
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.acquire(0, &vec![], &mut vec![]);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Cannot acquire an expired contract.")]
    fn should_panic_when_acquiring_post_expiry() {
        // Create combinator
        let mut combinator = GetCombinator::new(
            Box::new(TruncateCombinator::new(
                Box::new(OneCombinator::new()),
                0
            ))
        );

        // Acquire at time = 1
        combinator.acquire(1, &vec![], &mut vec![]);
    }
}