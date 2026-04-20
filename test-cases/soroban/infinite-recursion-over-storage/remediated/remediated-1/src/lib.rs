#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

fn write_once(env: Env, depth: u32) -> u32 {
    env.storage().persistent().set(&COUNTER, &depth);
    depth
}

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(env: Env, depth: u32) -> u32 {
        write_once(env, depth)
    }
}
