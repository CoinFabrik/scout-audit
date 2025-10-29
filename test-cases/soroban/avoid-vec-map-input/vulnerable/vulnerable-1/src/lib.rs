#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Addresses,
}

#[contract]
pub struct AvoidVecMapInputVulnerable;

#[contractimpl]
impl AvoidVecMapInputVulnerable {
    pub fn store_addresses(env: Env, addresses: Vec<Address>) {
        // Collection is stored without validating its contents.
        env.storage()
            .persistent()
            .set(&DataKey::Addresses, &addresses);
    }

    pub fn get_addresses(env: Env) -> Option<Vec<Address>> {
        env.storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::Addresses)
    }
}

#[cfg(test)]
mod tests {
    use super::{AvoidVecMapInputVulnerable, AvoidVecMapInputVulnerableClient};
    use soroban_sdk::{testutils::Address as _, vec, Address, Env};

    #[test]
    fn raw_vec_is_stored() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AvoidVecMapInputVulnerable);
        let client = AvoidVecMapInputVulnerableClient::new(&env, &contract_id);

        let first = Address::generate(&env);
        let second = Address::generate(&env);

        let addresses = vec![&env, first.clone(), second.clone()];
        client.store_addresses(&addresses);

        let stored = client.get_addresses().expect("addresses stored");
        assert_eq!(stored.len(), 2);
        assert_eq!(stored.get(0).unwrap(), first);
        assert_eq!(stored.get(1).unwrap(), second);
    }
}
