#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod unsafe_unwrap {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct UnsafeUnwrap {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
    }

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Account has insufficient balance.
        InsufficientBalance,
    }

    impl UnsafeUnwrap {
        /// Creates a new instance of the contract.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::new();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);

            Self {
                total_supply,
                balances,
            }
        }

        /// Returns the balance of a given account.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap()
        }

        /// Transfers tokens from the caller to the given `to` account.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: Balance) -> Result<(), Error> {
            let from_balance = self.balances.get(self.env().caller()).unwrap();

            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let new_from_balance = from_balance - amount;
            self.balances.insert(self.env().caller(), &new_from_balance);

            let new_to_balance = self.balance_of(to) + amount;
            self.balances.insert(to, &new_to_balance);

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::env::test;
        use ink::env::DefaultEnvironment;

        use super::*;

        #[ink::test]
        fn constructor_works() {
            // Arrange
            let initial_balance = 100;

            // Act
            let contract = UnsafeUnwrap::new(initial_balance);

            // Assert
            let alice_balance: Balance =
                contract.balance_of(test::default_accounts::<DefaultEnvironment>().alice);
            assert_eq!(alice_balance, initial_balance);
        }

        #[ink::test]
        #[should_panic]
        fn balance_of_unwraps_on_unknown_account() {
            // Arrange
            let initial_balance = 100;
            let contract = UnsafeUnwrap::new(initial_balance);

            // Act
            contract.balance_of(test::default_accounts::<DefaultEnvironment>().bob);
        }

        #[ink::test]
        #[should_panic]
        fn transfer_unwraps_to_unknown_account() {
            // Arrange
            let initial_balance = 100;
            let transfer_amount = 20;
            let mut contract = UnsafeUnwrap::new(initial_balance);

            // Act
            contract
                .transfer(
                    test::default_accounts::<DefaultEnvironment>().bob,
                    transfer_amount,
                )
                .unwrap();
        }
    }
}
