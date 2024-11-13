#![no_std]
use scout_utils::scout_allow;
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct UnnecessaryLintAllow;

#[contractimpl]
#[scout_allow(assert_violation)]
impl UnnecessaryLintAllow {
    pub fn assert_if_greater_than_10(value: u128) -> bool {
        assert!(value <= 10);
        true
    }
}
