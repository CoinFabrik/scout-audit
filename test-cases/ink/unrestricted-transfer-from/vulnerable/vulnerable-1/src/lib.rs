#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod unrestricted_transfer_from {
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };
    use ink::prelude::string::String;
    use ink::prelude::vec;
    use openbrush::contracts::psp22::PSP22Error as ob_PSP22Error;
    use openbrush::contracts::psp22::PSP22Ref;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PSP22Error {
        /// Custom error type for cases if writer of traits added own restrictions
        Custom(String),
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        /// Returned if recipient's address is zero.
        ZeroRecipientAddress,
        /// Returned if sender's address is zero.
        ZeroSenderAddress,
        /// Returned if safe transfer check fails
        SafeTransferCheckFailed(String),
    }

    #[derive(PartialEq, Eq, Debug, Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub enum Status {
        Created,
        Locked,
        Unlocked,
        Released,
        Refunded,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidAmount,
        CallerMustBeBuyer,
        CallerMustBeSeller,
        CallerMustBeArbiter,
        StatusMustBeCreated,
        StatusMustBeUnlocked,
        StatusMustBeLocked,
        #[allow(clippy::enum_variant_names)]
        PSP22Error(PSP22Error),
    }

    #[ink(storage)]
    pub struct UnrestrictedTransferFrom {
        buyer: AccountId,
        seller: AccountId,
        arbiter: AccountId,
        amount: Balance,
        psp22_address: AccountId,
        status: Status,
    }

    impl UnrestrictedTransferFrom {
        #[ink(constructor)]
        pub fn new(
            buyer: AccountId,
            seller: AccountId,
            arbiter: AccountId,
            psp22_address: AccountId,
            amount: Balance,
        ) -> Self {
            Self {
                buyer,
                seller,
                arbiter,
                psp22_address,
                amount,
                status: Status::Created,
            }
        }

        #[ink(message)]
        pub fn deposit(&mut self, from: AccountId) -> Result<(), Error> {
            if self.env().caller() != self.buyer {
                Err(Error::CallerMustBeBuyer)
            } else if self.status != Status::Created {
                return Err(Error::StatusMustBeCreated);
            } else {
                // 0x54b3c76e selector comes from https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md
                let res = PSP22Ref::transfer_from(
                    &self.psp22_address,
                    from,
                    self.env().account_id(),
                    self.amount,
                    vec![],
                );

                if res.is_ok() {
                    self.status = Status::Locked;
                }
                return res.map_err(|err| match err {
                    ob_PSP22Error::Custom(err_msg) => {
                        Error::PSP22Error(PSP22Error::Custom(err_msg))
                    }
                    ob_PSP22Error::InsufficientBalance => {
                        Error::PSP22Error(PSP22Error::InsufficientBalance)
                    }
                    ob_PSP22Error::InsufficientAllowance => {
                        Error::PSP22Error(PSP22Error::InsufficientAllowance)
                    }
                    ob_PSP22Error::ZeroRecipientAddress => {
                        Error::PSP22Error(PSP22Error::ZeroRecipientAddress)
                    }
                    ob_PSP22Error::ZeroSenderAddress => {
                        Error::PSP22Error(PSP22Error::ZeroSenderAddress)
                    }
                    ob_PSP22Error::SafeTransferCheckFailed(err_msg) => {
                        Error::PSP22Error(PSP22Error::SafeTransferCheckFailed(err_msg))
                    }
                });
            }
        }

        #[ink(message)]
        pub fn deposit_by_buildcall(&mut self, from: AccountId) -> Result<(), Error> {
            if self.env().caller() != self.buyer {
                Err(Error::CallerMustBeBuyer)
            } else if self.status != Status::Created {
                Err(Error::StatusMustBeCreated)
            } else {
                // 0x54b3c76e selector comes from https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md
                let call_params = build_call::<DefaultEnvironment>()
                    .exec_input(
                        ExecutionInput::new(Selector::new(ink::selector_bytes!(
                            "PSP22::transfer_from"
                        )))
                        .push_arg(from)
                        .push_arg(self.env().account_id())
                        .push_arg(self.amount)
                        .push_arg([0u8]),
                    )
                    .returns::<Result<(), PSP22Error>>()
                    .call(self.psp22_address)
                    .params();
                let res = self
                    .env()
                    .invoke_contract(&call_params)
                    .unwrap_or_else(|err| panic!("Err {err:?}"))
                    .unwrap_or_else(|err| panic!("LangErr {err:?}"))
                    .map_err(Error::PSP22Error);
                if res.is_ok() {
                    self.status = Status::Locked;
                }
                res
            }
        }

        #[ink(message)]
        pub fn unlock(&mut self) -> Result<(), Error> {
            if self.env().caller() != self.arbiter {
                Err(Error::CallerMustBeArbiter)
            } else if self.status != Status::Locked {
                return Err(Error::StatusMustBeLocked);
            } else {
                self.status = Status::Unlocked;
                Ok(())
            }
        }

        #[ink(message)]
        pub fn release(&mut self) -> Result<(), Error> {
            if self.env().caller() != self.seller {
                Err(Error::CallerMustBeSeller)
            } else if self.status != Status::Unlocked {
                return Err(Error::StatusMustBeUnlocked);
            } else {
                // 0x54b3c76e selector comes from https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md
                let call_params = build_call::<DefaultEnvironment>()
                    .exec_input(
                        ExecutionInput::new(Selector::new([0xdb, 0x20, 0xf9, 0xf5]))
                            .push_arg(self.env().caller())
                            .push_arg(self.amount)
                            .push_arg([0u8]),
                    )
                    .returns::<Result<(), PSP22Error>>()
                    .call(self.psp22_address)
                    .params();
                let res = self
                    .env()
                    .invoke_contract(&call_params)
                    .unwrap_or_else(|err| panic!("Err {err:?}"))
                    .unwrap_or_else(|err| panic!("LangErr {err:?}"))
                    .map_err(Error::PSP22Error);
                if res.is_ok() {
                    self.status = Status::Released;
                }
                return res;
            }
        }

        #[ink(message)]
        pub fn refund(&mut self) -> Result<(), Error> {
            if self.env().caller() != self.arbiter {
                Err(Error::CallerMustBeArbiter)
            } else if self.status != Status::Locked {
                return Err(Error::StatusMustBeLocked);
            } else {
                let call_params = build_call::<DefaultEnvironment>()
                    .exec_input(
                        ExecutionInput::new(Selector::new(ink::selector_bytes!(
                            "PSP22::transfer_from"
                        )))
                        .push_arg(self.env().account_id())
                        .push_arg(self.buyer)
                        .push_arg(self.amount)
                        .push_arg([0u8]),
                    )
                    .returns::<Result<(), PSP22Error>>()
                    .call(self.psp22_address)
                    .params();
                let res = self
                    .env()
                    .invoke_contract(&call_params)
                    .unwrap_or_else(|err| panic!("Err {err:?}"))
                    .unwrap_or_else(|err| panic!("LangErr {err:?}"))
                    .map_err(Error::PSP22Error);
                if res.is_ok() {
                    self.status = Status::Refunded;
                }
                Ok(())
            }
        }
    }
}
