#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol, Vec};

#[contract]
pub struct VectorStorage;

#[contractimpl]
impl VectorStorage {
    pub fn store_vector(e: Env, data: Vec<i32>) {
        e.storage()
            .persistent()
            .set(&Symbol::new(&e, "vector_data"), &data);
    }

    pub fn get_vector(e: Env) -> Vec<i32> {
        e.storage()
            .persistent()
            .get(&Symbol::new(&e, "vector_data"))
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{vec, Env};

    #[test]
    fn test_vector_storage() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, VectorStorage);
        let client = VectorStorageClient::new(&env, &contract_id);

        // When
        let test_vec = vec![&env, 1, 2, 3, 4, 5];
        client.store_vector(&test_vec);

        // Then
        let retrieved_vec = client.get_vector();
        assert_eq!(test_vec, retrieved_vec);
    }
}
