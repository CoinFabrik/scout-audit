#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn transfer_from(_env: Env, spender: Address, _from: Address, _to: Address, _amount: i128) {
        spender.require_auth();
        // do stuff..
    }

    pub fn burn_from(_env: Env, spender: Address, _from: Address, _amount: i128) {
        let delegated_spender = spender.clone();
        delegated_spender.require_auth();
        // do stuff..
    }

    pub fn decrease_allowance(_env: Env, from: Address, _spender: Address, _amount: i128) {
        from.require_auth();
        // do stuff..
    }
}
