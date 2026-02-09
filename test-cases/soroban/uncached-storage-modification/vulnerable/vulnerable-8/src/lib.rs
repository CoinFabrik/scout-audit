#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct UncachedStorageModificationTuple;

#[contractimpl]
impl UncachedStorageModificationTuple {
    pub fn tuple_test(env: Env) -> u32 {
        let key = Symbol::new(&env, "key");

        let (mut val, _): (u32, u32) = env.storage().persistent().get(&key).unwrap_or((0, 0));

        val += 1;

        let val2: (u32, u32) = env.storage().persistent().get(&key).unwrap_or((0, 0));

        val2.0
    }
}
