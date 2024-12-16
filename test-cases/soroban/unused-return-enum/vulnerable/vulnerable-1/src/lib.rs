#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl};

#[contract]
pub struct UnusedReturnEnum;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// An overflow was produced.
    Overflow = 1,
}

#[contractimpl]
impl UnusedReturnEnum {
    /// Returns the percentage difference between two values.
    pub fn get_percentage_difference(balance1: u128, balance2: u128) -> Result<u128, Error> {
        let absolute_difference = balance1.abs_diff(balance2);
        let sum = balance1 + balance2;

        match 100u128.checked_mul(absolute_difference / sum) {
            Some(result) => result,
            None => panic!("Overflow"),
        };

        Err(Error::Overflow)
    }
}

#[cfg(test)]
mod tests {

    use soroban_sdk::Env;

    use crate::{Error, UnusedReturnEnum, UnusedReturnEnumClient};

    #[test]
    fn get_percentage_difference_panics() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnusedReturnEnum);
        let client = UnusedReturnEnumClient::new(&env, &contract_id);

        // When
        let value1 = 100u128;
        let value2 = 150u128;
        let result = client.try_get_percentage_difference(&value1, &value2);

        // Then
        assert_eq!(result, Err(Ok(Error::Overflow)));
    }
}
