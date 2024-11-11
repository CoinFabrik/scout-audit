#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Data(Address),
}

#[contract]
pub struct NonPayableTransferredValueContract;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Ununitialized = 1,
    NoData = 2,
    NotFound = 3,
}

#[contractimpl]
impl NonPayableTransferredValueContract {
    pub fn add_data(e: Env, key: Address, value: i64) -> Result<(), Error> {
        e.storage().persistent().set(&DataKey::Data(key), &value);
        Ok(())
    }

    pub fn get(e: Env, key: Address) -> Result<i64, Error> {
        e.storage()
            .persistent()
            .get::<DataKey, i64>(&DataKey::Data(key))
            .ok_or(Error::NotFound)
    }
}

#[test]
fn simple_test() {}
