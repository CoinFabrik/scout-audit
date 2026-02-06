#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Map, Symbol};

const STORAGE_KEY: Symbol = symbol_short!("DATA");

#[contract]
pub struct VulnerableMapContract;

#[contractimpl]
impl VulnerableMapContract {
    // VULNERABLE: Allows unbounded growth of the Map without authorization.
    pub fn save_user_data(env: Env, user: Address, data: u64) {
        let mut map: Map<Address, u64> = env
            .storage()
            .instance()
            .get(&STORAGE_KEY)
            .unwrap_or(Map::new(&env));

        // Linter should trigger here on `set`
        map.set(user, data);

        env.storage().instance().set(&STORAGE_KEY, &map);
    }
}
