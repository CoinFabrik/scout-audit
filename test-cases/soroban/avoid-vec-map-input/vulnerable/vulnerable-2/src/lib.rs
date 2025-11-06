#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map};

#[contracttype]
pub enum DataKey {
    AddressFlags,
}

#[contract]
pub struct AvoidVecMapInputMapVulnerable;

#[contractimpl]
impl AvoidVecMapInputMapVulnerable {
    pub fn store_address_map(env: Env, address_map: Map<Address, bool>) {
        // Map is stored without validating its contents.
        env.storage()
            .persistent()
            .set(&DataKey::AddressFlags, &address_map);
    }

    pub fn get_address_map(env: Env) -> Option<Map<Address, bool>> {
        env.storage()
            .persistent()
            .get::<DataKey, Map<Address, bool>>(&DataKey::AddressFlags)
    }
}

#[cfg(test)]
mod tests {
    use super::{AvoidVecMapInputMapVulnerable, AvoidVecMapInputMapVulnerableClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, Map};

    #[test]
    fn raw_map_is_stored() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AvoidVecMapInputMapVulnerable);
        let client = AvoidVecMapInputMapVulnerableClient::new(&env, &contract_id);

        let first = Address::generate(&env);
        let second = Address::generate(&env);

        let mut address_map = Map::new(&env);
        address_map.set(first.clone(), true);
        address_map.set(second.clone(), false);
        client.store_address_map(&address_map);

        let stored = client.get_address_map().expect("map stored");
        assert_eq!(stored.len(), 2);
        assert!(stored.get(first).unwrap());
        assert!(!stored.get(second).unwrap());
    }
}
