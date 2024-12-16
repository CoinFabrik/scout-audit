#![no_std]
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct DivideBeforeMultiply;

#[contractimpl]
impl DivideBeforeMultiply {
    pub fn split_profit(percentage: u64, total_profit: u64) -> u64 {
        (percentage / 100) * total_profit
    }
}

#[cfg(test)]
mod tests {
    use crate::DivideBeforeMultiply;

    #[test]
    fn split_profit_works() {
        let result = DivideBeforeMultiply::split_profit(33, 100);
        assert_eq!(result, 0);
    }
}
