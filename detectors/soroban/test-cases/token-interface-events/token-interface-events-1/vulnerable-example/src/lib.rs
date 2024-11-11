#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token, Address, Env, String,
};

use soroban_sdk::token::TokenInterface;

#[derive(Clone, Debug)]
#[contracttype]
pub struct TokenMetadata {
    pub decimals: u32,
    pub name: String,
    pub symbol: String,
    pub admin: Address,
}

#[derive(Clone, Default)]
#[contracttype]
pub struct AllowanceFromSpender {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Balance(Address),
    TokenMetadata,
    AllowanceFromSpender(Address, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VTError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
}

#[contract]
pub struct TokenInterfaceEvents;

#[contractimpl]
impl TokenInterfaceEvents {
    pub fn initialize(
        env: Env,
        admin: Address,
        decimals: u32,
        name: String,
        symbol: String,
    ) -> Result<(), VTError> {
        let current_token_metadata: Option<TokenMetadata> =
            env.storage().instance().get(&DataKey::TokenMetadata);
        if current_token_metadata.is_some() {
            return Err(VTError::AlreadyInitialized);
        } else {
            env.storage().instance().set(
                &DataKey::TokenMetadata,
                &TokenMetadata {
                    decimals,
                    name,
                    symbol,
                    admin,
                },
            );
        }

        Ok(())
    }

    pub fn get_metadata(env: Env) -> TokenMetadata {
        env.storage()
            .instance()
            .get(&DataKey::TokenMetadata)
            .unwrap()
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        Self::get_metadata(env.clone()).admin.require_auth();
        let previous_balance: i128 = env
            .clone()
            .storage()
            .instance()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::Balance(to), &(previous_balance + amount));
    }

    fn get_allowance(env: Env, from: Address, spender: Address) -> AllowanceFromSpender {
        env.storage()
            .instance()
            .get(&DataKey::AllowanceFromSpender(from, spender))
            .unwrap_or_default()
    }
}

#[contractimpl]
impl token::TokenInterface for TokenInterfaceEvents {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        let allowance = Self::get_allowance(env.clone(), from, spender);
        if allowance.expiration_ledger < env.ledger().sequence() {
            0
        } else {
            allowance.amount
        }
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        assert!(env.ledger().sequence() < expiration_ledger || amount == 0);
        env.storage().instance().set(
            &DataKey::AllowanceFromSpender(from.clone(), spender.clone()),
            &AllowanceFromSpender {
                amount,
                expiration_ledger,
            },
        );
    }

    fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_balance = Self::balance(env.clone(), from.clone());
        let to_balance = Self::balance(env.clone(), to.clone());
        assert!(from_balance >= amount);
        env.storage()
            .instance()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::Balance(to), &(to_balance + amount));
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        let spender_allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        assert!(spender_allowance >= amount);

        let from_balance = Self::balance(env.clone(), from.clone());
        let to_balance = Self::balance(env.clone(), to.clone());
        assert!(from_balance >= amount);
        env.storage()
            .instance()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));

        let mut allowance = Self::get_allowance(env.clone(), from.clone(), spender.clone());
        allowance.amount -= amount;

        env.storage()
            .instance()
            .set(&DataKey::AllowanceFromSpender(from, spender), &allowance);
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        let from_balance = Self::balance(env.clone(), from.clone());
        assert!(from_balance >= amount);
        env.storage()
            .instance()
            .set(&DataKey::Balance(from), &(from_balance - amount));
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        let spender_allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        assert!(spender_allowance >= amount);
        let from_balance = Self::balance(env.clone(), from.clone());
        assert!(from_balance >= amount);
        env.storage()
            .instance()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));

        let mut allowance = Self::get_allowance(env.clone(), from.clone(), spender.clone());
        allowance.amount -= amount;
        env.storage()
            .instance()
            .set(&DataKey::AllowanceFromSpender(from, spender), &allowance);
    }
    fn decimals(env: Env) -> u32 {
        Self::get_metadata(env).decimals
    }
    fn name(env: Env) -> String {
        Self::get_metadata(env).name
    }
    fn symbol(env: Env) -> String {
        Self::get_metadata(env).symbol
    }
}

#[cfg(test)]
mod test;
