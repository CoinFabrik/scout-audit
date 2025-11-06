#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct DependencyContract;

#[contractimpl]
impl DependencyContract {
    pub fn noop(_env: Env) {}
}
