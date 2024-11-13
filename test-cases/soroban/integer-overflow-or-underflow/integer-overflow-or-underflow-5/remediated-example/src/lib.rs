#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct IntegerOverflowUnderflow;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    OverflowError = 1,
    UnderflowError = 2,
}

#[contractimpl]
impl IntegerOverflowUnderflow {
    const VALUE: Symbol = symbol_short!("VALUE");

    pub fn initialize(env: Env, value: i32) {
        env.storage().temporary().set(&Self::VALUE, &value);
    }

    pub fn neg(env: Env) -> Result<(), Error> {
        let current: i32 = env.storage().temporary().get(&Self::VALUE).unwrap_or(0);
        let new_value = match current.checked_neg() {
            Some(value) => value,
            None => return Err(Error::UnderflowError),
        };
        env.storage().temporary().set(&Self::VALUE, &new_value);
        Ok(())
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
    fn test_neg() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, IntegerOverflowUnderflow);
        let client = IntegerOverflowUnderflowClient::new(&env, &contract_id);

        // When
        client.initialize(&i32::MIN);
        let result = client.try_neg();

        // Then
        assert_eq!(result, Err(Ok(Error::UnderflowError)));
    }
}
