# Token interface inference

## Description 

- Category: `Best practices`
- Severity: `Enhancement`
- Detectors: [`token-interface-inference`](https://github.com/CoinFabrik/scout-audit/tree/main/nightly/2025-08-07/detectors/soroban/token-interface-inference)
- Test Cases: [`token-interface-inference`](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/token-interface-inference)

In Soroban, token contracts should implement the standard `TokenInterface` trait (`soroban_sdk::token::TokenInterface`) to comply with the SEP-41 standard. This detector identifies contracts that appear to be tokens—because they implement many canonical token functions natively (such as `transfer`, `balance`, `allowance`, etc.)—but fail to implement the actual `TokenInterface` trait.

## Why is this bad? 

Implementing the `TokenInterface` trait helps ensure proper compliance with the SEP-41 token standard. If a contract implements token functionalities natively but avoids the standard trait, the following issues may arise:

* **Interoperability:** Other developers, decentralized applications (dApps), and smart contracts interact with tokens assuming they implement `TokenInterface`. Without implementing this standard trait, external components might fail to integrate properly.
* **Standard Compliance:** The standard enforces exact function signatures (names, inputs, and outputs). Implementing methods manually without the trait can lead to mistakes or typos in function interfaces, breaking compatibility.
* **Safety and tooling:** The Soroban SDK provides specific types and utilities (like `TokenUtils`) that work seamlessly when contracts use the official token interfaces.

## Issue example 

Consider the following `Soroban` contract:

```rust
#[contract]
pub struct TokenInterfaceInference;

#[contractimpl]
impl TokenInterfaceInference {
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_balance = Self::balance(env.clone(), from.clone());
        let to_balance = Self::balance(env.clone(), to.clone());
        assert!(from_balance >= amount);
        env.storage()
            .instance()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::Balance(to), &(to_balance + amount));
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }
    
    // ... other token-interface functions like approve, allowance, burn, etc. ...
}
```

In this example, the contract implements typical token functions on its own `impl` block instead of implementing `soroban_sdk::token::TokenInterface`.

The code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/token-interface-inference/vulnerable/vulnerable-1).

## Remediated example

```rust
use soroban_sdk::token;

#[contract]
pub struct TokenInterfaceInference;

#[contractimpl]
impl TokenInterfaceInference {
    // Initialization and other non-standard logic...
}

#[contractimpl]
impl token::TokenInterface for TokenInterfaceInference {
    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_balance = Self::balance(env.clone(), from.clone());
        let to_balance = Self::balance(env.clone(), to.clone());
        assert!(from_balance >= amount);
        env.storage()
            .instance()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));
        // ... (events should be emitted as well) ...
    }

    fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }
    
    // ... other standard token functions ...
}
```

In this remediated example, the contract uses `impl token::TokenInterface for TokenInterfaceInference`, ensuring full compatibility with the SEP-41 standard.

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/token-interface-inference/remediated/remediated-1).

## How is it detected?

The detector iterates over the functions implemented in the contract and measures their similarity (accounting for naming variations) against the 10 canonical [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) `TokenInterface` functions: `allowance`, `approve`, `balance`, `transfer`, `transfer_from`, `burn`, `burn_from`, `decimals`, `name`, and `symbol`.

If the contract matches at least 6 of those 10 token-interface functions but relies on standard `impl` blocks rather than implementing the trait `soroban_sdk::token::TokenInterface`, the detector flags it.

SEP-41 standardizes a `mint` event, but it does not include a `mint` function in `TokenInterface`; minting logic is contract-specific and is not counted by this detector.
