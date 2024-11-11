#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod unprotected_mapping_operation {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct UnprotectedMappingOperation {
        balances: Mapping<AccountId, Balance>,
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
            }
        }

        #[ink(message, payable)]
        pub fn deposit(&mut self) {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();
            if let Some(current_bal) = self.balances.get(caller) {
                self.balances.insert(caller, &(current_bal + amount));
            } else {
                self.balances.insert(caller, &amount);
            }
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            let current_bal = self.balances.take(caller).unwrap_or(0);
            if current_bal >= amount {
                self.balances.insert(caller, &(current_bal - amount));
                self.env()
                    .transfer(caller, current_bal)
                    .map_err(|_| Error::TransferError)
            } else {
                Err(Error::BalanceNotEnough)
            }
        }

        #[ink(message)]
        pub fn withdraw_all(&mut self) -> Result<(), Error> {
            let caller = self.env().caller();
            let current_bal = self.balances.get(caller).unwrap_or(0);
            self.balances.remove(caller);
            self.env()
                .transfer(caller, current_bal)
                .map_err(|_| Error::TransferError)
        }
    }
}
