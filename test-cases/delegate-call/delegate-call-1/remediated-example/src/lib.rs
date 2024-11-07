#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod delegate_call {

    #[ink(storage)]
    pub struct DelegateCall {
        admin: AccountId,
        addresses: [AccountId; 3],
        percent1: u128,
        percent2: u128,
        percent3: u128,
        target: Hash,
    }

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        NotEnoughMoney,
        DelegateCallFailed,
        NotAnAdmin,
        TransferFailed,
    }

    impl DelegateCall {
        /// Instantiates a new DelegateCall contract
        #[ink(constructor)]
        pub fn new(
            address1: AccountId,
            address2: AccountId,
            address3: AccountId,
            percent1: u128,
            percent2: u128,
            percent3: u128,
            target: Hash,
        ) -> Self {
            Self {
                admin: Self::env().caller(),
                addresses: [address1, address2, address3],
                percent1,
                percent2,
                percent3,
                target,
            }
        }

        /// Delegates the fee calculation and pays the results to the corresponding addresses
        #[ink(message, payable)]
        pub fn ask_payouts(&mut self) -> Result<(), Error> {
            let amount = self.env().transferred_value();

            let result = ink::env::call::build_call::<ink::env::DefaultEnvironment>()
                .delegate(self.target)
                .exec_input(
                    ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                        ink::selector_bytes!("payouts"),
                    ))
                    .push_arg(amount),
                )
                .returns::<(Balance, Balance, Balance)>()
                .try_invoke()
                .map_err(|_| Error::DelegateCallFailed)?
                .map_err(|_| Error::DelegateCallFailed)?;

            let total = result.0 + result.1 + result.2;
            if total > amount {
                return Err(Error::NotEnoughMoney);
            }

            self.env()
                .transfer(self.addresses[0], result.0)
                .map_err(|_| Error::TransferFailed)?;
            self.env()
                .transfer(self.addresses[1], result.1)
                .map_err(|_| Error::TransferFailed)?;
            self.env()
                .transfer(self.addresses[2], result.2)
                .map_err(|_| Error::TransferFailed)?;

            Ok(())
        }

        /// Sets the target codehash for the delegated call
        #[ink(message)]
        pub fn set_target(&mut self, new_target: Hash) -> Result<(), Error> {
            if self.admin != self.env().caller() {
                return Err(Error::NotAnAdmin);
            }
            self.target = new_target;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::env::test::DefaultAccounts;

        use super::*;

        #[ink::test]
        fn constructor_works() {
            // Arrange
            let accounts: DefaultAccounts<ink::env::DefaultEnvironment> =
                ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let hash: Hash = [0x01; 32].into();

            // Act
            let contract = DelegateCall::new(
                accounts.bob,
                accounts.charlie,
                accounts.eve,
                10,
                20,
                70,
                hash,
            );

            // Assert
            assert_eq!(contract.admin, accounts.alice);
            assert_eq!(
                contract.addresses,
                [accounts.bob, accounts.charlie, accounts.eve]
            );
            assert_eq!(contract.percent1, 10);
            assert_eq!(contract.percent2, 20);
            assert_eq!(contract.percent3, 70);
        }

        #[ink::test]
        fn set_target_fails_if_not_called_by_admin() {
            // Arrange
            let accounts: DefaultAccounts<ink::env::DefaultEnvironment> =
                ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let hash: Hash = [0x01; 32].into();
            let mut contract = DelegateCall::new(
                accounts.bob,
                accounts.charlie,
                accounts.eve,
                10,
                20,
                70,
                hash,
            );

            // Act
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let result = contract.set_target([0x02; 32].into());

            // Assert
            assert!(result.is_err());
        }

        #[ink::test]
        fn set_target_works_called_by_admin() {
            // Arrange
            let accounts: DefaultAccounts<ink::env::DefaultEnvironment> =
                ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let hash: Hash = [0x01; 32].into();
            let mut contract = DelegateCall::new(
                accounts.bob,
                accounts.charlie,
                accounts.eve,
                10,
                20,
                70,
                hash,
            );

            // Act
            let result = contract.set_target([0x02; 32].into());

            // Assert
            assert!(result.is_ok());
            assert_eq!(contract.target, [0x02; 32].into());
        }
    }
}
