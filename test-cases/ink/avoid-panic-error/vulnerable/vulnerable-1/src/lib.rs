#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod panic_error {

    #[ink(storage)]
    pub struct PanicError {
        /// Stored value.
        value: u32,
    }

    impl PanicError {
        /// Creates a new instance of the contract.
        #[ink(constructor)]
        pub fn new(value: u32) -> Self {
            Self { value }
        }

        /// Increments the stored value by the given amount.
        #[ink(message)]
        pub fn add(&mut self, value: u32) -> Result<(), ()> {
            match self.value.checked_add(value) {
                Some(v) => self.value = v,
                None => panic!("Overflow error"),
            };
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn constructor_works() {
            // Arrange
            let value = 42;

            // Act
            let contract = PanicError::new(42);

            // Assert
            assert_eq!(contract.value, value);
        }

        #[ink::test]
        #[should_panic(expected = "Overflow error")]
        fn add_panics() {
            // Arrange
            let mut contract = PanicError::new(u32::MAX);

            // Act
            contract.add(1);

            // Assert - handled by the `should_panic` attribute
        }
    }
}
