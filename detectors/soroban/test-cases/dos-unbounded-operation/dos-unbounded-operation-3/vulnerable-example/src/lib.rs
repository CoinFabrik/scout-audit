#![no_std]
use soroban_sdk::{contract, contractimpl, BytesN};

#[contract]
pub struct DosUnboundedOperation;

#[contractimpl]
impl DosUnboundedOperation {
    pub fn unsafe_loop_with_array(unknown_array: BytesN<8>) -> u32 {
        let mut sum = 0;
        for i in 0..unknown_array.len() {
            sum += i;
        }
        sum
    }
}

#[cfg(test)]
mod tests {

    use soroban_sdk::{BytesN, Env};

    use crate::{DosUnboundedOperation, DosUnboundedOperationClient};

    #[test]
    fn test_for_loop() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, DosUnboundedOperation);
        let client = DosUnboundedOperationClient::new(&env, &contract_id);

        // When
        let unknown_array = BytesN::from_array(&env, &[0; 8]);
        let count = client.unsafe_loop_with_array(&unknown_array);

        // Then
        assert_eq!(count, 28);
    }
}
