use super::contract_combinator::{ Combinator, ContractCombinator, CombinatorDetails, Vec, Box };

// The zero combinator
pub struct ZeroCombinator {
    // The common combinator details
    combinator_details: CombinatorDetails
}

// Method implementation of the zero combinator
impl ZeroCombinator {
    // Constructor
    pub fn new() -> ZeroCombinator {
        ZeroCombinator {
            combinator_details: CombinatorDetails::new()
        }
    }

    // Deserialize
    pub fn deserialize(index: usize, serialized_combinator: &Vec<i64>) -> (usize, Box<ContractCombinator>) {
        if index + 1 >= serialized_combinator.len() {
            panic!("Attempted to deserialize ill-formed serialized ZeroCombinator.")
        }
        (
            index + 2,
            Box::new(ZeroCombinator {
                combinator_details: CombinatorDetails::deserialize([serialized_combinator[index], serialized_combinator[index + 1]])
            })
        )
    }
}

// Contract combinator implementation of the zero combinator
impl ContractCombinator for ZeroCombinator {
    fn get_combinator_number(&self) -> Combinator {
        Combinator::ZERO
    }

    fn get_combinator_details(&self) -> &CombinatorDetails {
        &self.combinator_details
    }

    // Acquires the combinator and acquirable sub-combinators
    fn acquire(&mut self, time: u32, _or_choices: &Vec<Option<bool>>, _anytime_acquisition_times: &mut Vec<(bool, Option<u32>)>) {
        if self.combinator_details.acquisition_time != None {
            panic!("Acquiring a previously-acquired zero combinator is not allowed.");
        }

        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, _or_choices: &Vec<Option<bool>>, _obs_values: &Vec<Option<i64>>, _anytime_acquisition_times: &mut Vec<(bool, Option<u32>)>) -> i64 {
        // If not acquired yet or fully updated (no more pending balance), return 0
        if self.combinator_details.acquisition_time == None
            || self.combinator_details.acquisition_time.unwrap() > time
            || self.combinator_details.fully_updated {
            return 0;
        }

        self.combinator_details.fully_updated = true;
        0
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, Combinator, ZeroCombinator };
    use super::super::contract_combinator::{ vec };

    // Combinator number is correct
    #[test]
    fn correct_combinator_number() {
        let combinator = ZeroCombinator::new();
        assert_eq!(combinator.get_combinator_number(), Combinator::ZERO);
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

    // Acquiring zero-combinator sets combinator details correctly
    #[test]
    fn acquiring_sets_combinator_details() {
        // Create combinator zero
        let mut combinator = ZeroCombinator::new();

        // Acquire and check details
        let time: u32 = 5;
        combinator.acquire(time, &vec![], &mut vec![]);
        let combinator_details = combinator.get_combinator_details();

        assert_eq!(
            combinator_details.acquisition_time,
            Some(time),
            "Acquisition time of zero combinator is not equal to Some(5): {:?}",
            combinator_details.acquisition_time
        );
    }

    // Acquiring and updating zero-combinator sets fully updated to true
    #[test]
    fn acquiring_and_updating_sets_fully_updated() {
        // Create combinator zero
        let mut combinator = ZeroCombinator::new();

        // Acquire and check details
        let time: u32 = 5;
        combinator.acquire(time, &vec![], &mut vec![]);
        combinator.update(time, &vec![], &vec![], &mut vec![]);
        let combinator_details = combinator.get_combinator_details();

        assert!(
            combinator_details.fully_updated,
            "fully_updated != true: {}",
            combinator_details.fully_updated
        );
    }

    // Acquiring and updating zero-combinator returns correct value
    #[test]
    fn acquiring_and_updating_returns_correct_value() {
        // Create combinator zero
        let mut combinator = ZeroCombinator::new();

        // Acquire and check value
        combinator.acquire(0, &vec![], &mut vec![]);
        let value = combinator.update(0, &vec![], &vec![], &mut vec![]);

        assert_eq!(
            value,
            0,
            "Acquisition value of zero combinator is not equal to 0: {}",
            value
        );
    }

    // Updating before acquiring does not set fully updated, and returns correct value
    #[test]
    fn updating_before_acquiring_does_nothing() {
        // Create combinator zero
        let mut combinator = ZeroCombinator::new();

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
        // Create combinator zero
        let mut combinator = ZeroCombinator::new();

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

    // Deserializing zero-combinator is correct
    #[test]
    fn deserialization_correct() {
        let mut combinator = ZeroCombinator::new();
        let mut serialized = combinator.serialize();
        let mut deserialized = ZeroCombinator::deserialize(1, &serialized).1;
        assert_eq!(deserialized.serialize(), serialized);

        combinator.acquire(1, &vec![], &mut vec![]);
        combinator.update(2, &vec![], &vec![], &mut vec![]);
        serialized = combinator.serialize();
        deserialized = ZeroCombinator::deserialize(1, &serialized).1;
        assert_eq!(deserialized.serialize(), serialized)
    }

    // Acquiring zero-combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired zero combinator is not allowed.")]
    fn should_panic_when_acquiring_zero_combinator_twice() {
        // Create combinator zero
        let mut combinator = ZeroCombinator::new();

        // Acquire twice
        combinator.acquire(0, &vec![], &mut vec![]);
        combinator.acquire(0, &vec![], &mut vec![]);
    }
}