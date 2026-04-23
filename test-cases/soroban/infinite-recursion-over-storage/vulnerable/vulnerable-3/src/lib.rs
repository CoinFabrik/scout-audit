#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

fn prepare(env: Env, depth: u32) -> u32 {
    recurse_left(env, depth + 1)
}

fn recurse_left(env: Env, depth: u32) -> u32 {
    recurse_middle(env, depth + 1)
}

fn recurse_middle(env: Env, depth: u32) -> u32 {
    env.storage().temporary().set(&COUNTER, &depth);
    recurse_right(env, depth + 1)
}

fn recurse_right(env: Env, depth: u32) -> u32 {
    recurse_left(env, depth + 1)
}

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(env: Env, depth: u32) -> u32 {
        prepare(env, depth)
    }
}
