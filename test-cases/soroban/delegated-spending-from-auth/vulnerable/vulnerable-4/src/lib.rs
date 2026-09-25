#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        Self::authorize_spender(spender.clone());
        // Vulnerable: delegated spending routes into a helper that still requires auth from `from`.
        Self::decrease_allowance(env, from, spender, amount);
        let _ = to;
    }

    fn authorize_spender(spender: Address) {
        let delegated_spender = spender.clone();
        delegated_spender.require_auth();
    }

    fn decrease_allowance(_env: Env, from: Address, _spender: Address, _amount: i128) {
        let allowance_owner = from.clone();
        // Vulnerable: helper requires auth from the allowance owner.
        allowance_owner.require_auth();
    }
}
