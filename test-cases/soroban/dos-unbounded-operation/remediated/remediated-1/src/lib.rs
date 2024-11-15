#![no_std]
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct DosUnboundedOperation;

const FIXED_COUNT: u64 = 1000;

#[contractimpl]
impl DosUnboundedOperation {
    pub fn restricted_loop_with_const() -> u64 {
        let mut sum = 0;
        for i in 0..FIXED_COUNT {
            sum += i;
        }
        sum
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::Env;

    use crate::{DosUnboundedOperation, DosUnboundedOperationClient};

    #[test]
    fn test_for_loop() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, DosUnboundedOperation);
        let client = DosUnboundedOperationClient::new(&env, &contract_id);

        // When
        let count = client.restricted_loop_with_const();

        // Then
        assert_eq!(count, 499500);
    }
}
