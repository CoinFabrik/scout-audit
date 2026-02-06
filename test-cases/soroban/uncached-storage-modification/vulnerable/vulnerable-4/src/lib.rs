#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct UncachedStorageModificationComplex1;

fn helper_modify(val: &mut u32) {
    *val += 1;
}

#[contractimpl]
impl UncachedStorageModificationComplex1 {
    pub fn complex_test(env: Env) -> u32 {
        let key = Symbol::new(&env, "key");

        let mut val: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        // Modification via helper function (mutable reference)
        helper_modify(&mut val);

        // Re-read
        // Detector should catch that 'val' was modified via the helper call.
        let val2: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        val2
    }
}
