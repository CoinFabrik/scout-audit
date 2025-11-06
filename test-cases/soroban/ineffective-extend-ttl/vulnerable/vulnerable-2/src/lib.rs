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
    pub fn store_cache(env: Env, data: u64) {
        let entry = CacheEntry {
            data,
            timestamp: env.ledger().timestamp(),
        };

        let storage = env.storage().temporary();
        storage.set(&CACHE_KEY, &entry);
        storage.extend_ttl(&CACHE_KEY, 5 + 5, 7 + 3);
        storage.extend_ttl(&CACHE_KEY, 10 + 5, 5 + 5);
    }
}
