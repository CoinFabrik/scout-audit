#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

fn dispatch(env: Env, depth: u32, toggle: bool) -> u32 {
    if toggle {
        grow_storage(env, depth)
    } else {
        dispatch(env, depth + 1, true)
    }
}

fn grow_storage(env: Env, depth: u32) -> u32 {
    env.storage().persistent().set(&COUNTER, &depth);
    dispatch(env, depth + 1, false)
}

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(env: Env, depth: u32) -> u32 {
        dispatch(env, depth, false)
    }
}
