#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn transfer_from(_env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        let _ = (from, to, amount);
        // do stuff..
    }
}
