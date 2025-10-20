#![no_std]

use soroban_sdk::{contract, contractimpl, contractimport, Env};

contractimport!(file = "../../../target/wasm32v1-none/release/dependency_contract.wasm");

#[contract]
pub struct DependencyConsumer;

#[contractimpl]
impl DependencyConsumer {
    pub fn noop(_: Env) {}
}
