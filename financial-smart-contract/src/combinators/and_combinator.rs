use super::contract_combinator::{ ContractCombinator, latest_horizon };

// The and combinator
pub struct AndCombinator<'a, T: ContractCombinator, U: ContractCombinator> {
    // The first sub-combinator
    sub_combinator0: &'a T,
    // The second sub-combinator
    sub_combinator1: &'a U
}

// Method implementation for the and combinator
impl<'a, T: ContractCombinator, U: ContractCombinator> AndCombinator<'a, T, U> {
    pub fn new(sub_combinator0: &'a T, sub_combinator1: &'a U) -> AndCombinator<'a, T, U> {
        AndCombinator {
            sub_combinator0,
            sub_combinator1
        }
    }
}

// Contract combinator implementation for the and combinator
impl<'a, T: ContractCombinator, U: ContractCombinator> ContractCombinator for AndCombinator<'a, T, U> {
    // Returns the latest of the two sub-horizons
    fn get_horizon(&self) -> Option<i32> {
        latest_horizon(self.sub_combinator0.get_horizon(), self.sub_combinator1.get_horizon())
    }

    fn acquire(&self, time: i32) -> i32 {
        self.sub_combinator0.acquire(time) + self.sub_combinator1.acquire(time)
    }
}