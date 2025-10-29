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
            .extend_ttl(&CACHE_KEY, base_ttl + bump, base_ttl + bump);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ineffective_extend_with_expression() {
        let env = Env::default();
        let contract = IneffectiveExtendTtlClient::new(
            &env,
            &env.register_contract(None, IneffectiveExtendTtl {}),
        );

        contract.store_cache_with_expression(&4242);
    }
}
