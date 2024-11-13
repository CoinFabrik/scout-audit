#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Map, Symbol};

#[contract]
pub struct MapStorage;

#[contractimpl]
impl MapStorage {
    pub fn store_map(e: Env, data: Map<Symbol, i32>) {
        e.storage()
            .persistent()
            .set(&Symbol::new(&e, "map_data"), &data);
    }

    pub fn get_map(e: Env) -> Map<Symbol, i32> {
        e.storage()
            .persistent()
            .get(&Symbol::new(&e, "map_data"))
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{symbol_short, Env, Map};

    #[test]
    fn test_map_storage() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, MapStorage);
        let client = MapStorageClient::new(&env, &contract_id);

        // When
        let mut test_map = Map::new(&env);
        test_map.set(symbol_short!("key1"), 1);
        test_map.set(symbol_short!("key2"), 2);
        client.store_map(&test_map);

        // Then
        let retrieved_map = client.get_map();
        assert_eq!(
            test_map.get(symbol_short!("key1")),
            retrieved_map.get(symbol_short!("key1"))
        );
        assert_eq!(
            test_map.get(symbol_short!("key2")),
            retrieved_map.get(symbol_short!("key2"))
        );
    }
}
