#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Map, Symbol};

const STORAGE_KEY: Symbol = symbol_short!("DATA");

#[contract]
pub struct RemediatedMapContract;

#[contractimpl]
impl RemediatedMapContract {
    // FIXED: Access is controlled via require_auth
    pub fn save_user_data(env: Env, user: Address, data: u64) {
        user.require_auth(); // Fix: Auth is required before mutation

        let mut map: Map<Address, u64> = env
            .storage()
            .instance()
            .get(&STORAGE_KEY)
            .unwrap_or(Map::new(&env));

        map.set(user, data);

        env.storage().instance().set(&STORAGE_KEY, &map);
    }
}
