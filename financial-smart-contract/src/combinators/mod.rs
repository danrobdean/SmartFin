// Import combinators
mod contract_combinator;
mod null_combinator;
mod zero_combinator;
mod one_combinator;
mod and_combinator;
mod or_combinator;
mod truncate_combinator;
mod scale_combinator;
mod give_combinator;
mod then_combinator;
mod get_combinator;

// Re-export combinators
pub use self::contract_combinator::ContractCombinator;
pub use self::null_combinator::NullCombinator;
pub use self::zero_combinator::ZeroCombinator;
pub use self::one_combinator::OneCombinator;
pub use self::and_combinator::AndCombinator;
pub use self::or_combinator::OrCombinator;
pub use self::truncate_combinator::TruncateCombinator;
pub use self::scale_combinator::ScaleCombinator;
pub use self::give_combinator::GiveCombinator;
pub use self::then_combinator::ThenCombinator;
pub use self::get_combinator::GetCombinator;