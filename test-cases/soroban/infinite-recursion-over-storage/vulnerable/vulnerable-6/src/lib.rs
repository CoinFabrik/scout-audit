#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

fn storage_helper(env: Env, depth: u32) -> u32 {
    env.storage().persistent().set(&COUNTER, &depth);
    depth
}

fn cycle_left(env: Env, depth: u32) -> u32 {
    cycle_right(env, depth + 1)
}

fn cycle_right(env: Env, depth: u32) -> u32 {
    let updated_depth = storage_helper(env.clone(), depth);
    cycle_left(env, updated_depth + 1)
}

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(env: Env, depth: u32) -> u32 {
        cycle_left(env, depth)
    }
}
