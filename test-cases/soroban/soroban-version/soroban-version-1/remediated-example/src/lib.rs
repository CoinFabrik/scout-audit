#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    // Empty contract - vulnerability is in the Cargo.toml
}
