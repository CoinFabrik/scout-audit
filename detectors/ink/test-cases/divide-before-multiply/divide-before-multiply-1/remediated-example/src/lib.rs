#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod divide_before_multiply {

    #[ink(storage)]
    pub struct DivideBeforeMultiply {}

    impl DivideBeforeMultiply {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn split_profit(&self, percentage: u64, total_profit: u64) -> u64 {
            (percentage * total_profit) / 100
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn split_profit_precision() {
            let contract = DivideBeforeMultiply::new();
            assert_eq!(contract.split_profit(33, 100), 33);
        }
    }
}
