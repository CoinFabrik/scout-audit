#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

#[derive(Clone, Debug)]
#[contracttype]
pub struct CounterState {
    admin: Address,
    count: u32,
}

const STATE: Symbol = symbol_short!("STATE");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    FailedToRetrieveState = 3,
}

#[contract]
pub struct StorageChangeEvents;

#[contractimpl]
impl StorageChangeEvents {
    pub fn initialize(env: Env, admin: Address) -> Result<(), SCError> {
        let current_state = Self::get_state(env.clone());
        if current_state.is_ok() {
            return Err(SCError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&STATE, &CounterState { admin, count: 0 });

        Ok(())
    }

    pub fn increase_counter(env: Env) -> Result<(), SCError> {
        let mut counter = Self::get_state(env.clone())?;

        counter.count += 1;

        env.storage().instance().set(&STATE, &counter);
        Ok(())
    }

    pub fn get_state(env: Env) -> Result<CounterState, SCError> {
        let state_op: Option<CounterState> = env.storage().instance().get(&STATE);
        if let Some(state) = state_op {
            Ok(state)
        } else {
            Err(SCError::FailedToRetrieveState)
        }
    }

    fn internal_fn(env: Env) {}
}
