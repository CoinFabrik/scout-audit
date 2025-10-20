#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String};

mod contract_1 {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../../target/wasm32v1-none/release/deps/dependency_contract.wasm");
}

#[contract]
pub struct DependencyConsumer;

#[contractimpl]
impl DependencyConsumer {
    pub fn noop(env: Env) {
        let contract_addr: Address = Address::from_string(&String::from_str(&env, "test"));
        let client = contract_1::Client::new(&env, &contract_addr);
        client.noop();
    }
}
