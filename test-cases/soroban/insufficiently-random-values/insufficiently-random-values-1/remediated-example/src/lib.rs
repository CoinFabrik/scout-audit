#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    MaxValZero = 1,
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn generate_random_value(env: Env, max_val: u64) -> Result<u64, Error> {
        if max_val == 0 {
            Err(Error::MaxValZero)
        } else {
            let val = env.prng().gen_range(0..max_val);
            Ok(val)
        }
    }
}

#[cfg(test)]
mod test {
    use soroban_sdk::Env;

    use crate::{Contract, ContractClient};

    #[test]
    fn random_value_sequence() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        // When
        let first_random_value = client.generate_random_value(&10);
        let second_random_value = client.generate_random_value(&100);
        let third_random_value = client.generate_random_value(&1000);
        let fourth_random_value = client.generate_random_value(&10000);
        let fifth_random_value = client.generate_random_value(&100000);

        // Then
        assert!(first_random_value < 10);
        assert!(second_random_value < 100);
        assert!(third_random_value < 1000);
        assert!(fourth_random_value < 10000);
        assert!(fifth_random_value < 100000);
    }
}
