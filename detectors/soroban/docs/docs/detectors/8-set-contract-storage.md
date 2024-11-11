# Set contract storage

## Description 

- Category: `Authorization`
- Severity: `Critical`
- Detector: [`set-contract-storage`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/set-contract-storage)
- Test Cases: [`set-contract-storage-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-1) [`set-contract-storage-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-2) [`set-contract-storage-3`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-3)  [`set-contract-storage-4`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-4)

Smart contracts can store important information in memory which changes through the contract's lifecycle. Changes happen via user interaction with the smart contract. An unauthorized set contract storage issue happens when a smart contract call allows a user to set or modify contract memory when he was not supposed to be authorized.

## Why is this bad? 

Unauthorized access to storage by a user can lead to serious issues, as they might manipulate memory data to attack the contract.

## Issue example 

Consider the following `Soroban` contract:

```rust

 #[contractimpl]
impl SetContractStorage {
    /// Increment an internal counter; return the new value.
    pub fn increment(env: Env, user: Address) -> u32 {
        let storage = env.storage().instance();
        let mut count: u32 = storage.get(&user).unwrap_or_default();
        count += 1;
        storage.set(&user, &count);
        storage.extend_ttl(100, 100);
        count
    }
}    

```

In this example we see that any user may access the SetContractStorage() function, and therefore modify the value of the internal counter.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-1/vulnerable-example).


## Remediated example

Arbitrary users should not have control over keys because it implies writing any value of a mapping, lazy variable, or the main struct of the contract located in position 0 of the storage. To prevent this issue, set access control and proper authorization validation for the SetContractStorage() function.

For example, the code below, ensures only the authorized users can call SetContractStorage().

```rust

 #[contractimpl]
impl SetContractStorage {
    /// Increment an internal counter; return the new value.
    pub fn increment(env: Env, user: Address) -> u32 {
        user.require_auth();
        let storage = env.storage().instance();
        let mut count: u32 = storage.get(&user).unwrap_or_default();
        count += 1;
        storage.set(&user, &count);
        storage.extend_ttl(100, 100);
        count
    }
}
        
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-1/remediated-example).

## How is it detected?

Checks for calls to env.storage() without a prior call to env.require_auth().



    
