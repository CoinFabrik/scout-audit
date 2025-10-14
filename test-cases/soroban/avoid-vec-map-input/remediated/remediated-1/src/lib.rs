#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Addresses,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ValidationError {
    DuplicateAddress = 1,
}

#[contract]
pub struct AvoidVecMapInputRemediated;

#[contractimpl]
impl AvoidVecMapInputRemediated {
    pub fn add_address(env: Env, address: Address) -> Result<(), ValidationError> {
        let mut stored = load_addresses(&env);
        let len = stored.len();
        let mut i = 0;
        while i < len {
            if stored.get(i).unwrap() == address {
                return Err(ValidationError::DuplicateAddress);
            }
            i += 1;
        }
        stored.push_back(address);
        save_addresses(&env, &stored);
        Ok(())
    }

    pub fn list_addresses(env: Env) -> Vec<Address> {
        load_addresses(&env)
    }
}

fn load_addresses(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get::<DataKey, Vec<Address>>(&DataKey::Addresses)
        .unwrap_or_else(|| Vec::new(env))
}

fn save_addresses(env: &Env, addresses: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::Addresses, addresses);
}

#[cfg(test)]
mod tests {
    use super::{AvoidVecMapInputRemediated, AvoidVecMapInputRemediatedClient, ValidationError};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn stores_one_address_at_a_time() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AvoidVecMapInputRemediated);
        let client = AvoidVecMapInputRemediatedClient::new(&env, &contract_id);

        let first = Address::generate(&env);
        let second = Address::generate(&env);

        assert_eq!(client.try_add_address(&first), Ok(Ok(())));
        assert_eq!(client.try_add_address(&second), Ok(Ok(())));

        let stored = client.list_addresses();
        assert_eq!(stored.len(), 2);
        assert_eq!(stored.get(0).unwrap(), first);
        assert_eq!(stored.get(1).unwrap(), second);
    }

    #[test]
    fn rejects_duplicates() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AvoidVecMapInputRemediated);
        let client = AvoidVecMapInputRemediatedClient::new(&env, &contract_id);

        let address = Address::generate(&env);

        assert_eq!(client.try_add_address(&address), Ok(Ok(())));
        assert_eq!(
            client.try_add_address(&address),
            Err(Ok(ValidationError::DuplicateAddress))
        );

        let stored = client.list_addresses();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored.get(0).unwrap(), address);
    }
}
