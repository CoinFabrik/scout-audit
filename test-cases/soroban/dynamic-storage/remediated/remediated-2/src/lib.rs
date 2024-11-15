#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Map, Symbol};

#[contract]
pub struct MapStorage;

#[contractimpl]
impl MapStorage {
    pub fn store_map(e: Env, data: Map<Symbol, i32>) {
        data.iter().for_each(|(key, value)| {
            e.storage().persistent().set(&key, &value);
        });
    }

    pub fn get_key(e: Env, key: Symbol) -> i32 {
        e.storage().persistent().get(&key).unwrap()
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
        let key1 = client.get_key(&symbol_short!("key1"));
        let key2 = client.get_key(&symbol_short!("key2"));
        assert_eq!(test_map.get(symbol_short!("key1")).unwrap(), key1);
        assert_eq!(test_map.get(symbol_short!("key2")).unwrap(), key2);
    }
}
