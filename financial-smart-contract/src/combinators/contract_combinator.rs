extern crate pwasm_std;

pub use self::pwasm_std::{Box, Vec};

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
            None => false,
        }
    }

    // Returns the value of the combinator if acquired at the given time
    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64;
}

// Returns the earliest of the given horizons
pub fn earliest_horizon(horizon0: Option<u32>, horizon1: Option<u32>) -> Option<u32> {
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
pub fn latest_horizon(horizon0: Option<u32>, horizon1: Option<u32>) -> Option<u32> {
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


#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    // Earliest horizon should return the earliest horizon correctly.
    #[test]
    fn earliest_horizon_returns_earliest_horizon() {
        let horizon0 = Some(2);
        let horizon1 = Some(1);

        assert_eq!(earliest_horizon(horizon0, horizon1), horizon1);
    }

    // Latest horizon should return the latest horizon correctly.
    #[test]
    fn latest_horizon_returns_latest_horizon() {
        let horizon0 = Some(2);
        let horizon1 = Some(1);

        assert_eq!(latest_horizon(horizon0, horizon1), horizon0);
    }

    // Earliest horizon should return a horizon if one is None.
    #[test]
    fn earliest_horizon_returns_non_none_value() {
        let horizon0 = Some(2);
        let horizon1 = None;

        assert_eq!(earliest_horizon(horizon0, horizon1), horizon0);
    }

    // Latest horizon should return None if one is None.
    #[test]
    fn latest_horizon_returns_none_value() {
        let horizon0 = Some(2);
        let horizon1 = None;

        assert_eq!(latest_horizon(horizon0, horizon1), horizon1);
    }
}