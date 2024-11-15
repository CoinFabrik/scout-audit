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
    pub fn safe_unwrap(n: u64) -> (u64, u64) {
        // We get the same element twice for demonstration purposes
        let result_1 = Self::non_zero_or_error(n);
        let result_2 = Self::non_zero_or_error(n);
        if result_1.is_err() || result_2.is_err() {
            return (0, 0);
        }
        (result_1.unwrap(), result_2.unwrap())
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
        assert_eq!(result.0, test_value);
        assert_eq!(result.1, test_value);
    }

    #[test]
    fn test_unwrap_non_zero() {
        // Given
        let test_value = 100;

        // When
        let result = UnsafeUnwrap::safe_unwrap(test_value);

        // Then
        assert_eq!(result.0, test_value);
        assert_eq!(result.1, test_value);
    }
}
