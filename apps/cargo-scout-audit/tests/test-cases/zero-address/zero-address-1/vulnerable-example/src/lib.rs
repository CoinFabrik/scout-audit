#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Data,
}

#[contract]
pub struct ZeroAddressContract;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Ununitialized = 1,
    NotAdmin = 2,
    NoData = 3,
}

#[contractimpl]
impl ZeroAddressContract {
    pub fn init(e: Env, admin: Address) {
        e.storage().persistent().set(&DataKey::Admin, &admin);
    }

    fn ensure_is_admin(e: &Env, admin: Address) -> Result<bool, Error> {
        let registered_admin = e
            .storage()
            .persistent()
            .get::<DataKey, Address>(&DataKey::Admin)
            .ok_or(Error::Ununitialized)?;
        if admin != registered_admin {
            return Ok(false);
        }
        admin.require_auth();
        Ok(true)
    }

    pub fn set(e: Env, admin: Address, data: i32) -> Result<(), Error> {
        if !ZeroAddressContract::ensure_is_admin(&e, admin)? {
            return Err(Error::NotAdmin);
        }
        e.storage().persistent().set(&DataKey::Data, &data);
        Ok(())
    }

    pub fn get(e: Env) -> Result<i32, Error> {
        e.storage()
            .persistent()
            .get::<DataKey, i32>(&DataKey::Data)
            .ok_or(Error::NoData)
    }

    pub fn change_admin(e: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        if !ZeroAddressContract::ensure_is_admin(&e, admin)? {
            return Err(Error::NotAdmin);
        }

        e.storage().persistent().set(&DataKey::Admin, &new_admin);

        Ok(())
    }
}

#[test]
fn simple_test() {
    use soroban_sdk::testutils::Address as _;

    let e = Env::default();
    e.mock_all_auths();
    let client =
        ZeroAddressContractClient::new(&e, &e.register_contract(None, ZeroAddressContract {}));
    let admin = Address::generate(&e);
    client.init(&admin);
    assert_eq!(client.try_get(), Err(Ok(Error::NoData)));
    client.set(&admin, &5);
    assert_eq!(client.get(), 5);
    client.change_admin(&admin, &Address::generate(&e));
    assert_eq!(client.get(), 5);
    assert_eq!(client.try_set(&admin, &6), Err(Ok(Error::NotAdmin)));
}
