#![cfg_attr(not(feature = "std"), no_std)]

#[allow(clippy::new_without_default)]
#[ink::contract]
mod dos_unbounded_operation {
    use ink::storage::Mapping;

    /// A payment to be made to an account.
    #[derive(Debug, scale::Decode, scale::Encode)]
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
        payees: Mapping<u128, Payee>,
        /// The next payee index.
        next_payee_ix: u128,
    }

    impl DosUnboundedOperation {
        /// Creates a new instance of the contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                payees: Mapping::new(),
                next_payee_ix: 0,
            }
        }

        /// Adds a new payee to the operation.
        #[ink(message, payable)]
        pub fn add_payee(&mut self) -> u128 {
            let address = self.env().caller();
            let value = self.env().transferred_value();
            let new_payee = Payee { address, value };

            self.payees.insert(self.next_payee_ix, &new_payee);
            self.next_payee_ix = self.next_payee_ix.checked_add(1).unwrap();

            // Return the index of the new payee
            self.next_payee_ix.checked_sub(1).unwrap()
        }

        /// Add n payees to the operation, used only for testing.
        #[ink(message, payable)]
        pub fn add_n_payees(&mut self, n: u128) -> u128 {
            let address = self.env().caller();
            let value = self.env().transferred_value().checked_div(n).unwrap();
            let new_payee = Payee { address, value };

            for _ in 0..n {
                self.payees.insert(self.next_payee_ix, &new_payee);
                self.next_payee_ix = self.next_payee_ix.checked_add(1).unwrap();
            }

            // Return the index of the last added payee
            self.next_payee_ix.checked_sub(1).unwrap()
        }

        /// Returns the payee at the given index.
        #[ink(message)]
        pub fn get_payee(&self, id: u128) -> Option<Payee> {
            self.payees.get(id)
        }

        /// Pays out all payees.
        #[ink(message)]
        pub fn pay_out(&mut self) {
            for i in 0..self.next_payee_ix {
                let payee = self.payees.get(i).unwrap();
                self.env().transfer(payee.address, payee.value).unwrap();
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
