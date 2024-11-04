#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract {

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn test(&self, percentage: u64, total_profit: u64) -> u64 {
            (percentage / 100) * total_profit
        }
    }
}
