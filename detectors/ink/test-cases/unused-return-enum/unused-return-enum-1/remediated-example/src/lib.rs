#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod unused_return_enum {

    #[ink(storage)]
    pub struct UnusedReturnEnum {}

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// An overflow was produced.
        Overflow,
    }

    impl UnusedReturnEnum {
        /// Creates a new instance of the contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Returns the percentage difference between two values.
        #[ink(message)]
        pub fn get_percentage_difference(
            &mut self,
            value1: Balance,
            value2: Balance,
        ) -> Result<Balance, Error> {
            let absolute_difference = value1.abs_diff(value2);
            let sum = value1 + value2;
            match 100u128.checked_mul(absolute_difference / sum) {
                Some(result) => Ok(result),
                None => Err(Error::Overflow),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn get_percentage_difference_panics() {
            // Arrange
            let mut contract = UnusedReturnEnum::new();
            let value1 = 100;
            let value2 = 150;

            // Act
            let result = contract.get_percentage_difference(value1, value2);

            // Assert
            assert_eq!(result, Ok(0));
        }
    }
}
