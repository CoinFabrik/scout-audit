#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn test(percentage: u64, total_profit: u64) -> u64 {
        (percentage / 100) * total_profit
    }
}
