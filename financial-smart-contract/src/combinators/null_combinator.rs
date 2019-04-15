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
    fn get_value(&self, _time: u32, _or_choices: &Vec<Option<bool>>, _obs_values: &Vec<Option<i64>>) -> i64 {
        panic!("Attempted to get value of a null combinator.")
    }

    fn get_horizon(&self) -> Option<u32> {
        panic!("Attempted to get horizon of a null combinator.")
    }

    fn past_horizon(&self, _time: u32) -> bool {
        panic!("Attempted to check if past horizon of a null combinator.")
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::super::{ ContractCombinator, NullCombinator };
    use super::super::contract_combinator::vec;

    // Calling get_value on null-combinator is not allowed
    #[test]
    #[should_panic(expected = "Attempted to get value of a null combinator.")]
    fn should_panic_if_getting_value_of_null_combinator() {
        let null_combinator = NullCombinator::new();

        null_combinator.get_value(0, &vec![], &vec![]);
    }

    // Calling get_horizon on null-combinator is not allowed
    #[test]
    #[should_panic(expected = "Attempted to get horizon of a null combinator.")]
    fn should_panic_if_getting_horizon_of_null_combinator() {
        let null_combinator = NullCombinator::new();

        null_combinator.get_horizon();        
    }

    // Calling past_horizon on null-combinator is not allowed
    #[test]
    #[should_panic(expected = "Attempted to check if past horizon of a null combinator.")]
    fn should_panic_if_checking_if_past_horizon_of_null_combinator() {
        let null_combinator = NullCombinator::new();

        null_combinator.past_horizon(0);        
    }
}