#![no_std]

use soroban_sdk::{contract, contractimpl, map, Env, Error, IntoVal, Map, TryIntoVal, Val};

#[contract]
pub struct UnsafeMapGet;

#[contractimpl]
impl UnsafeMapGet {
    pub fn get_map_with_different_values(env: Env, key: i32) -> Result<Option<i32>, Error> {
        let map: Map<Val, Val> = map![
            &env,
            (1i32.into_val(&env), 2i32.into_val(&env)),
            (3i32.into_val(&env), 4i64.into_val(&env)),
        ];
        let map: Val = map.into();
        let map: Map<i32, i32> = map.try_into_val(&env).unwrap();
        map.try_get(key).map_err(Error::from)
    }

    pub fn get_map_with_different_keys(env: Env, key: i32) -> Result<Option<i32>, Error> {
        let map: Map<Val, Val> = map![
            &env,
            (1i32.into_val(&env), 2i32.into_val(&env)),
            (3i64.into_val(&env), 4i32.into_val(&env)),
        ];
        let map: Val = map.into();
        let map: Map<i32, i32> = map.try_into_val(&env).unwrap();
        map.try_get(key).map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{
        xdr::{ScError, ScErrorCode},
        Env, Error,
    };

    use crate::{UnsafeMapGet, UnsafeMapGetClient};

    #[test]
    fn try_get_errors_on_different_values() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnsafeMapGet);
        let client = UnsafeMapGetClient::new(&env, &contract_id);

        // When
        let value_ok = client.try_get_map_with_different_values(&1);
        let value_err = client.try_get_map_with_different_values(&3);

        // Then
        assert_eq!(value_ok.unwrap(), Ok(Some(2)));
        assert_eq!(
            value_err.err().unwrap(),
            Ok(Error::from_scerror(ScError::Context(
                ScErrorCode::InvalidAction
            )))
        );
    }

    #[test]
    fn try_get_errors_on_different_keys() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnsafeMapGet);
        let client = UnsafeMapGetClient::new(&env, &contract_id);

        // When
        let key_ok = client.try_get_map_with_different_values(&1);
        let key_err = client.try_get_map_with_different_values(&3);

        // Then
        assert_eq!(key_ok.unwrap(), Ok(Some(2)));
        assert_eq!(
            key_err.err().unwrap(),
            Ok(Error::from_scerror(ScError::Context(
                ScErrorCode::InvalidAction
            )))
        );
    }
}
