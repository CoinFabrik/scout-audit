#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod delegate_call {
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct DelegateCall {
        admin: AccountId,
        balances: Mapping<AccountId, Balance>,
    }

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        NotAnAdmin,
        DelegateCallFailed,
    }

    impl DelegateCall {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                admin: Self::env().caller(),
                balances: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn get_admin(&self) -> AccountId {
            self.admin
        }

        #[ink(message, payable)]
        pub fn change_admin(
            &mut self,
            target: Hash,
            new_admin: AccountId,
        ) -> Result<AccountId, Error> {
            if self.admin != self.env().caller() {
                return Err(Error::NotAnAdmin);
            }

            let res = build_call::<DefaultEnvironment>()
                .delegate(target)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("change_admin")))
                        .push_arg(new_admin),
                )
                .returns::<AccountId>()
                .try_invoke()
                .map_err(|_| Error::DelegateCallFailed)?
                .map_err(|_| Error::DelegateCallFailed)?;

            Ok(res)
        }
    }
}
