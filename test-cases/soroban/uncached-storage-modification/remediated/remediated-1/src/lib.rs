#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct UncachedStorageModification;

#[contractimpl]
impl UncachedStorageModification {
    pub fn remediated_fn(env: Env) -> u32 {
        let key = Symbol::new(&env, "key");

        // 1. Read from storage
        let mut val: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        // 2. Modify local variable
        val += 1;

        // 3. Write back (Write Barrier)
        env.storage().persistent().set(&key, &val);

        // 4. Re-read from storage (Safe because we wrote back)
        let val2: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        val2
    }
}
