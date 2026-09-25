#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        // Vulnerable: delegated spending flows into allowance logic that still requires auth from `from`.
        Self::decrease_allowance(env, from, spender, amount);
        let _ = to;
    }

    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        let allowance_owner = from.clone();
        spender.require_auth();
        // Vulnerable: delegated burning should not require auth from the allowance owner.
        Self::decrease_allowance(env, allowance_owner, spender, amount);
    }

    pub fn decrease_allowance(_env: Env, from: Address, _spender: Address, _amount: i128) {
        from.require_auth();
    }
}
