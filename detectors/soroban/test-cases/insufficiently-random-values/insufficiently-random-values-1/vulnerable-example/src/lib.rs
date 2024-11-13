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
    pub fn generate_random_value_timestamp(env: Env, max_val: u64) -> Result<u64, Error> {
        if max_val == 0 {
            Err(Error::MaxValZero)
        } else {
            let val = env.ledger().timestamp() % max_val;
            Ok(val)
        }
    }
    pub fn generate_random_value_sequence(env: Env, max_val: u32) -> Result<u32, Error> {
        if max_val == 0 {
            Err(Error::MaxValZero)
        } else {
            let val = env.ledger().sequence() % max_val;
            Ok(val)
        }
    }
}

#[cfg(test)]
mod test {
    use soroban_sdk::{
        testutils::{Ledger, LedgerInfo},
        Env,
    };

    use crate::{Contract, ContractClient};

    #[test]
    fn random_value_sequence() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        // When
        let mut ledger = LedgerInfo {
            sequence_number: 0,
            ..Default::default()
        };
        env.ledger().set(ledger.clone());
        let first_random_value = client.generate_random_value_sequence(&10);

        ledger.sequence_number = 1;
        env.ledger().set(ledger.clone());
        let second_random_value = client.generate_random_value_sequence(&10);

        ledger.sequence_number = 11;
        env.ledger().set(ledger.clone());
        let third_random_value = client.generate_random_value_sequence(&10);

        // Then
        assert_eq!(first_random_value, 0);
        assert_eq!(second_random_value, 1);
        assert_eq!(third_random_value, 1);
    }

    #[test]
    fn random_value_timestamp() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        // When
        let mut ledger = LedgerInfo {
            timestamp: 0,
            ..Default::default()
        };
        env.ledger().set(ledger.clone());
        let first_random_value = client.generate_random_value_timestamp(&10);

        ledger.timestamp = 1;
        env.ledger().set(ledger.clone());
        let second_random_value = client.generate_random_value_timestamp(&10);

        ledger.timestamp = 11;
        env.ledger().set(ledger.clone());
        let third_random_value = client.generate_random_value_timestamp(&10);

        // Then
        assert_eq!(first_random_value, 0);
        assert_eq!(second_random_value, 1);
        assert_eq!(third_random_value, 1);
    }
}
