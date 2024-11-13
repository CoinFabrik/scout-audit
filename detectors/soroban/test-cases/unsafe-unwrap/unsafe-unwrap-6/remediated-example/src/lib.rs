#![no_std]
#![allow(clippy::unnecessary_literal_unwrap)]

use soroban_sdk::{contract, contracterror, contractimpl};

#[contract]
pub struct UnsafeUnwrap;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    CustomError = 1,
}

#[contractimpl]
impl UnsafeUnwrap {
    pub fn safe_unwrap(n: u64) -> u64 {
        let result = Self::non_zero_or_error(n);
        if result.is_err() {
            return 0;
        }
        let known_value = Some(1u64);
        let first_operation = known_value.unwrap().checked_mul(result.unwrap());
        if let Some(first_operation) = first_operation {
            return first_operation;
        }
        0
    }

    pub fn non_zero_or_error(n: u64) -> Result<u64, Error> {
        if n == 0 {
            return Err(Error::CustomError);
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use crate::UnsafeUnwrap;

    #[test]
    fn test_unwrap_zero() {
        // Given
        let test_value = 0;

        // When
        let result = UnsafeUnwrap::safe_unwrap(test_value);

        // Then
        assert_eq!(result, test_value);
    }

    #[test]
    fn test_unwrap_non_zero() {
        // Given
        let test_value = 100;

        // When
        let result = UnsafeUnwrap::safe_unwrap(test_value);

        // Then
        assert_eq!(result, test_value);
    }
}
