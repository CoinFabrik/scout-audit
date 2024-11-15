#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Data,
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
    pub fn init(e: Env) {
        e.storage()
            .persistent()
            .set(&DataKey::Data, &soroban_sdk::Vec::<(Address, i64)>::new(&e));
    }

    pub fn add_data(e: Env, key: Address, value: i64) -> Result<(), Error> {
        let mut data = e
            .storage()
            .persistent()
            .get::<DataKey, soroban_sdk::Vec<(Address, i64)>>(&DataKey::Data)
            .ok_or(Error::Ununitialized)?;
        data.push_back((key, value));
        e.storage().persistent().set(&DataKey::Data, &data);
        Ok(())
    }

    pub fn get(e: Env, key: Address) -> Result<i64, Error> {
        let data = e
            .storage()
            .persistent()
            .get::<DataKey, soroban_sdk::Vec<(Address, i64)>>(&DataKey::Data)
            .ok_or(Error::Ununitialized)?;
        data.iter()
            .find(|(a, _)| *a == key)
            .map(|(_, b)| b)
            .ok_or(Error::NotFound)
    }

    //Second sub-optimal example.
    pub fn get2(e: Env, key: Address) -> Result<i64, Error> {
        e.storage()
            .persistent()
            .get::<DataKey, soroban_sdk::Vec<(Address, i64)>>(&DataKey::Data)
            .ok_or(Error::Ununitialized)?
            .iter()
            .find(|(a, _)| *a == key)
            .map(|(_, b)| b)
            .ok_or(Error::NotFound)
    }
}

#[test]
fn simple_test() {}
