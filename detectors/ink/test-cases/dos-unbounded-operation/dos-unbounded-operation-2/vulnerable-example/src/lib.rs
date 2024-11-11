#![cfg_attr(not(feature = "std"), no_std)]
#[allow(clippy::new_without_default)]
#[ink::contract]
mod dos_unbounded_operation_2 {
    use ink::prelude::vec::Vec;
    /// A payment to be made to an account.
    #[derive(Debug, scale::Decode, scale::Encode, Clone, Copy)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Payee {
        /// The account to which the payment is to be made.
        pub address: AccountId,
        /// The amount to be paid.
        pub value: Balance,
    }

    #[ink(storage)]
    pub struct DosUnboundedOperation {
        /// The payees of the operation.
        payees: Vec<Payee>,
    }

    impl DosUnboundedOperation {
        /// Creates a new instance of the contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { payees: Vec::new() }
        }

        /// Adds a new payee to the operation.
        #[ink(message, payable)]
        pub fn add_payee(&mut self) -> u128 {
            let address = self.env().caller();
            let value = self.env().transferred_value();
            let new_payee = Payee { address, value };

            self.payees.push(new_payee);

            // Return the index of the new payee
            self.payees
                .len()
                .checked_sub(1)
                .unwrap()
                .try_into()
                .unwrap()
        }

        /// Add n payees to the operation, used only for testing.
        #[ink(message, payable)]
        pub fn add_n_payees(&mut self, n: u128) -> u128 {
            let address = self.env().caller();
            let value = self.env().transferred_value().checked_div(n).unwrap();
            let new_payee = Payee { address, value };

            for _ in 0..n {
                self.payees.push(new_payee);
            }

            // Return the index of the last added payee
            self.payees
                .len()
                .checked_sub(1)
                .unwrap()
                .try_into()
                .unwrap()
        }

        /// Returns the payee at the given index.
        #[ink(message)]
        pub fn get_payee(&self, id: u128) -> Option<Payee> {
            let payee = self.payees.get(usize::try_from(id).unwrap())?;
            Some(*payee)
        }

        /// Pays out all payees.
        #[ink(message)]
        pub fn pay_out(&mut self) {
            for payee in &self.payees {
                self.env().transfer(payee.address, payee.value).unwrap();
            }
        }

        ///Same as pay_out but using a different approach to iterate self.payees
        #[ink(message)]
        pub fn pay_out2(&mut self) {
            for id in 0..self.payees.len() {
                self.env()
                    .transfer(self.payees[id].address, self.payees[id].value)
                    .unwrap();
            }
        }

        /// Pays out a range of payees.
        #[ink(message)]
        pub fn pay_out_range(&mut self, n: u64, m: u64) {
            for id in n..m {
                self.env()
                    .transfer(
                        self.payees[id as usize].address,
                        self.payees[id as usize].value,
                    )
                    .unwrap();
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn constructor_works() {
            // Arrange
            let contract = DosUnboundedOperation::new();

            // Act
            let first_payee = contract.get_payee(0);

            // Assert
            assert!(first_payee.is_none());
        }

        #[ink::test]
        fn next_payee_advances() {
            // Arrange
            let mut contract = DosUnboundedOperation::new();

            // Act
            let first_payee_id = contract.add_payee();
            let second_payee_id = contract.add_payee();

            // Assert
            assert_eq!(first_payee_id, 0);
            assert_eq!(second_payee_id, 1);
        }

        #[ink::test]
        fn add_payee_works() {
            // Arrange
            let mut contract = DosUnboundedOperation::new();

            // Act
            let payee_id = contract.add_payee();
            let payee = contract.get_payee(payee_id).unwrap();

            // Assert
            assert_eq!(payee.address, AccountId::from([0x01; 32]));
            assert_eq!(payee.value, 0);
        }

        #[ink::test]
        fn add_n_payees_works() {
            // Arrange
            let mut contract = DosUnboundedOperation::new();

            // Act
            let payee_id = contract.add_n_payees(10);
            let payee = contract.get_payee(payee_id).unwrap();

            // Assert
            assert_eq!(payee.address, AccountId::from([0x01; 32]));
            assert_eq!(payee.value, 0);
        }
    }
}
