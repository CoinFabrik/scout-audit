# Unprotected update current contract wasm

## Description 

- Category: `Authorization`
- Severity: `Critical`
- Detector: [`unprotected-update-current-contract-wasm`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unprotected-update-current-contract-wasm)
- Test Cases: [`unprotected-update-current-contract-wasm-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-update-current-contract-wasm/unprotected-update-current-contract-wasm-1) [`unprotected-update-current-contract-wasm-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-update-current-contract-wasm/unprotected-update-current-contract-wasm-2) 

It warns you if `update_current_contract_wasm()` function is called without a previous check of the address of the caller. 


## Why is this bad? 

If users are allowed to call `update_current_contract_wasm()`, they can intentionally modify the contract behaviour, leading to the loss of all associated data/tokens and functionalities given by this contract or by others that depend on it.

## Issue example 

Consider the following `Soroban` contract:

```rust

 #[contractimpl]
impl UpgradeableContract {
    pub fn init(e: Env, admin: Address) {
        e.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn version() -> u32 {
        1
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}

```

This contract allows upgrades through the `update_current_contract_wasm` function. If just anyone can call this function, they could modify the contract behaviour.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-update-current-contract-wasm/unprotected-update-current-contract-wasm-1/vulnerable-example).


## Remediated example

To prevent this, the function should be restricted to administrators or authorized users only.

```rust

 #[contractimpl]
impl UpgradeableContract {
    pub fn init(e: Env, admin: Address) {
        e.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn version() -> u32 {
        1
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-update-current-contract-wasm/unprotected-update-current-contract-wasm-1/remediated-example).

## How is it detected?

It warns you if `update_current_contract_wasm()` function is called without a previous check of the address of the caller.
