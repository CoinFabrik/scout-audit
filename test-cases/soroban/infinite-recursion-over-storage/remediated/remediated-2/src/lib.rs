#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

fn advance_once(depth: u32) -> u32 {
    depth + 1
}

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(_env: Env, depth: u32) -> u32 {
        advance_once(depth)
    }
}
