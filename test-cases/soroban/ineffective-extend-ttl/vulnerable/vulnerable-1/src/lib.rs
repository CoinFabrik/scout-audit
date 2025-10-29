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
    pub fn store_cache_with_expression(env: Env, data: u64) {
        let now = env.ledger().timestamp();
        let entry = CacheEntry {
            data,
            timestamp: now,
        };

        env.storage().temporary().set(&CACHE_KEY, &entry);

        let base_ttl = ((now as u32) % 10_000) + 1;
        let bump = ((data as u32) % 1_000) + 1;

        env.storage()
            .temporary()
            .extend_ttl(&CACHE_KEY, base_ttl + bump, base_ttl + bump * 2);
    }

    pub fn store_cache(env: Env, data: u64) {
        let entry = CacheEntry {
            data,
            timestamp: env.ledger().timestamp(),
        };

        env.storage().temporary().set(&CACHE_KEY, &entry);
        let ttl = 100_000;
        env.storage().temporary().extend_ttl(&CACHE_KEY, ttl, ttl);
    }

    pub fn store_persistent(env: Env, key: Symbol, value: u64) {
        env.storage().persistent().set(&key, &value);
        env.storage().persistent().extend_ttl(&key, 10_000, 10_000);
    }

    pub fn init_instance(env: Env, config: u64) {
        env.storage().instance().set(&CACHE_KEY, &config);
        let ttl_value = 50_000;
        env.storage().instance().extend_ttl(ttl_value, ttl_value);
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
        env.storage().temporary().extend_ttl(&CACHE_KEY, 1000, 1000);
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

        contract.store_cache_with_expression(&4242);
        contract.store_cache(&42);
        contract.store_persistent(&symbol_short!("KEY1"), &100);
        contract.init_instance(&999);
    }
}
