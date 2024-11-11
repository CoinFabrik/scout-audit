#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, vec, Env, Vec};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Data,
}

#[contract]
pub struct IteratorsOverIndexingVulnerableContract;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NoData = 1,
    IntegerOverflow = 2,
}

#[contractimpl]
impl IteratorsOverIndexingVulnerableContract {
    pub fn init(e: Env) {
        e.storage()
            .instance()
            .set::<DataKey, Vec<i32>>(&DataKey::Data, &vec![&e, 1_i32, 2_i32, 3_i32, 4_i32]);
    }

    pub fn sum(e: Env) -> Result<i32, Error> {
        let mut ret = 0_i32;
        let vec = e
            .storage()
            .instance()
            .get::<DataKey, Vec<i32>>(&DataKey::Data)
            .ok_or(Error::NoData)?;
        for i in vec {
            ret = ret.checked_add(i).ok_or(Error::IntegerOverflow)?;
        }
        Ok(ret)
    }
}

#[test]
fn simple_test() {
    let e = Env::default();
    e.mock_all_auths();
    let client = IteratorsOverIndexingVulnerableContractClient::new(
        &e,
        &e.register_contract(None, IteratorsOverIndexingVulnerableContract {}),
    );
    client.init();
    assert_eq!(client.sum(), 10);
}
