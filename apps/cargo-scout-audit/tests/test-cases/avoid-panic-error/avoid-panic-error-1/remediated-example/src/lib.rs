#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNTER");

#[contract]
pub struct AvoidPanicError;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    OverflowError = 1,
}

#[contractimpl]
impl AvoidPanicError {
    pub fn add(env: Env, value: u32) -> Result<u32, Error> {
        let storage = env.storage().instance();
        let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);
        match count.checked_add(value) {
            Some(value) => count = value,
            None => return Err(Error::OverflowError),
        }
        storage.set(&COUNTER, &count);
        storage.extend_ttl(100, 100);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::Env;

    use crate::{AvoidPanicError, AvoidPanicErrorClient, Error};

    #[test]
    fn add() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, AvoidPanicError);
        let client = AvoidPanicErrorClient::new(&env, &contract_id);

        // When
        let first_increment = client.try_add(&1);
        let second_increment = client.try_add(&2);
        let third_increment = client.try_add(&3);

        // Then
        assert_eq!(first_increment, Ok(Ok(1)));
        assert_eq!(second_increment, Ok(Ok(3)));
        assert_eq!(third_increment, Ok(Ok(6)));
    }

    #[test]
    fn overflow() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, AvoidPanicError);
        let client = AvoidPanicErrorClient::new(&env, &contract_id);

        // When
        let _max_value = client.try_add(&u32::MAX);
        let overflow = client.try_add(&1);

        // Then
        assert_eq!(overflow, Err(Ok(Error::OverflowError)));
    }
}
