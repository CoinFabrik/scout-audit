#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod divide_before_multiply {

    #[ink(storage)]
    pub struct DivideBeforeMultiply {}

    impl DivideBeforeMultiply {
        /// Creates a new DivideBeforeMultiply contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Calculates the profit for a given percentage of the total profit.
        #[ink(message)]
        pub fn split_profit(&self, percentage: u64, total_profit: u64) -> u64 {
            (percentage / 100) * total_profit
        }

        // same as split_profit but using checked functions
        #[ink(message)]
        pub fn checked_split_profit(&self, percentage: u64, total_profit: u64) -> Option<u64> {
            percentage.checked_div(100)?.checked_mul(total_profit)
        }

        // same as split_profit but using both normal operations and checked functions
        #[ink(message)]
        pub fn hybrid_split_profit(&self, percentage: u64, total_profit: u64) -> Option<u64> {
            Some(percentage.checked_div(100)? * total_profit)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn split_profit_precision() {
            let contract = DivideBeforeMultiply::new();
            assert_eq!(contract.split_profit(33, 100), 0);
        }
    }
}
