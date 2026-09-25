#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        spend_allowance(env, from, spender, amount);
        let _ = to;
        // do stuff..
    }

    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        spend_allowance(env, from, spender, amount);
        // do stuff..
    }
}

fn spend_allowance(_env: Env, _from: Address, _spender: Address, _amount: i128) {
    // do stuff..
}
