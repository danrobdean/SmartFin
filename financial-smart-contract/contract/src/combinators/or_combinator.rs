use super::contract_combinator::{ Combinator, ContractCombinator, CombinatorDetails, latest_time, deserialize_combinator, Box, Vec };
use { or_choices_key };
use storage::*;

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
    fn get_or_choice(&self, time: u32, storage: &mut Storage) -> Option<bool> {
        if self.sub_combinator0.past_horizon(time) {
            Some(false)
        } else if self.sub_combinator1.past_horizon(time) {
            Some(true)
        } else {
            storage.get(&or_choices_key(), self.or_index)
        }
    }

    // Deserialize
    pub fn deserialize(index: usize, serialized_combinator: &Vec<i64>) -> (usize, Box<ContractCombinator>) {
        if index + 2 >= serialized_combinator.len() {
            panic!("Attempted to deserialize ill-formed serialized OrCombinator.")
        }
        let (index0, sub_combinator0) = deserialize_combinator(index + 3, serialized_combinator);
        let (index1, sub_combinator1) = deserialize_combinator(index0, serialized_combinator);

        (
            index1,
            Box::new(OrCombinator {
                sub_combinator0,
                sub_combinator1,
                or_index: serialized_combinator[index + 2] as usize,
                combinator_details: CombinatorDetails::deserialize([serialized_combinator[index], serialized_combinator[index + 1]])
            })
        )
    }
}

// Contract combinator implementation for the or combinator
impl ContractCombinator for OrCombinator {
    fn get_combinator_number(&self) -> Combinator {
        Combinator::OR
    }

    // Returns the latest of the two sub-horizons
    fn get_horizon(&self) -> Option<u32> {
        latest_time(self.sub_combinator0.get_horizon(), self.sub_combinator1.get_horizon())
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
            panic!("Acquiring a previously-acquired or combinator is not allowed.");
        }

        // Check which sub-combinator to acquire. If ambiguous, acquire both branches.
        match self.get_or_choice(time, storage) {
            Some(true) => self.sub_combinator0.acquire(time, storage),
            Some(false) => self.sub_combinator1.acquire(time, storage),
            None => { }
        }

        self.combinator_details.acquisition_time = Some(time);
    }

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, storage: &mut Storage) -> i64 {
        // If not acquired yet or fully updated (no more pending balance), return 0
        if self.combinator_details.acquisition_time == None
            || self.combinator_details.acquisition_time.unwrap() > time
            || self.combinator_details.fully_updated {
            return 0;
        }

        let or_choice = self.get_or_choice(self.combinator_details.acquisition_time.unwrap(), storage);

        let sub_combinator;
        match or_choice {
            Some(true) => sub_combinator = &mut self.sub_combinator0,
            Some(false) => sub_combinator = &mut self.sub_combinator1,
            None => return 0
        }

        if sub_combinator.get_combinator_details().acquisition_time == None {
            sub_combinator.acquire(self.combinator_details.acquisition_time.unwrap(), storage);
        }

        let sub_value = sub_combinator.update(time, storage);
        self.combinator_details.fully_updated = sub_combinator.get_combinator_details().fully_updated;
        sub_value
    }

    // Serializes this combinator
    fn serialize(&self) -> Vec<i64> {
        let mut serialized = self.serialize_details();
        serialized.push(self.or_index as i64);
        serialized.extend_from_slice(&self.sub_combinator0.serialize());
        serialized.extend_from_slice(&self.sub_combinator1.serialize());
        serialized
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, Combinator, OrCombinator, OneCombinator, ZeroCombinator, TruncateCombinator, GetCombinator };
    use super::super::contract_combinator::{ Box, Vec, vec };
    use { or_choices_key };
    use storage::*;

    // Sets up the storage struct
    fn setup_storage(or_choices: &Vec<Option<bool>>) -> Storage {
        let mut storage = Storage::new();
        storage.write_vec(&or_choices_key(), or_choices);
        storage
    }

    // Combinator number is correct
    #[test]
    fn correct_combinator_number() {
        let combinator = OrCombinator::new(Box::new(ZeroCombinator::new()), Box::new(ZeroCombinator::new()), 0);
        assert_eq!(combinator.get_combinator_number(), Combinator::OR);
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
        let mut storage = setup_storage(&vec![Some(true)]);
        combinator.acquire(time, &mut storage);
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
        let mut storage = setup_storage(&vec![Some(false)]);
        combinator.acquire(0, &mut storage);
        let value = combinator.update(0, &mut storage);

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
        let mut storage = setup_storage(&vec![Some(false)]);
        combinator.acquire(0, &mut storage);
        combinator.update(0, &mut storage);
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
        // Create combinator or zero one
        let mut combinator = OrCombinator::new(
            Box::from(ZeroCombinator::new()),
            Box::from(OneCombinator::new()),
            0
        );

        // Acquire and check value
        let mut storage = setup_storage(&vec![Some(false)]);
        combinator.acquire(0, &mut storage);
        combinator.update(0, &mut storage);
        let value = combinator.update(0, &mut storage);

        assert_eq!(
            value,
            0,
            "Second update value of or zero one is not equal to 0: {}",
            value
        );
    }

    // Acquiring and updating combinator after the acquired sub-combinator expires returns the correct value
    #[test]
    fn acquiring_and_updating_after_acquired_sub_contract_expires_returns_correct_value() {
        // Create combinator or get truncate 2 one zero
        let mut combinator = OrCombinator::new(
            Box::from(GetCombinator::new(
                Box::from(TruncateCombinator::new(
                    Box::from(OneCombinator::new()),
                    2
                ))
            )),
            Box::from(ZeroCombinator::new()),
            0
        );

        // Acquire, update and check value
        let mut storage = setup_storage(&vec![Some(true)]);
        combinator.acquire(0, &mut storage);
        let value = combinator.update(3, &mut storage);

        assert_eq!(
            value,
            1,
            "Second update value of or get truncate 2 one zero is not equal to 1: {}",
            value
        );
    }

    // Acquiring and updating combinator after the acquired sub-combinator expires returns the correct value
    #[test]
    fn acquiring_and_updating_after_unacquired_sub_contract_expires_returns_correct_value() {
        // Create combinator or get truncate 2 one zero
        let mut combinator = OrCombinator::new(
            Box::from(GetCombinator::new(
                Box::from(TruncateCombinator::new(
                    Box::from(OneCombinator::new()),
                    2
                ))
            )),
            Box::from(ZeroCombinator::new()),
            0
        );

        // Acquire, update and check value
        let mut storage = setup_storage(&vec![None]);
        combinator.acquire(0, &mut storage);
        storage.set(&or_choices_key(), 0, Some(true));
        let value = combinator.update(3, &mut storage);

        assert_eq!(
            value,
            1,
            "Second update value of or get truncate 2 one zero is not equal to 1: {}",
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
        let mut storage = setup_storage(&vec![Some(false)]);
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
        let mut storage = setup_storage(&vec![Some(false)]);
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
        let mut storage = setup_storage(&vec![None]);
        combinator.acquire(0, &mut storage);
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
        let mut storage = setup_storage(&vec![Some(true)]);
        combinator.acquire(0, &mut storage);
        let value = combinator.update(2, &mut storage);

        assert_eq!(
            value,
            0,
            "Value of updating later than acquiring != 0: {}",
            value
        )
    }

    // Serializing or-combinator is correct
    #[test]
    fn serialization_correct() {
        let sub_combinator0 = OneCombinator::new();
        let sub_combinator1 = ZeroCombinator::new();
        let mut sub_combinators_serialized = sub_combinator0.serialize();
        sub_combinators_serialized.extend_from_slice(&sub_combinator1.serialize()[..]);
        let or_index = 13;
        let combinator = OrCombinator::new(Box::new(sub_combinator0), Box::new(sub_combinator1), or_index);
        let serialized = combinator.serialize();
        assert_eq!(serialized[0..3], combinator.serialize_details()[..]);
        assert_eq!(serialized[3] as usize, or_index);
        assert_eq!(serialized[4..], sub_combinators_serialized[..]);
    }

    // Deserializing or-combinator is correct
    #[test]
    fn deserialization_correct() {
        let or_index = 0;
        let mut combinator = OrCombinator::new(Box::new(OneCombinator::new()), Box::new(ZeroCombinator::new()), or_index);
        let or_choices = &vec![Some(true)];
        let mut storage = setup_storage(or_choices);
        combinator.acquire(1, &mut storage);
        combinator.update(2, &mut storage);
        let serialized = combinator.serialize();
        let deserialized = OrCombinator::deserialize(1, &serialized).1;
        assert_eq!(deserialized.serialize(), serialized);
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
        let mut storage = setup_storage(&vec![Some(false)]);
        combinator.acquire(0, &mut storage);
        combinator.acquire(0, &mut storage);
    }

    // Acquiring combinator post-expiry is not allowed
    #[test]
    #[should_panic(expected = "Cannot acquire an expired contract.")]
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
        let mut storage = setup_storage(&vec![]);
        combinator.acquire(1, &mut storage);
    }
}