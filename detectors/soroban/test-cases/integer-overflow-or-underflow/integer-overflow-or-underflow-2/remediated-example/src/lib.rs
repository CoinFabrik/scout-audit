#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct IntegerOverflowUnderflow;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    UnderflowError = 1,
}

#[contractimpl]
impl IntegerOverflowUnderflow {
    const VALUE: Symbol = symbol_short!("VALUE");

    pub fn initialize(env: Env, value: u32) {
        env.storage().temporary().set(&Self::VALUE, &value);
    }

    pub fn sub(env: Env, value: u32) -> Result<(), Error> {
        let current: u32 = env.storage().temporary().get(&Self::VALUE).unwrap_or(0);
        let new_value = match current.checked_sub(value) {
            Some(value) => value,
            None => return Err(Error::UnderflowError),
        };
        env.storage().temporary().set(&Self::VALUE, &new_value);
        Ok(())
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
    fn test_sub_underflow() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, IntegerOverflowUnderflow);
        let client = IntegerOverflowUnderflowClient::new(&env, &contract_id);

        // When
        client.initialize(&0);
        let result = client.try_sub(&1);

        // Then
        assert_eq!(result, Err(Ok(Error::UnderflowError)));
    }
}
