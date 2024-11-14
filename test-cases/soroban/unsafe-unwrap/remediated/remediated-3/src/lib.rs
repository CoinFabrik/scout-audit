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
    pub fn safe_unwrap(n: u64) -> u64 {
        let result = Self::non_zero_or_error(n);
        let mut return_value = 0;
        if result.is_ok() {
            return_value = result.unwrap();
        }
        return_value
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
