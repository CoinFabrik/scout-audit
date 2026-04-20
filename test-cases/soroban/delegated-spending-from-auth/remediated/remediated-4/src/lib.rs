#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        Self::authorize_spender(spender.clone());
        Self::decrease_allowance_internal(env, from, spender, amount);
        let _ = to;
        // do stuff..
    }

    fn authorize_spender(spender: Address) {
        let delegated_spender = spender.clone();
        delegated_spender.require_auth();
    }

    fn decrease_allowance_internal(_env: Env, _from: Address, _spender: Address, _amount: i128) {
        // do stuff..
    }
}
