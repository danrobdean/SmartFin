use super::contract_combinator::{ ContractCombinator, Box, Vec };

// The scale combinator
pub struct ScaleCombinator {
    // The sub-combinator
    sub_combinator: Box<ContractCombinator>,
    // The observable index
    obs_index: Option<usize>,
    // The scale value
    scale_value: Option<i64>
}

// Method implementation for the scale combinator
impl ScaleCombinator {
    pub fn new(sub_combinator: Box<ContractCombinator>, obs_index: Option<usize>, scale_value: Option<i64>) -> ScaleCombinator {
        ScaleCombinator {
            sub_combinator,
            obs_index,
            scale_value
        }
    }
}

// Contract combinator implementation for the scale combinator
impl ContractCombinator for ScaleCombinator {
    fn get_value(&self, time: u32, or_choices: &Vec<Option<bool>>, obs_values: &Vec<Option<i64>>) -> i64 {
        match self.scale_value {
            Some(value) => value* self.sub_combinator.get_value(time, or_choices, obs_values),
            None => {
                match self.obs_index {
                    Some(index) => {
                        let obs_value = obs_values[index];
                        match obs_value {
                            Some(value) => value * self.sub_combinator.get_value(time, or_choices, obs_values),
                            None => panic!("Cannot get value of scale combinator with an undefined observable.")
                        }
                    },
                    None => panic!("Scale combinator has no scale value or observable index.")
                }
            }
        }
    }
}