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
    pub fn store_cache(env: Env, data: u64) {
        let entry = CacheEntry {
            data,
            timestamp: env.ledger().timestamp(),
        };

        env.storage().temporary().set(&CACHE_KEY, &entry);
        env.storage()
            .temporary()
            .extend_ttl(&CACHE_KEY, 50_000, 100_000);
    }

    pub fn store_persistent(env: Env, key: Symbol, value: u64) {
        env.storage().persistent().set(&key, &value);
        env.storage().persistent().extend_ttl(&key, 10_000, 100_000);
    }

    pub fn init_instance(env: Env, config: u64) {
        env.storage().instance().set(&CACHE_KEY, &config);
        env.storage().instance().extend_ttl(50_000, 200_000);
    }

    pub fn update_cache(env: Env, data: u64) {
        let mut entry = env
            .storage()
            .temporary()
            .get::<_, CacheEntry>(&CACHE_KEY)
            .unwrap_or(CacheEntry {
                data: 0,
                timestamp: 0,
            });

        entry.data = data;
        entry.timestamp = env.ledger().timestamp();
        env.storage().temporary().set(&CACHE_KEY, &entry);
        env.storage().temporary().extend_ttl(&CACHE_KEY, 1000, 5000);
    }

    pub fn store_expiring_cache(env: Env, data: u64, expires_at: u64) {
        let entry = CacheEntry {
            data,
            timestamp: expires_at,
        };

        env.storage().temporary().set(&CACHE_KEY, &entry);

        env.storage()
            .temporary()
            .extend_ttl(&CACHE_KEY, 10_000, 50_000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_extend() {
        let env = Env::default();
        let contract = EffectiveExtendTtlClient::new(
            &env,
            &env.register_contract(None, EffectiveExtendTtl {}),
        );

        contract.store_cache(&42);
        contract.store_persistent(&symbol_short!("KEY1"), &100);
        contract.init_instance(&999);
    }
}
