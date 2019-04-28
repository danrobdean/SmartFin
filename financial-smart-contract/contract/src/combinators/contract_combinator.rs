extern crate pwasm_std;

pub use self::pwasm_std::{ Box, Vec, vec };

// The types of combinators
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
    pub fn deserialize_details(details_serialized: [i64; 2]) -> CombinatorDetails {
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
    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &Vec<Option<u32>>) -> i64;

    // Returns the common combinator details of the combinator
    fn get_combinator_details(&self) -> &CombinatorDetails;

    // Acquires the combinator, setting the acquisition time in the combinator details
    fn acquire(&mut self, time: u32, or_choices: &Vec<Option<bool>>, anytime_acquisition_times: &mut Vec<Option<u32>>);

    // Updates the combinator, returning the current balance to be paid from the holder to the counter-party
    fn update(&mut self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>, anytime_acquisition_times: &mut Vec<Option<u32>>) -> i64;

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

// Unit tests
#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

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
}