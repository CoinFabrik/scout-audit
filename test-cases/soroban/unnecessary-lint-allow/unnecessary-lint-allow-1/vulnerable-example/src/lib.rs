#![no_std]
use scout_utils::scout_allow;
use soroban_sdk::{contract, contracterror, contractimpl};

#[contract]
pub struct UnnecessaryLintAllow;

#[contracterror]
#[derive(Copy, Clone)]
pub enum AssertError {
    GreaterThan10 = 1,
}

#[contractimpl]
#[scout_allow(assert_violation)]
impl UnnecessaryLintAllow {
    pub fn assert_if_greater_than_10(value: u128) -> Result<bool, AssertError> {
        if value <= 10 {
            Ok(true)
        } else {
            Err(AssertError::GreaterThan10)
        }
    }
}
