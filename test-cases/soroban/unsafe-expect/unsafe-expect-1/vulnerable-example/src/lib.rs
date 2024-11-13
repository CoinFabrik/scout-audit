#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol};

#[contract]
pub struct UnsafeExpect;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    balances: Map<Address, i128>,
}
const STATE: Symbol = symbol_short!("STATE");

#[contractimpl]
impl UnsafeExpect {
    pub fn set_balance(env: Env, address: Address, balance: i128) -> State {
        // Get the current state.
        let mut state = Self::get_state(env.clone());

        // Set the new account to have total supply if it doesn't exist.
        if !state.balances.contains_key(address.clone()) {
            state.balances.set(address, balance);
            // Save the state.
            env.storage().persistent().set(&STATE, &state);
        }

        state
    }

    // Returns the balance of a given account.
    pub fn balance_of(env: Env, owner: Address) -> Option<i128> {
        let state = Self::get_state(env);
        let balance = state.balances.get(owner).expect("could not get balance");
        Some(balance)
    }

    /// Return the current state.
    pub fn get_state(env: Env) -> State {
        env.storage().persistent().get(&STATE).unwrap_or(State {
            balances: Map::new(&env),
        })
    }
}

#[cfg(test)]
const TOTAL_SUPPLY: i128 = 200;

#[cfg(test)]
mod tests {

    use soroban_sdk::Env;

    extern crate std;

    use crate::{UnsafeExpect, UnsafeExpectClient, TOTAL_SUPPLY};

    #[test]
    fn balance_of_works() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnsafeExpect);
        let client = UnsafeExpectClient::new(&env, &contract_id);

        // When
        client.set_balance(&contract_id, &TOTAL_SUPPLY);

        // Then
        let balance = client.balance_of(&contract_id);
        assert_eq!(TOTAL_SUPPLY, balance.unwrap());
    }

    #[test]
    #[should_panic(expected = "could not get balance")]
    fn balance_of_expect_works() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnsafeExpect);
        let client = UnsafeExpectClient::new(&env, &contract_id);

        // When - Balance not set

        // Then
        let _balance = client.balance_of(&contract_id);

        // Test should panic
    }
}
