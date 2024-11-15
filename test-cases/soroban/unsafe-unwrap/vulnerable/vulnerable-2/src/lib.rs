#![no_std]
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
    pub fn unwrap(n: u64) -> (u64, u64) {
        let result = Self::non_zero_or_error(n);
        (result.unwrap(), result.unwrap())
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
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: CustomError")]
    fn test_unwrap_zero() {
        // Given
        let test_value = 0;

        // When
        let _result = UnsafeUnwrap::unwrap(test_value);

        // Then
        // The test should panic
    }

    #[test]
    fn test_unwrap_non_zero() {
        // Given
        let test_value = 100;

        // When
        let result = UnsafeUnwrap::unwrap(test_value);

        // Then
        assert_eq!(result.0, test_value);
        assert_eq!(result.1, test_value);
    }
}
