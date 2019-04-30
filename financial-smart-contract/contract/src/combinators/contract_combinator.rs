extern crate pwasm_std;

pub use self::pwasm_std::{ Box, Vec, vec };

use ZeroCombinator;
use OneCombinator;
use AndCombinator;
use OrCombinator;
use TruncateCombinator;
use ScaleCombinator;
use GiveCombinator;
use ThenCombinator;
use GetCombinator;
use AnytimeCombinator;

// The types of combinators
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
pub enum Combinator {
    ZERO,
    ONE,
    AND,
    OR,
    TRUNCATE,
    SCALE,
    GIVE,
    THEN,
    GET,
    ANYTIME
}

// Conversion from i64 to Combinator
impl From<i64> for Combinator {
    fn from(val: i64) -> Combinator {
        match val {
            0 => Combinator::ZERO,
            1 => Combinator::ONE,
            2 => Combinator::AND,
            3 => Combinator::OR,
            4 => Combinator::TRUNCATE,
            5 => Combinator::SCALE,
            6 => Combinator::GIVE,
            7 => Combinator::THEN,
            8 => Combinator::GET,
            9 => Combinator::ANYTIME,
            _ => panic!("Unrecognised combinator.")
        }
    }
}

// Conversion from Combinator to i64
impl From<Combinator> for i64 {
    fn from(val: Combinator) -> i64 {
        match val {
            Combinator::ZERO => 0,
            Combinator::ONE => 1,
            Combinator::AND => 2,
            Combinator::OR => 3,
            Combinator::TRUNCATE => 4,
            Combinator::SCALE => 5,
            Combinator::GIVE => 6,
            Combinator::THEN => 7,
            Combinator::GET => 8,
            Combinator::ANYTIME => 9
        }
    }
}

// The details shared by all combinators
pub struct CombinatorDetails {
    // The acquisition time of the combinator
    pub acquisition_time: Option<u32>,

    // Whether or not the combinator is fully updated
    pub fully_updated: bool
}

// Combinator details method implementation
impl CombinatorDetails {
    // Constructor
    pub fn new() -> CombinatorDetails {
        CombinatorDetails {
            acquisition_time: None,
            fully_updated: false
        }
    }

    // Converts a serialized combinator details array to CombinatorDetails
    pub fn deserialize(details_serialized: [i64; 2]) -> CombinatorDetails {
        CombinatorDetails {
            acquisition_time: if details_serialized[0] >= 0 { Some(details_serialized[0] as u32) } else { None },
            fully_updated: details_serialized[1] == 1
        }
    }
}

// API for combinators
pub trait ContractCombinator {
    // Returns the horizon of the combinator, or -1 if none exists
    fn get_horizon(&self) -> Option<u32> {
        None
    }

    // Returns whether the given time is beyond the combinator's horizon or not
    fn past_horizon(&self, time: u32) -> bool {
        match self.get_horizon() {
            Some(t) => t < time,
            None => false
        }
    }

    // Serializes a combinator's number and details
    fn serialize_details(&self) -> Vec<i64> {
        let mut serialized: Vec<i64> = Vec::new();
        serialized.push(self.get_combinator_number() as i64);

        let details = self.get_combinator_details();
        match details.acquisition_time {
            Some(time) => serialized.push(time as i64),
            None => serialized.push(-1)
        };
        serialized.push(if details.fully_updated { 1 } else { 0 });

        serialized
    }

    // Returns the value of the combinator if acquired at the given time
    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<(bool, Option<u32>)>) -> i64;

    // Returns the common combinator details of the combinator
    fn get_combinator_details(&self) -> &CombinatorDetails;

    // Acquires the combinator, setting the acquisition time in the combinator details
    fn acquire(&mut self, time: u32, or_choices: &Vec<Option<bool>>, anytime_acquisition_times: &mut Vec<(bool, Option<u32>)>);

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &mut Vec<(bool, Option<u32>)>) -> i64;

    // Gets the combinator number
    fn get_combinator_number(&self) -> Combinator;

    // Serializes this combinator
    fn serialize(&self) -> Vec<i64> {
        self.serialize_details()
    }
}

// Returns the earliest of the given horizons
pub fn earliest_time(horizon0: Option<u32>, horizon1: Option<u32>) -> Option<u32> {
    match horizon0 {
        Some(h0) => match horizon1 {
            Some(h1) => if h0 < h1 {
                horizon0
            } else {
                horizon1
            },
            None => horizon0
        },
        None => horizon1
    }
}

// Returns the latest of the given horizons
pub fn latest_time(horizon0: Option<u32>, horizon1: Option<u32>) -> Option<u32> {
    match horizon0 {
        Some(h0) => match horizon1 {
            Some(h1) => if h0 > h1 {
                horizon0
            } else {
                horizon1
            },
            None => None
        },
        None => None
    }
}

// Deserializes a ContractCombinator
pub fn deserialize_combinator(index: usize, serialized_combinator: &Vec<i64>) -> (usize, Box<ContractCombinator>) {
    if index >= serialized_combinator.len() {
        panic!("Attempted to deserialize ill-formed serialized ContractCombinator.");
    }
    match Combinator::from(serialized_combinator[index]) {
        Combinator::ZERO => ZeroCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::ONE => OneCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::AND => AndCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::OR => OrCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::TRUNCATE => TruncateCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::SCALE => ScaleCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::GIVE => GiveCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::THEN => ThenCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::GET => GetCombinator::deserialize(index + 1, serialized_combinator),
        Combinator::ANYTIME => AnytimeCombinator::deserialize(index + 1, serialized_combinator)
    }
}

// Unit tests
#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    // Dummy combinator
    pub struct DummyCombinator {
        // The common combinator details
        combinator_details: CombinatorDetails
    }

    // Method implementation of dummy combinator
    impl DummyCombinator {
        // Constructor
        pub fn new() -> DummyCombinator {
            DummyCombinator {
                combinator_details: CombinatorDetails::new()
            }
        }
    }

    // Contract combinator implementation of the dummy combinator
    impl ContractCombinator for DummyCombinator {
        fn get_combinator_number(&self) -> Combinator {
            Combinator::ZERO
        }

        fn get_value(&self, _time: u32, _or_choices: &Vec<Option<bool>>, _obs_values: &Vec<Option<i64>>, _anytime_acquisition_times: &Vec<(bool, Option<u32>)>) -> i64 {
            panic!("Method not implemented.");
        }

        fn get_combinator_details(&self) -> &CombinatorDetails {
            &self.combinator_details
        }

        // Acquires the combinator and acquirable sub-combinators
        fn acquire(&mut self, time: u32, _or_choices: &Vec<Option<bool>>, _anytime_acquisition_times: &mut Vec<(bool, Option<u32>)>) {
            self.combinator_details.acquisition_time = Some(time);
        }

        // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
        fn update(&mut self, _time: u32, _or_choices: &Vec<Option<bool>>, _obs_values: &Vec<Option<i64>>, _anytime_acquisition_times: &mut Vec<(bool, Option<u32>)>) -> i64 {
            self.combinator_details.fully_updated = true;
            0
        }
    }

    // Earliest horizon should return the earliest horizon correctly.
    #[test]
    fn earliest_time_returns_earliest_time() {
        let horizon0 = Some(2);
        let horizon1 = Some(1);

        assert_eq!(earliest_time(horizon0, horizon1), horizon1);
    }

    // Latest horizon should return the latest horizon correctly.
    #[test]
    fn latest_time_returns_latest_time() {
        let horizon0 = Some(2);
        let horizon1 = Some(1);

        assert_eq!(latest_time(horizon0, horizon1), horizon0);
    }

    // Earliest horizon should return a horizon if one is None.
    #[test]
    fn earliest_time_returns_non_none_value() {
        let horizon0 = Some(2);
        let horizon1 = None;

        assert_eq!(earliest_time(horizon0, horizon1), horizon0);
    }

    // Latest horizon should return None if one is None.
    #[test]
    fn latest_time_returns_none_value() {
        let horizon0 = Some(2);
        let horizon1 = None;

        assert_eq!(latest_time(horizon0, horizon1), horizon1);
    }

    // Combinator to/from i64 converts correctly
    #[test]
    fn combinator_conversion_correct() {
        for i in 0..10 {
            let combinator = Combinator::from(i);
            let val = i64::from(combinator);
            assert_eq!(i, val);
        }
    }

    // Combinator details serialization is correct
    #[test]
    fn serialization_details_correct() {
        let mut combinator = DummyCombinator::new();
        let mut serialized_details = combinator.serialize_details();
        assert_eq!(Combinator::from(serialized_details[0]), combinator.get_combinator_number());
        assert_eq!(serialized_details[1], -1);
        assert_eq!(serialized_details[2], 0);


        combinator.acquire(10, &vec![], &mut vec![]);
        combinator.update(11, &vec![], &vec![], &mut vec![]);
        serialized_details = combinator.serialize_details();
        
        assert_eq!(Combinator::from(serialized_details[0]), combinator.get_combinator_number());
        assert_eq!(serialized_details[1] as u32, combinator.get_combinator_details().acquisition_time.unwrap());
        assert_eq!(serialized_details[2], 1);
    }

    // Combinator serialization is correct
    #[test]
    fn serialization_correct() {
        let mut combinator = DummyCombinator::new();
        let mut serialized = combinator.serialize();
        assert_eq!(serialized, combinator.serialize());


        combinator.acquire(10, &vec![], &mut vec![]);
        combinator.update(11, &vec![], &vec![], &mut vec![]);
        serialized = combinator.serialize();

        assert_eq!(serialized, combinator.serialize_details());
    }

    // Combinator deserialization is correct
    #[test]
    fn deserialization_correct() {
        let mut combinator = AnytimeCombinator::new(
            Box::new(GetCombinator::new(
                Box::new(ThenCombinator::new(
                    Box::new(GiveCombinator::new(
                        Box::new(ScaleCombinator::new(
                            Box::new(AndCombinator::new(
                                Box::new(OrCombinator::new(
                                    Box::new(ZeroCombinator::new()),
                                    Box::new(OneCombinator::new()),
                                    0
                                )),
                                Box::new(TruncateCombinator::new(
                                    Box::new(OneCombinator::new()),
                                    10
                                ))
                            )),
                            None,
                            Some(5)
                        ))
                    )),
                    Box::new(OneCombinator::new())
                ))
            )),
            0
        );
        let mut anytime_acquisition_times = vec![(false, None)];
        combinator.acquire(10, &vec![None], &mut anytime_acquisition_times);
        combinator.update(11, &vec![None], &vec![], &mut anytime_acquisition_times);
        let serialized = combinator.serialize();

        let (_, deserialized) = deserialize_combinator(0, &serialized);

        assert_eq!(deserialized.serialize(), serialized);
    }

    // Attempting to deserialize an empty vector is not allowed.
    #[test]
    #[should_panic(expected = "Attempted to deserialize ill-formed serialized ContractCombinator.")]
    fn should_panic_if_deserializing_empty_vector() {
        deserialize_combinator(0, &vec![]);
    }
}