#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod unprotected_mapping_operation {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct UnprotectedMappingOperation {
        balances: Mapping<AccountId, Balance>,
        another_mapping: Mapping<u128, AccountId>,
    }

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        TransferError,
        BalanceNotEnough,
    }

    impl UnprotectedMappingOperation {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                balances: Mapping::new(),
                another_mapping: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn this_should_not_trigger(&mut self, key: u128, value: AccountId) {
            self.another_mapping.insert(key, &value);
        }

        #[ink(message, payable)]
        pub fn deposit(&mut self, dest: AccountId) {
            let amount: Balance = self.env().transferred_value();
            if let Some(current_bal) = self.balances.get(dest) {
                self.balances.insert(dest, &(current_bal + amount));
            } else {
                self.balances.insert(dest, &amount);
            }
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance, from: AccountId) -> Result<(), Error> {
            let current_bal = self.balances.take(from).unwrap_or(0);
            if current_bal >= amount {
                self.balances.insert(from, &(current_bal - amount));
                self.env()
                    .transfer(from, current_bal)
                    .map_err(|_| Error::TransferError)
            } else {
                Err(Error::BalanceNotEnough)
            }
        }

        #[ink(message)]
        pub fn withdraw_all(&mut self, from: AccountId) -> Result<(), Error> {
            let current_bal = self.balances.get(from).unwrap_or(0);
            self.balances.remove(from);
            self.env()
                .transfer(from, current_bal)
                .map_err(|_| Error::TransferError)
        }
    }
}
