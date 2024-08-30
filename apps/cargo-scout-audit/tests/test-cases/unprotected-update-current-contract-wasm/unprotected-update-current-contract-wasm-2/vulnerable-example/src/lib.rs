#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
}

#[contract]
pub struct UpgradeableContract;

#[contractimpl]
impl UpgradeableContract {
    pub fn init(e: Env, admin: Address) {
        e.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn version() -> u32 {
        1
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        utils::upgrade(&e, new_wasm_hash);
    }
}

mod utils {
    use soroban_sdk::{BytesN, Env};

    pub fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
