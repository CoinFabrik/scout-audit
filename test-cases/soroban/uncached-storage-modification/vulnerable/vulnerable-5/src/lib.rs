#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Symbol};

#[contracttype]
pub struct Data {
    pub count: u32,
}

impl Data {
    pub fn increment(&mut self) {
        self.count += 1;
    }
}

#[contract]
pub struct UncachedStorageModificationComplex2;

#[contractimpl]
impl UncachedStorageModificationComplex2 {
    pub fn method_test(env: Env) -> u32 {
        let key = Symbol::new(&env, "key");

        let mut data: Data = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Data { count: 0 });

        // Modification via method call
        data.increment();

        // Re-read
        let data2: Data = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Data { count: 0 });

        data2.count
    }
}
