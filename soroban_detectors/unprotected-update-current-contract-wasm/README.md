# Unprotected update of current contract wasm

### What it does

It warns you if `update_current_contract_wasm()` function is called without a previous check of the address of the caller.

### Why is this bad?

If users are allowed to call `update_current_contract_wasm()`, they can intentionally modify the contract behaviour, leading to the loss of all associated data/tokens and functionalities given by this contract or by others that depend on it.

### Example

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

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
``` 

Use instead:

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

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unprotected-update-current-contract-wasm)
