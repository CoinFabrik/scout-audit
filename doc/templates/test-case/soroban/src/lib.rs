#![no_std]
use soroban_sdk::{contract, contractimpl, log, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNTER");

#[contract]
pub struct IncrementorContract;

#[contractimpl]
impl IncrementorContract {
    /// Increment an internal counter; return the new value.
    pub fn increment(env: Env) -> u32 {
        let mut count: u32 = env.storage().instance().get(&COUNTER).unwrap_or(0);

        count += 1;

        log!(&env, "count: {}", count);

        env.storage().instance().set(&COUNTER, &count);

        env.storage().instance().extend_ttl(100, 100);

        count
    }
}

#[cfg(test)]
mod test{
    use crate::{IncrementorContract, IncrementorContractClient};
    use soroban_sdk::{ Env };
    
    #[test]
    fn increment() {
        let env = Env::default();
        let contract_id = env.register_contract(None, IncrementorContract);
        let client = IncrementorContractClient::new(&env, &contract_id);

        assert_eq!(client.increment(), 1);
        assert_eq!(client.increment(), 2);
        assert_eq!(client.increment(), 3);
    }
}