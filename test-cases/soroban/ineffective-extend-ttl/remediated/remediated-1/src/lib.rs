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
pub struct EffectiveExtendTtl;

#[contractimpl]
impl EffectiveExtendTtl {
    pub fn store_cache(env: Env, data: u64, var1: u32, var2: u32) {
        let entry = CacheEntry {
            data,
            timestamp: env.ledger().timestamp(),
        };

        let storage = env.storage().temporary();
        storage.set(&CACHE_KEY, &entry);

        storage.extend_ttl(&CACHE_KEY, 1, 2);
        storage.extend_ttl(&CACHE_KEY, var1, var2);
        const OFFSET: u32 = 3;
        storage.extend_ttl(&CACHE_KEY, var1, var1 + OFFSET);
        storage.extend_ttl(&CACHE_KEY, var1 + OFFSET, var2 + OFFSET);

        const CONST_LEFT: u32 = 5;
        const CONST_RIGHT: u32 = 9;
        storage.extend_ttl(&CACHE_KEY, CONST_LEFT, CONST_RIGHT);
    }
}
