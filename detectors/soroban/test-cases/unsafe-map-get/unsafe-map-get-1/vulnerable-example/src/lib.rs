#![no_std]

use soroban_sdk::{contract, contractimpl, map, Env, IntoVal, Map, TryIntoVal, Val};

#[contract]
pub struct UnsafeMapGet;

#[contractimpl]
impl UnsafeMapGet {
    pub fn get_from_map(env: Env) -> Option<i32> {
        let map: Map<Val, Val> = map![&env, (1i32.into_val(&env), 2i64.into_val(&env))];
        let map: Val = map.into();
        let map: Map<i32, i32> = map.try_into_val(&env).unwrap();
        map.get(1)
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::Env;

    use crate::{UnsafeMapGet, UnsafeMapGetClient};

    #[test]
    #[should_panic(expected = "ConversionError")]
    fn test_insert_balances() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnsafeMapGet);
        let client = UnsafeMapGetClient::new(&env, &contract_id);

        // When
        let _value = client.get_from_map();

        // Then

        // Test should panic
    }
}
