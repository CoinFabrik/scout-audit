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
        // Only the owner can set the balance.
        address.require_auth();

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
    pub fn balance_of(env: Env, owner: Address) -> (i128, i128) {
        let state = Self::get_state(env);

        // For this example we will get the same balance twice.
        let balance_1 = state.balances.get(owner.clone());
        let balance_2 = state.balances.get(owner);
        if balance_1.is_none() || balance_2.is_none() {
            return (0, 0);
        }
        (
            balance_1.expect("could not get balance"),
            balance_2.expect("could not get balance"),
        )
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

    use crate::{UnsafeExpect, UnsafeExpectClient, TOTAL_SUPPLY};

    #[test]
    fn balance_of_works() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnsafeExpect);
        let client = UnsafeExpectClient::new(&env, &contract_id);

        // When
        client
            .mock_all_auths()
            .set_balance(&contract_id, &TOTAL_SUPPLY);
        let balance = client.balance_of(&contract_id);

        // Then
        assert_eq!(TOTAL_SUPPLY, balance.0);
        assert_eq!(TOTAL_SUPPLY, balance.1);
    }

    #[test]
    fn balance_of_unwrap_works() {
        // Given
        let env = Env::default();
        let contract_id = env.register_contract(None, UnsafeExpect);
        let client = UnsafeExpectClient::new(&env, &contract_id);

        // When - Balance not set
        let balance = client.balance_of(&contract_id);

        // Then
        assert_eq!(0, balance.0);
        assert_eq!(0, balance.1);
    }
}
