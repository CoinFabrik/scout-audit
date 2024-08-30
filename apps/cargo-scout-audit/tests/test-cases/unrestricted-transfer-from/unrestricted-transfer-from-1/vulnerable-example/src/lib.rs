#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

#[derive(Debug, Clone)]
#[contracttype]
pub struct State {
    buyer: Address,
    seller: Address,
    arbiter: Address,
    amount: i128,
    token: Address,
    status: Status,
}

const STATE: Symbol = symbol_short!("STATE");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UTFError {
    InvalidAmount = 1,
    CallerMustBeBuyer = 2,
    CallerMustBeSeller = 3,
    CallerMustBeArbiter = 4,
    StatusMustBeCreated = 5,
    StatusMustBeUnlocked = 6,
    StatusMustBeLocked = 7,
    CouldntRetrieveState = 8,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Created,
    Locked,
    Unlocked,
    Released,
    Refunded,
}

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn init(
        env: Env,
        buyer: Address,
        seller: Address,
        arbiter: Address,
        token: Address,
        amount: i128,
    ) -> State {
        let state = State {
            buyer,
            seller,
            arbiter,
            token,
            amount,
            status: Status::Created,
        };
        env.storage().instance().set(&STATE, &state);
        state
    }

    pub fn deposit(env: Env, from: Address) -> Result<(), UTFError> {
        let mut state: State = Self::get_state(env.clone())?;
        state.buyer.require_auth();
        if state.status != Status::Created {
            return Err(UTFError::StatusMustBeCreated);
        }
        let token_client = token::Client::new(&env, &state.token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &from,
            &env.current_contract_address(),
            &state.amount,
        );
        state.status = Status::Locked;
        env.storage().instance().set(&STATE, &state);
        Ok(())
    }

    pub fn get_state(env: Env) -> Result<State, UTFError> {
        let op_state = env.storage().instance().get(&STATE);
        match op_state {
            None => Err(UTFError::CouldntRetrieveState),
            Some(state) => Ok(state),
        }
    }

    pub fn unlock(env: Env) -> Result<(), UTFError> {
        let mut state = Self::get_state(env.clone())?;
        state.arbiter.require_auth();
        if state.status != Status::Locked {
            Err(UTFError::StatusMustBeLocked)
        } else {
            state.status = Status::Unlocked;
            env.storage().instance().set(&STATE, &state);
            Ok(())
        }
    }

    pub fn release(env: Env) -> Result<(), UTFError> {
        let mut state = Self::get_state(env.clone())?;
        state.seller.require_auth();
        if state.status != Status::Unlocked {
            return Err(UTFError::StatusMustBeUnlocked);
        }
        let token_client = token::Client::new(&env, &state.token);
        token_client.transfer(
            &env.current_contract_address(),
            &state.seller,
            &state.amount,
        );
        state.status = Status::Released;
        env.storage().instance().set(&STATE, &state);
        Ok(())
    }

    pub fn refund(env: Env) -> Result<(), UTFError> {
        let mut state = Self::get_state(env.clone())?;
        state.arbiter.require_auth();
        if state.status != Status::Locked {
            return Err(UTFError::StatusMustBeLocked);
        }
        let token_client = token::Client::new(&env, &state.token);
        token_client.transfer_from(
            &state.arbiter,
            &env.current_contract_address(),
            &state.buyer,
            &state.amount,
        );
        state.status = Status::Refunded;
        env.storage().instance().set(&STATE, &state);
        Ok(())
    }
}
