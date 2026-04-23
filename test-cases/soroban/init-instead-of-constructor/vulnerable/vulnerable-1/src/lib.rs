#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contract]
pub struct InitInsteadOfConstructor;

#[contractimpl]
impl InitInsteadOfConstructor {
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        // Store admin in the contract.
    }
}
