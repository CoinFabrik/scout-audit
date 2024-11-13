use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, Env};

fn initialize_env<'a>() -> (Env, TokenInterfaceEventsClient<'a>, Address, [Address; 5]) {
    let env = Env::default();
    let token_contract = TokenInterfaceEventsClient::new(
        &env,
        &env.register_contract(None, TokenInterfaceEvents {}),
    );
    let admin = Address::generate(&env);
    let decimals: u32 = 3;
    let name: String = String::from_str(&env, "TestToken");
    let symbol: String = String::from_str(&env, "TTK");
    let users = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    token_contract.initialize(&admin, &decimals, &name, &symbol);
    (env, token_contract, admin, users)
}
#[test]
fn test_init_token() {
    let env = Env::default();
    let token_contract = TokenInterfaceEventsClient::new(
        &env,
        &env.register_contract(None, TokenInterfaceEvents {}),
    );
    let admin = Address::generate(&env);
    let decimals: u32 = 9;
    let name: String = String::from_str(&env, "TestToken");
    let symbol: String = String::from_str(&env, "TTK");

    token_contract.initialize(&admin, &decimals, &name, &symbol);
    let token_metadata = token_contract.get_metadata();
    assert_eq!(token_metadata.admin, admin);
    assert_eq!(token_metadata.decimals, 9);
    assert_eq!(token_metadata.name, name);
    assert_eq!(token_metadata.symbol, symbol);
}

#[test]

fn test_mint_token() {
    let (env, token_contract, _admin, users) = initialize_env();
    env.mock_all_auths();

    let mut balance = token_contract.balance(&users[0]);
    assert_eq!(balance, 0);
    token_contract.mint(&users[0], &100000);

    balance = token_contract.balance(&users[0]);
    assert_eq!(balance, 100000);
}

#[test]

fn test_transfer_burn_token() {
    let (env, token_contract, _admin, users) = initialize_env();
    env.mock_all_auths();
    token_contract.mint(&users[0], &100_000);
    let previous_balance_user_0 = token_contract.balance(&users[0]);
    let previous_balance_user_1 = token_contract.balance(&users[1]);
    assert_eq!(previous_balance_user_0, 100_000);
    assert_eq!(previous_balance_user_1, 0);

    let transfer_amount = 50_000;
    token_contract.transfer(&users[0], &users[1], &transfer_amount);
    let mut balance_user_0 = token_contract.balance(&users[0]);
    let balance_user_1 = token_contract.balance(&users[1]);
    assert_eq!(balance_user_0, previous_balance_user_0 - transfer_amount);
    assert_eq!(balance_user_1, previous_balance_user_1 + transfer_amount);

    token_contract.burn(&users[0], &10_000);
    balance_user_0 = token_contract.balance(&users[0]);
    assert_eq!(
        balance_user_0,
        previous_balance_user_0 - transfer_amount - 10_000
    );
}

#[test]

fn test_allowance() {
    let (env, token_contract, _admin, users) = initialize_env();
    env.mock_all_auths();
    token_contract.mint(&users[0], &500_000); // 500 tokens (3 decimals)
    let from = users[0].clone();
    let spender = users[1].clone();
    let to = users[2].clone();

    let mut current_allowance = token_contract.allowance(&from, &spender);
    assert_eq!(current_allowance, 0);
    let allowance_amount = 100_000;
    let expiration_ledger = 300;
    token_contract.approve(&from, &spender, &allowance_amount, &expiration_ledger);
    current_allowance = token_contract.allowance(&from, &spender);
    assert_eq!(current_allowance, 100_000);

    let transfer_amount = 20_000;
    // transfer from 20 tokens - sequence is still 0, allowance should be valid
    token_contract.transfer_from(&spender, &from, &to, &transfer_amount);

    let mut from_balance = token_contract.balance(&from);
    let to_balance = token_contract.balance(&to);
    assert_eq!(from_balance, 500_000 - 20_000);
    assert_eq!(to_balance, transfer_amount);

    current_allowance = token_contract.allowance(&from, &spender);

    assert_eq!(current_allowance, allowance_amount - transfer_amount);

    token_contract.burn_from(&spender, &from, &10_000);
    from_balance = token_contract.balance(&from);
    assert_eq!(from_balance, 470_000);

    current_allowance = token_contract.allowance(&from, &spender);
    assert_eq!(current_allowance, 70_000);

    // advance time to verify allowance is now invalid
    env.ledger().with_mut(|info| {
        info.sequence_number += 500;
    });

    current_allowance = token_contract.allowance(&users[0], &users[1]);
    assert_eq!(current_allowance, 0);
}

#[should_panic]
#[test]

fn test_no_allowance() {
    let (env, token_contract, _admin, users) = initialize_env();
    env.mock_all_auths();
    token_contract.mint(&users[0], &500_000); // 500 tokens (3 decimals)
    let from = users[0].clone();
    let spender = users[1].clone();

    token_contract.burn_from(&spender, &from, &100_000);

    let from_balance = token_contract.balance(&from);

    assert_eq!(from_balance, 500_000);
}
