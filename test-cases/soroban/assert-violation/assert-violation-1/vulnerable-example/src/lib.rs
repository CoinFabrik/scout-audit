#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol};

#[derive(Debug, Clone)]
#[contracttype]
pub struct State {
    value: u128,
}

const STATE: Symbol = symbol_short!("STATE");
#[contract]
pub struct AssertViolation;

#[contractimpl]
impl AssertViolation {
    pub fn init(env: Env, init_value: u128) -> State {
        let state = State { value: init_value };

        env.storage().instance().set(&STATE, &state);
        state
    }

    pub fn assert_if_greater_than_10(_env: Env, value: u128) -> bool {
        assert!(value <= 10, "value should be less than 10");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_revert_if_greater() {
        let env = Env::default();
        let contract =
            AssertViolationClient::new(&env, &env.register_contract(None, AssertViolation {}));
        assert!(contract.assert_if_greater_than_10(&5));
    }

    #[test]
    #[should_panic(expected = "value should be less than 10")]
    fn reverts_if_greater() {
        let env = Env::default();
        let contract =
            AssertViolationClient::new(&env, &env.register_contract(None, AssertViolation {}));
        contract.assert_if_greater_than_10(&11);
    }
}
