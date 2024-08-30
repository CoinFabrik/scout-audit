#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNTER");

#[contract]
pub struct AvoidPanicError;

#[contractimpl]
impl AvoidPanicError {
    pub fn add(env: Env, value: u32) -> u32 {
        let storage = env.storage().instance();
        let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);
        match count.checked_add(value) {
            Some(value) => count = value,
            None => panic!("Overflow error"),
        }
        storage.set(&COUNTER, &count);
        storage.extend_ttl(100, 100);
        count
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::Env;

    use crate::{AvoidPanicError, AvoidPanicErrorClient};

    #[test]
    fn add() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, AvoidPanicError);
        let client = AvoidPanicErrorClient::new(&env, &contract_id);

        // When
        let first_increment = client.add(&1);
        let second_increment = client.add(&2);
        let third_increment = client.add(&3);

        // Then
        assert_eq!(first_increment, 1);
        assert_eq!(second_increment, 3);
        assert_eq!(third_increment, 6);
    }

    #[test]
    #[should_panic(expected = "Overflow error")]
    fn overflow() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, AvoidPanicError);
        let client = AvoidPanicErrorClient::new(&env, &contract_id);

        // When
        client.add(&u32::MAX);
        client.add(&1);

        // Then
        // panic
    }
}
