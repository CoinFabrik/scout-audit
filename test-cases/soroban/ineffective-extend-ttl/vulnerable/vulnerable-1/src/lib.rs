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

        env.storage().temporary().set(&CACHE_KEY, &entry);
        let ttl = 100_000;
        env.storage().temporary().extend_ttl(&CACHE_KEY, ttl, ttl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ineffective_extend() {
        let env = Env::default();
        let contract = IneffectiveExtendTtlClient::new(
            &env,
            &env.register_contract(None, IneffectiveExtendTtl {}),
        );

        contract.store_cache(&42);
    }
}
