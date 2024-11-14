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

    // This function is minimal and intends to showcase the correct retrieval of the admin.
    pub fn set_admin(env: Env, new_admin: Address) {
        // Initialize has already set the admin, so we can retrieve it directly.
        let current_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        current_admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }
}

#[cfg(test)]
mod tests {

    use soroban_sdk::Env;
    use soroban_sdk::{testutils::Address as _, Address};

    use crate::{DataKey, UnnecessaryAdminParameter, UnnecessaryAdminParameterClient};

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
    fn test_remediated_set_admin_authorized() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnnecessaryAdminParameter);
        let client = UnnecessaryAdminParameterClient::new(&env, &contract_id);

        // When
        let admin = Address::generate(&env);
        client.initialize(&admin);

        // Set new admin
        let new_admin = Address::generate(&env);
        env.mock_all_auths();
        client.set_admin(&new_admin);

        // Then
        let stored_admin: Address = env.as_contract(&contract_id, || {
            env.storage().instance().get(&DataKey::Admin).unwrap()
        });
        assert_eq!(stored_admin, new_admin);
    }

    #[test]
    #[should_panic(expected = "Unauthorized function call for address")]
    fn test_remediated_set_admin_unauthorized() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnnecessaryAdminParameter);
        let client = UnnecessaryAdminParameterClient::new(&env, &contract_id);

        // When
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let bad_actor = Address::generate(&env);

        // No authorization mocking, simulating an unauthorized attempt

        // Attempt to change admin without proper authorization
        client.set_admin(&bad_actor);

        // Then
        // This point should not be reached due to the expected panic
    }
}
