#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contract]
pub struct UnnecessaryAdminParameter;

#[contracttype]
pub enum DataKey {
    Admin,
}

#[contractimpl]
impl UnnecessaryAdminParameter {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    // This function is minimal and intends to showcase the misuse of the admin parameter.
    pub fn set_admin(env: Env, new_admin: Address, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }
}

#[cfg(test)]
mod tests {
    use crate::{DataKey, UnnecessaryAdminParameter, UnnecessaryAdminParameterClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_vulnerable_initialize() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnnecessaryAdminParameter);
        let client = UnnecessaryAdminParameterClient::new(&env, &contract_id);

        // When
        let admin = soroban_sdk::Address::generate(&env);
        client.initialize(&admin);

        // Then
        let stored_admin: Address = env.as_contract(&contract_id, || {
            env.storage().instance().get(&DataKey::Admin).unwrap()
        });
        assert_eq!(stored_admin, admin);
    }

    #[test]
    fn test_vulnerable_set_admin() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnnecessaryAdminParameter);
        let client = UnnecessaryAdminParameterClient::new(&env, &contract_id);
        env.mock_all_auths();

        // When
        let admin = Address::generate(&env);
        client.initialize(&admin);
        let stored_admin: Address = env.as_contract(&contract_id, || {
            env.storage().instance().get(&DataKey::Admin).unwrap()
        });
        assert_eq!(stored_admin, admin);

        // Bad actor sets the admin to themselves
        let bad_actor = Address::generate(&env);
        client.set_admin(&bad_actor, &bad_actor);

        // Then
        let stored_admin: Address = env.as_contract(&contract_id, || {
            env.storage().instance().get(&DataKey::Admin).unwrap()
        });
        assert_eq!(stored_admin, bad_actor);
    }
}
