#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct IntegerOverflowUnderflow;

#[contractimpl]
impl IntegerOverflowUnderflow {
    const VALUE: Symbol = symbol_short!("VALUE");

    pub fn initialize(env: Env, value: i32) {
        env.storage().temporary().set(&Self::VALUE, &value);
    }

    pub fn neg(env: Env) {
        let current: i32 = env.storage().temporary().get(&Self::VALUE).unwrap_or(0);
        let new_value = -current;
        env.storage().temporary().set(&Self::VALUE, &new_value);
    }

    pub fn get(env: Env) -> i32 {
        env.storage().temporary().get(&Self::VALUE).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn test_neg() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, IntegerOverflowUnderflow);
        let client = IntegerOverflowUnderflowClient::new(&env, &contract_id);

        // When
        client.initialize(&i32::MIN);
        client.neg();

        // Then
        // Panic
    }
}
