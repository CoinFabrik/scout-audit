#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
}

fn set_admin_internal(e: &Env, new_admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, new_admin);
}

#[contract]
pub struct MissingNewAdminAuth;

#[contractimpl]
impl MissingNewAdminAuth {
    pub fn initialize(e: Env, admin: Address) {
        e.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        set_admin_internal(&e, &new_admin);
    }
}
