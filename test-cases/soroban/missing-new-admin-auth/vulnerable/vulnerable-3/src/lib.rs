#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Owner,
    NewOwner,
}

#[contract]
pub struct MissingNewOwnerAuth;

#[contractimpl]
impl MissingNewOwnerAuth {
    pub fn initialize(e: Env, owner: Address) {
        e.storage().instance().set(&DataKey::Owner, &owner);
    }

    pub fn set_owner(e: Env, nnw_owner: Address) {
        let owner: Address = e.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        e.storage().instance().set(&DataKey::NewOwner, &nnw_owner);
    }
}
