#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct UncachedStorageModificationLoop;

#[contractimpl]
impl UncachedStorageModificationLoop {
    pub fn loop_test(env: Env, count: u32) -> u32 {
        let key = Symbol::new(&env, "key");

        let mut val: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        // Loop modification
        for _ in 0..count {
            val += 1;
        }

        // Re-read
        // Since loop body is visited, `val` should be marked as modified.
        let val2: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        val2
    }
}
