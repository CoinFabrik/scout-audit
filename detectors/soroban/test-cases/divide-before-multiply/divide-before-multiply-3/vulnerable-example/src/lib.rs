#![no_std]
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct DivideBeforeMultiply;

#[contractimpl]
impl DivideBeforeMultiply {
    pub fn hybrid_split_profit(percentage: u64, total_profit: u64) -> Option<u64> {
        Some(percentage.checked_div(100)? * total_profit)
    }
}

#[cfg(test)]
mod tests {
    use crate::DivideBeforeMultiply;

    #[test]
    fn hybrid_split_profit_works() {
        let result = DivideBeforeMultiply::hybrid_split_profit(33, 100);
        assert_eq!(result.unwrap(), 0);
    }
}
