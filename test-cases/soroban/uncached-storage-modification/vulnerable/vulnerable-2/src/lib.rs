#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct UncachedStorageModificationBranching;

#[contractimpl]
impl UncachedStorageModificationBranching {
    pub fn branch_test(env: Env, condition: bool) -> u32 {
        let key = Symbol::new(&env, "key");

        // 1. Read
        let mut val: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        // 2. Branching modification
        if condition {
            val += 1;
        } else {
            // No modification
        }

        // 3. Re-read
        // The detector should conservatively assume 'val' MIGHT be modified (merge logic)
        // So checking against 'key' again should trigger the lint.
        let val2: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        val2
    }
}
