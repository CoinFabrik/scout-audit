#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        let allowance_owner = from.clone();
        spender.require_auth();
        Self::consume_allowance(env, allowance_owner, spender, amount);
    }

    pub fn consume_allowance(env: Env, from: Address, spender: Address, amount: i128) {
        Self::decrease_allowance(env, from, spender, amount);
    }

    pub fn decrease_allowance(_env: Env, from: Address, _spender: Address, _amount: i128) {
        let owner = from.clone();
        // Vulnerable: this helper eventually requires auth from `from`, defeating delegated spending.
        owner.require_auth();
    }
}
