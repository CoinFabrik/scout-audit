#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct TokenInterfaceInferenceVulnerable2;

#[contractimpl]
impl TokenInterfaceInferenceVulnerable2 {
    pub fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        0
    }

    pub fn approve(
        _env: Env,
        _from: Address,
        _spender: Address,
        _amount: i128,
        _expiration_ledger: u32,
    ) {
    }

    pub fn balance(_env: Env, _id: Address) -> i128 {
        0
    }

    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}

    pub fn burn(_env: Env, _from: Address, _amount: i128) {}

    pub fn decimals(_env: Env) -> u32 {
        7
    }
}
