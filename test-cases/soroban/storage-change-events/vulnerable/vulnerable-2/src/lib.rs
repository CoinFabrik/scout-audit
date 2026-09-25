#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

fn recurse_left(env: Env, value: u32) {
    recurse_right(env, value + 1);
}

fn recurse_right(env: Env, value: u32) {
    env.storage().persistent().set(&COUNTER, &value);
    recurse_left(env, value + 1);
}

#[contract]
pub struct StorageChangeEvents;

#[contractimpl]
impl StorageChangeEvents {
    pub fn update(env: Env, value: u32) {
        recurse_left(env, value);
    }
}
