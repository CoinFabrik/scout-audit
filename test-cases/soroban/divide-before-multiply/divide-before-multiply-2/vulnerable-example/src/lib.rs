#![no_std]
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct DivideBeforeMultiply;

#[contractimpl]
impl DivideBeforeMultiply {
    pub fn checked_split_profit(percentage: u64, total_profit: u64) -> Option<u64> {
        percentage.checked_div(100)?.checked_mul(total_profit)
    }
}

#[cfg(test)]
mod tests {
    use crate::DivideBeforeMultiply;

    #[test]
    fn checked_split_profit_works() {
        let result = DivideBeforeMultiply::checked_split_profit(33, 100);
        assert_eq!(result.unwrap(), 0);
    }
}
