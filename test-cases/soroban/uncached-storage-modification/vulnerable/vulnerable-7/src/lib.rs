#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct UncachedStorageModificationIfLet;

#[contractimpl]
impl UncachedStorageModificationIfLet {
    pub fn if_let_test(env: Env) -> u32 {
        let key = Symbol::new(&env, "key");

        if let Some(mut val) = env.storage().persistent().get::<Symbol, u32>(&key) {
            val += 1;
        }

        let val2: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        val2
    }
}
