#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype};

#[contract]
pub struct AvoidCoreMemForget;

#[contracttype]
#[derive(Eq, PartialEq)]
pub struct WithoutCopy {
    pub a: u64,
    pub b: u64,
}

impl Drop for WithoutCopy {
    fn drop(&mut self) {
        // Prevent clippy warning
    }
}

#[contractimpl]
impl AvoidCoreMemForget {
    pub fn forget_something(n: WithoutCopy) -> u64 {
        core::mem::forget(n);
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_forget_something() {
        let test_value: WithoutCopy = WithoutCopy { a: 80, b: 60 };

        let result = AvoidCoreMemForget::forget_something(test_value);

        assert_eq!(result, 0);
    }
}
