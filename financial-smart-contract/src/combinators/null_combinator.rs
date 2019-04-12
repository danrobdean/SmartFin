use super::contract_combinator::{ ContractCombinator, Vec };

// The null combinator - for use when the contract has no combinators (e.g. pre-initialisation)
pub struct NullCombinator {}

// Method implementation of the null combinator
impl NullCombinator {
    // Constructor
    pub fn new() -> NullCombinator {
        NullCombinator {}
    }
}

// Contract combinator implementation of the null combinator
impl ContractCombinator for NullCombinator {
    fn get_value(&self, _time: u32, _or_choices: &Vec<Option<bool>>) -> u64 {
        panic!("Attempted to get value of a null combinator.")
    }
}