#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

fn bounce_b(env: Env, depth: u32) -> u32 {
    env.storage().instance().set(&COUNTER, &depth);
    bounce_a(env, depth + 1)
}

fn bounce_a(env: Env, depth: u32) -> u32 {
    bounce_b(env, depth + 1)
}

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(env: Env, depth: u32) -> u32 {
        bounce_a(env, depth)
    }
}
