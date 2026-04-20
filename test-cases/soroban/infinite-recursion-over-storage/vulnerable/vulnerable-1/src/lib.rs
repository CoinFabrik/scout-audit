#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn recurse(env: Env, depth: u32) -> u32 {
        env.storage().persistent().set(&COUNTER, &depth);
        Self::recurse(env, depth + 1)
    }
}
