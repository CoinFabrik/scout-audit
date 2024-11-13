#![no_std]
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct DosUnboundedOperation;

#[contractimpl]
impl DosUnboundedOperation {
    pub fn safe_loop_with_array() -> u64 {
        let mut sum = 0;
        let known_array = [0; 8];
        for i in 0..known_array.len() {
            sum += i;
        }
        sum as u64
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
        let count = client.safe_loop_with_array();

        // Then
        assert_eq!(count, 28);
    }
}
