# Unprotected mapping operation

## Description 

- Category: `Authorization`
- Severity: `Critical`
- Detector: [`unprotected-mapping-operation`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unprotected-mapping-operation)
- Test Cases: [`unprotected-mapping-operation-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-1) [`unprotected-mapping-operation-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-2)  

In Rust, Modifying mappings with an arbitrary key given by the user could lead to several issues. Ideally, only users who have been previously verified should be able to do it.

## Why is this bad? 

- Unintended Modifications: Allowing users to provide arbitrary keys can lead to unintended modifications of critical data within the smart contract. If the input validation and sanitation are not done properly, users may be able to manipulate the data in ways that were not intended by the contract's author.

- Data Corruption: Malicious users could intentionally provide keys that result in the corruption or manipulation of important data stored in the mapping. This could lead to incorrect calculations, unauthorized access, or other undesirable outcomes.

- Denial-of-Service (DoS) Attacks: If users can set arbitrary keys, they may be able to create mappings with a large number of entries, potentially causing the contract to exceed its gas limit. This could lead to denial-of-service attacks, making the contract unusable for other users.

## Issue example 

Consider the following `Soroban` contract:

```rust
   pub fn set_balance(env: Env, address: Address, balance: i128) -> State {
        // Get the current state.
        let mut state = Self::get_state(env.clone());

        // Set the new account to have total supply if it doesn't exist.
        if !state.balances.contains_key(address.clone()) {
            state.balances.set(address, balance);
            // Save the state.
            env.storage().persistent().set(&STATE, &state);
        }

        state
    }
```
The `set_balance()` function allows anyone to call it and modify the account balances in the state. It lacks authorization checks and allows modifying the mutable state directly.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-1/vulnerable-example).


## Remediated example

The fix adds an `address.require_auth()` step, likely checking user permissions to update balances. This ensures only authorized users can modify account data.

```rust
    pub fn set_balance(env: Env, address: Address, balance: i128) -> State {
        // Authenticate user
        address.require_auth();

        // Get the current state.
        let mut state = Self::get_state(env.clone());

        // Set the new account to have total supply if it doesn't exist.
        if !state.balances.contains_key(address.clone()) {
            state.balances.set(address, balance);
            // Save the state.
            env.storage().persistent().set(&STATE, &state);
        }

        state
    }
```

The remediated code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-1/remediated-example).

## How is it detected?

It warns you if a mapping operation (`insert`, `take`, `remove`) function is called with a user-given `key` field of the type `AccountId`.
