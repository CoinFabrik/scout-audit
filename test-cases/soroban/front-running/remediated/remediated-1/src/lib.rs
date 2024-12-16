#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};

#[contracttype]
pub enum DataKey {
    Token,
}

#[contract]
pub struct FrontRunning;

#[contractimpl]
impl FrontRunning {
    pub fn init(e: Env, contract: Address) {
        e.storage().persistent().set(&DataKey::Token, &contract);
    }

    pub fn get_token(e: Env) -> Address {
        get_token(&e)
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        StellarAssetClient::new(&e, &get_token(&e)).mint(&to, &amount);
    }

    pub fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        TokenClient::new(&e, &get_token(&e)).approve(&from, &spender, &amount, &expiration_ledger);
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128, min_amount: i128) {
        let transfer_amount = get_conversion_price(amount);
        assert!(transfer_amount >= min_amount, "Insufficient amount");
        TokenClient::new(&e, &get_token(&e)).transfer(&from, &to, &transfer_amount);
    }

    pub fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        TokenClient::new(&e, &get_token(&e)).allowance(&from, &spender)
    }
}

fn get_token(e: &Env) -> Address {
    e.storage().persistent().get(&DataKey::Token).unwrap()
}

fn get_conversion_price(amount: i128) -> i128 {
    // This function symbolizes a change in the state of the blockchain
    100 * amount
}

#[cfg(test)]
mod tests {
    extern crate std;

    use soroban_sdk::{
        testutils::{AuthorizedFunction, AuthorizedInvocation},
        token::TokenClient,
        Address, Env, IntoVal, Symbol,
    };

    use crate::{FrontRunning, FrontRunningClient};
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_approve() {
        // Given
        let env = Env::default();
        let admin = Address::generate(&env);
        let asset_contract = env.register_stellar_asset_contract_v2(admin);

        let contract_id = env.register_contract(None, FrontRunning);
        let client = FrontRunningClient::new(&env, &contract_id);
        client.init(&asset_contract.address());

        // When
        let from = Address::generate(&env);
        let spender = Address::generate(&env);

        // Then
        client
            .mock_all_auths_allowing_non_root_auth()
            .approve(&from, &spender, &200, &200);

        let auths = env.auths();
        assert_eq!(
            auths,
            std::vec![(
                from.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        asset_contract.address().clone(),
                        Symbol::new(&env, "approve"),
                        (&from, &spender, 200_i128, 200_u32).into_val(&env)
                    )),
                    sub_invocations: std::vec![]
                }
            )]
        );

        // Check allowance
        let token_client = TokenClient::new(&env, &client.get_token());
        assert_eq!(token_client.allowance(&from, &spender), 200);
    }

    #[test]
    fn test_transfer() {
        // Given
        let env = Env::default();
        let admin = Address::generate(&env);
        let asset_contract = env.register_stellar_asset_contract_v2(admin);

        let contract_id = env.register_contract(None, FrontRunning);
        let client = FrontRunningClient::new(&env, &contract_id);
        client.init(&asset_contract.address());

        // When
        let token_client = TokenClient::new(&env, &client.get_token());
        assert_eq!(token_client.decimals(), 7);
        let from = Address::generate(&env);
        let to = Address::generate(&env);

        // Mint tokens to the `from` address
        client
            .mock_all_auths_allowing_non_root_auth()
            .mint(&from, &10000);

        // Then
        client
            .mock_all_auths_allowing_non_root_auth()
            .transfer(&from, &to, &1, &0);

        // Check final balances
        let from_balance = token_client.balance(&from);
        let to_balance = token_client.balance(&to);

        assert_eq!(from_balance, 10000 - 100);
        assert_eq!(to_balance, 100);
    }

    #[test]
    #[should_panic(expected = "Insufficient amount")]
    fn test_front_running() {
        // Given
        let env = Env::default();
        let admin = Address::generate(&env);
        let asset_contract = env.register_stellar_asset_contract_v2(admin);

        let contract_id = env.register_contract(None, FrontRunning);
        let client = FrontRunningClient::new(&env, &contract_id);
        client.init(&asset_contract.address());

        // When
        let token_client = TokenClient::new(&env, &client.get_token());
        assert_eq!(token_client.decimals(), 7);
        let from = Address::generate(&env);
        let to = Address::generate(&env);

        // Mint tokens to the `from` address
        client
            .mock_all_auths_allowing_non_root_auth()
            .mint(&from, &10000);

        // Then
        client
            .mock_all_auths_allowing_non_root_auth()
            .transfer(&from, &to, &100, &10001);
        // The contract should now panic since the transfer amount is less than the minimum amount
    }
}
