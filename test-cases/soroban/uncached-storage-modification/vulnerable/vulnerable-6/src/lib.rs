#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct UncachedStorageModificationMatch;

#[contractimpl]
impl UncachedStorageModificationMatch {
    pub fn match_test(env: Env) -> u32 {
        let key = Symbol::new(&env, "key");

        match env.storage().persistent().get::<Symbol, u32>(&key) {
            Some(mut val) => {
                val += 1;
            }
            None => {
                // Write barrier in another arm should not cancel the modification arm.
                env.storage().persistent().set(&key, &0u32);
            }
        }

        let val2: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        val2
    }
}
