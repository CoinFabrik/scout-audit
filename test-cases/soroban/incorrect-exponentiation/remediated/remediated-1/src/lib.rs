#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum IEError {
    CouldntRetrieveData = 1,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Data,
}

#[contract]
pub struct IncorrectExponentiation;

#[contractimpl]
impl IncorrectExponentiation {
    pub fn init(e: Env) {
        e.storage()
            .instance()
            .set::<DataKey, u128>(&DataKey::Data, &(255_u128.pow(2) - 1));
    }

    pub fn get_data(e: Env) -> Result<u128, IEError> {
        let data = e.storage().instance().get(&DataKey::Data);
        match data {
            Some(x) => Ok(x),
            None => Err(IEError::CouldntRetrieveData),
        }
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils, Address, Env};

    use crate::{IncorrectExponentiation, IncorrectExponentiationClient};

    #[test]
    fn simple_test() {
        let env = Env::default();
        let contract_id = env.register_contract(None, IncorrectExponentiation);
        let client = IncorrectExponentiationClient::new(&env, &contract_id);
        env.mock_all_auths();
        let _user = <Address as testutils::Address>::generate(&env);

        client.init();
        assert_eq!(client.get_data(), 65024);
    }
}
