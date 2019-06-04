use super::contract_combinator::{ Combinator, ContractCombinator, CombinatorDetails, deserialize_combinator, Box, Vec };
use { anytime_acquisition_times_key };
use storage::*;

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

    // Deserialize
    pub fn deserialize(index: usize, serialized_combinator: &Vec<i64>) -> (usize, Box<ContractCombinator>) {
        if index + 2 >= serialized_combinator.len() {
            panic!("Attempted to deserialize ill-formed serialized AnytimeCombinator.")
        }
        let (index0, sub_combinator) = deserialize_combinator(index + 3, serialized_combinator);

        (
            index0,
            Box::new(AnytimeCombinator {
                sub_combinator,
                anytime_index: serialized_combinator[index + 2] as usize,
                combinator_details: CombinatorDetails::deserialize([serialized_combinator[index], serialized_combinator[index + 1]])
            })
        )
    }
}

// Contract combinator implementation for the anytime combinator
impl ContractCombinator for AnytimeCombinator {
    fn get_combinator_number(&self) -> Combinator {
        Combinator::ANYTIME
    }

    // Returns the sub-horizon
    fn get_horizon(&self) -> Option<u32> {
        self.sub_combinator.get_horizon()
    }

    fn get_combinator_details(&self) -> &CombinatorDetails {
        &self.combinator_details
    }

    // Acquires the combinator and acquirable sub-combinators
    fn acquire(&mut self, time: u32, storage: &mut Storage) {
        if self.past_horizon(time) {
            panic!("Cannot acquire an expired contract.");
        }
        if self.combinator_details.acquisition_time != None {
            panic!("Acquiring a previously-acquired anytime combinator is not allowed.");
        }

        storage.set(&anytime_acquisition_times_key(), self.anytime_index, (true, self.sub_combinator.get_horizon()));
        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, setting the acquisition time, and returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, storage: &mut Storage) -> i64 {
        // If not acquired yet or fully updated (no more pending balance), return 0
        if self.combinator_details.acquisition_time == None
            || self.combinator_details.acquisition_time.unwrap() > time
            || self.combinator_details.fully_updated {
            return 0;
        }

        // Check if sub-combinator acquisition time already set
        let mut acquisition_time = self.sub_combinator.get_combinator_details().acquisition_time;

        // If not, check if provided
        let anytime_acquisition_time: (bool, Option<u32>) = storage.get(&anytime_acquisition_times_key(), self.anytime_index);
        if acquisition_time == None && anytime_acquisition_time.1 != None {
            acquisition_time = anytime_acquisition_time.1;

            // If sub-horizon is before the given acquisition time, use sub-horizon as acquisition time
            if self.sub_combinator.past_horizon(acquisition_time.unwrap()) {
                acquisition_time = self.sub_combinator.get_horizon();
            }

            if self.combinator_details.acquisition_time.unwrap() > acquisition_time.unwrap() {
                panic!("Cannot acquire anytime sub-combinator before the anytime combinator is acquired.");
            }

            // If acquisition time has been passed then acquire sub-contract, otherwise do nothing more
            if acquisition_time.unwrap() <= time {
                self.sub_combinator.acquire(acquisition_time.unwrap(), storage);
            }
        }

        let sub_value = self.sub_combinator.update(time, storage);
        self.combinator_details.fully_updated = self.sub_combinator.get_combinator_details().fully_updated;
        sub_value
    }

    // Serializes this combinator
    fn serialize(&self) -> Vec<i64> {
        let mut serialized = self.serialize_details();
        serialized.push(self.anytime_index as i64);
        serialized.extend_from_slice(&self.sub_combinator.serialize());
        serialized
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, Combinator, AnytimeCombinator, OneCombinator, TruncateCombinator };
    use super::super::contract_combinator::{ Box, Vec, vec };
    use { anytime_acquisition_times_key };
    use storage::*;

    // Sets up the storage struct
    fn setup_storage(anytime_acquisition_times: &Vec<(bool, Option<u32>)>) -> Storage {
        let mut storage = Storage::new();
        storage.write_vec(&anytime_acquisition_times_key(), anytime_acquisition_times);
        storage
    }

    // Combinator number is correct
    #[test]
    fn correct_combinator_number() {
        let combinator = AnytimeCombinator::new(Box::new(OneCombinator::new()), 0);
        assert_eq!(combinator.get_combinator_number(), Combinator::ANYTIME);
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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(time, &mut storage);
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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        let value = combinator.update(0, &mut storage);

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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        assert_eq!(StoresFixedVec::<(bool, Option<u32>)>::get(&mut storage, &anytime_acquisition_times_key(), 0).1, Some(1));
        let value = combinator.update(2, &mut storage);

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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        storage.set(&anytime_acquisition_times_key(), 0, (true, Some(3 as u32)));
        let value = combinator.update(2, &mut storage);

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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        storage.set(&anytime_acquisition_times_key(), 0, (true, Some(1 as u32)));
        combinator.update(0, &mut storage);
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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);

        combinator.acquire(0, &mut storage);
        storage.set(&anytime_acquisition_times_key(), 0, (true, Some(0 as u32)));
        combinator.update(1, &mut storage);

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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);

        combinator.acquire(0, &mut storage);
        combinator.update(3, &mut storage);

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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        storage.set(&anytime_acquisition_times_key(), 0, (true, Some(1 as u32)));
        let value = combinator.update(1, &mut storage);

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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        combinator.update(2, &mut storage);
        let value = combinator.update(2, &mut storage);

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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        combinator.update(0, &mut storage);
        let value = combinator.update(2, &mut storage);

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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        let value = combinator.update(2, &mut storage);
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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(1, &mut storage);
        let value = combinator.update(0, &mut storage);
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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        let value = combinator.update(10, &mut storage);
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

    // Serializing anytime-combinator is correct
    #[test]
    fn serialization_correct() {
        let sub_combinator = OneCombinator::new();
        let sub_combinator_serialized = sub_combinator.serialize();
        let combinator = AnytimeCombinator::new(Box::new(sub_combinator), 0);
        let serialized = combinator.serialize();
        assert_eq!(serialized[0..3], combinator.serialize_details()[..]);
        assert_eq!(serialized[3] as usize, combinator.anytime_index);
        assert_eq!(serialized[4..], sub_combinator_serialized[..]);
    }

    // Deserializing anytime-combinator is correct
    #[test]
    fn deserialization_correct() {
        let mut combinator = AnytimeCombinator::new(Box::new(OneCombinator::new()), 0);
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(1, &mut storage);
        combinator.update(2, &mut storage);

        let serialized = combinator.serialize();
        let deserialized = AnytimeCombinator::deserialize(1, &serialized).1;
        assert_eq!(deserialized.serialize(), serialized);
    }

    // Acquiring combinator twice is not allowed
    #[test]
    #[should_panic(expected = "Acquiring a previously-acquired anytime combinator is not allowed.")]
    fn should_panic_when_acquiring_combinator_twice() {
        // Create combinator
        let mut combinator = AnytimeCombinator::new(Box::new(OneCombinator::new()), 0);

        // Acquire twice
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(0, &mut storage);
        combinator.acquire(0, &mut storage);
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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(1, &mut storage);
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
        let acquisition_times = &vec![(false, None)];
        let mut storage = setup_storage(acquisition_times);
        combinator.acquire(1, &mut storage);
        storage.set(&anytime_acquisition_times_key(), 0, (true, Some(0 as u32)));
        combinator.update(2, &mut storage);
    }
}