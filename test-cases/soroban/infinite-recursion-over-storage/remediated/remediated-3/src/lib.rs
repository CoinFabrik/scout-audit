#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

fn unreachable_cycle_left(env: Env, depth: u32) -> u32 {
    env.storage().instance().set(&COUNTER, &depth);
    unreachable_cycle_right(env, depth + 1)
}

fn unreachable_cycle_right(env: Env, depth: u32) -> u32 {
    unreachable_cycle_left(env, depth + 1)
}

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(env: Env, depth: u32) -> u32 {
        let current: u32 = env.storage().temporary().get(&COUNTER).unwrap_or(depth);
        current + 1
    }
}
