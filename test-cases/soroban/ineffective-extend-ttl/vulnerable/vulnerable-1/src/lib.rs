#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol};

#[derive(Debug, Clone)]
#[contracttype]
pub struct CacheEntry {
    data: u64,
    timestamp: u64,
}

const CACHE_KEY: Symbol = symbol_short!("CACHE");

#[contract]
pub struct IneffectiveExtendTtl;

#[contractimpl]
impl IneffectiveExtendTtl {
    pub fn store_cache(env: Env, data: u64, var1: u32) {
        let entry = CacheEntry {
            data,
            timestamp: env.ledger().timestamp(),
        };

        let storage = env.storage().temporary();
        storage.set(&CACHE_KEY, &entry);

        storage.extend_ttl(&CACHE_KEY, 1, 1);
        storage.extend_ttl(&CACHE_KEY, var1, var1);
        storage.extend_ttl(&CACHE_KEY, 2, 1);

        const CONST_1: u32 = 5;
        const CONST_2: u32 = 5;
        const CONST_3: u32 = 10;

        storage.extend_ttl(&CACHE_KEY, CONST_1, CONST_2);
        storage.extend_ttl(&CACHE_KEY, CONST_3, CONST_2);
    }
}
