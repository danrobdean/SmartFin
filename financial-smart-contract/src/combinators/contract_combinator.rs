// API for combinators
pub trait ContractCombinator {
    // Returns the horizon of the combinator, or -1 if none exists
    fn get_horizon(&self) -> Option<i32> {
        None
    }

    // Returns whether the given time is beyond the combinator's horizon or not
    fn past_horizon(&self, time: i32) -> bool {
        match self.get_horizon() {
            Some(t) => t < time,
            None => false,
        }
    }

    // Returns the value of the combinator when acquired at the given time
    fn acquire(&self, time: i32) -> i32;
}

// Returns the earliest of the given horizons
pub fn earliest_horizon(horizon0: Option<i32>, horizon1: Option<i32>) -> Option<i32> {
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
pub fn latest_horizon(horizon0: Option<i32>, horizon1: Option<i32>) -> Option<i32> {
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