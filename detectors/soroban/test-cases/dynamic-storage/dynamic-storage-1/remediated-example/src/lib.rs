#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Vec};

#[contract]
pub struct VectorStorage;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    VecElement(u32),
}

#[contractimpl]
impl VectorStorage {
    pub fn store_vector(e: Env, data: Vec<i32>) {
        for (i, value) in data.iter().enumerate() {
            let key = DataKey::VecElement(i as u32);
            e.storage().persistent().set(&key, &value);
        }
    }

    pub fn get_vector(e: Env) -> Vec<i32> {
        let mut result = Vec::new(&e);
        let mut i = 0;

        while let Some(value) = VectorStorage::get_element(e.clone(), i) {
            result.push_back(value);
            i += 1;
        }

        result
    }

    pub fn get_element(e: Env, index: u32) -> Option<i32> {
        let key = DataKey::VecElement(index);
        e.storage().persistent().get(&key)
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

        assert_eq!(client.get_element(&2), Some(3));
        assert_eq!(client.get_element(&5), None);
    }
}
