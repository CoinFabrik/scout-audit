#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

const COUNTER: Symbol = symbol_short!("COUNT");

#[contract]
pub struct InfiniteRecursionOverStorage;

#[contractimpl]
impl InfiniteRecursionOverStorage {
    pub fn start(env: Env, depth: u32) -> u32 {
        Self::enter(env, depth)
    }

    fn enter(env: Env, depth: u32) -> u32 {
        if depth % 2 == 0 {
            Self::touch_storage(env, depth)
        } else {
            Self::reenter(env, depth + 1)
        }
    }

    fn touch_storage(env: Env, depth: u32) -> u32 {
        env.storage()
            .instance()
            .get::<Symbol, u32>(&COUNTER)
            .unwrap_or(depth);
        Self::reenter(env, depth + 1)
    }

    fn reenter(env: Env, depth: u32) -> u32 {
        Self::enter(env, depth + 1)
    }
}
