#![no_std]

macro_rules! contractimport {
    (file = $path:expr) => {
        const _: &str = $path;
    };
}

use soroban_sdk::{contract, contractimpl, Env};

contractimport!(file = "../../../target/wasm32v1-none/release/dependency_contract.wasm");

#[contract]
pub struct DependencyConsumer;

#[contractimpl]
impl DependencyConsumer {
    pub fn noop(_: Env) {}
}
