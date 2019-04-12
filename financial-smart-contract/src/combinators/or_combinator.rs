use super::contract_combinator::{ ContractCombinator, latest_horizon, Box, Vec };

// The or combinator
pub struct OrCombinator {
    // The first sub-combinator
    sub_combinator0: Box<ContractCombinator>,
    // The second sub-combinator
    sub_combinator1: Box<ContractCombinator>,
    // The index of this or combinator in the contract with reference to all or combinators
    or_index: usize
}

// Method implementation for the or combinator
impl OrCombinator {
    pub fn new(sub_combinator0: Box<ContractCombinator>, sub_combinator1: Box<ContractCombinator>, or_index: usize) -> OrCombinator {
        OrCombinator {
            sub_combinator0,
            sub_combinator1,
            or_index
        }
    }
}

// Contract combinator implementation for the or combinator
impl ContractCombinator for OrCombinator {
    // Returns the latest of the two sub-horizons
    fn get_horizon(&self) -> Option<u32> {
        latest_horizon(self.sub_combinator0.get_horizon(), self.sub_combinator1.get_horizon())
    }

    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>) -> u64 {
        // If one sub-combinator has expired, choose the other
        if self.sub_combinator0.past_horizon(time) {
            self.sub_combinator1.get_value(time, or_choices)
        } else if self.sub_combinator1.past_horizon(time) {
            self.sub_combinator0.get_value(time, or_choices)
        } else {
            // If both sub-combinators can be acquired, use the provided choice, or panic if no choice has been provided
            match or_choices[self.or_index] {
                Some(true) => self.sub_combinator0.get_value(time, or_choices),
                Some(false) => self.sub_combinator1.get_value(time, or_choices),
                None => panic!("Cannot get value of OR combinator when neither sub-combinator has been chosen or has expired.")
            }
        }
    }
}