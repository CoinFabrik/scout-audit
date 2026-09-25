#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn transfer_from(_env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        let owner = from.clone();
        spender.require_auth();
        // Vulnerable: delegated spending directly requires auth from `from`.
        owner.require_auth();
        let _ = (to, amount);
    }
}
