#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[derive(Debug, Clone)]
#[contracttype]
pub struct CacheEntry {
    admin: Address,
}

const STATE: Symbol = symbol_short!("STATE");

#[contract]
pub struct InitInsteadOfConstructor;

#[contractimpl]
impl InitInsteadOfConstructor {
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&STATE, &admin);
    }
}
