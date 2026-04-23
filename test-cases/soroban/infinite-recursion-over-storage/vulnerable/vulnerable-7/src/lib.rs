#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

fn recurse_without_storage(depth: u32) -> u32 {
    recurse_without_storage(depth + 1)
}

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(_env: Env, depth: u32) -> u32 {
        recurse_without_storage(depth)
    }
}
