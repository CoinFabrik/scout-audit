#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct IntegerOverflowUnderflow;

#[contractimpl]
impl IntegerOverflowUnderflow {
    const VALUE: Symbol = symbol_short!("VALUE");

    pub fn initialize(env: Env, value: u32) {
        env.storage().temporary().set(&Self::VALUE, &value);
    }

    pub fn mul(env: Env, value: u32) {
        let current: u32 = env.storage().temporary().get(&Self::VALUE).unwrap_or(0);
        let new_value = current * value;
        env.storage().temporary().set(&Self::VALUE, &new_value);
    }

    pub fn get(env: Env) -> u32 {
        env.storage().temporary().get(&Self::VALUE).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_mul_overflow() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, IntegerOverflowUnderflow);
        let client = IntegerOverflowUnderflowClient::new(&env, &contract_id);

        // When
        client.initialize(&u32::MAX);
        client.mul(&2);

        // Then
        // Panic
    }
}
