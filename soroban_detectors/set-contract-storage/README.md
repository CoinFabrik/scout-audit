# Set contract storage

### What it does

Checks for calls to `env.storage()` without a prior call to `env.require_auth()`.

### Why is this bad?

Functions using keys as variables without proper access control or input sanitation can allow users to perform changes in arbitrary memory locations.

### Known problems

Only check the function call, so false positives could result.

### Example

```rust
fn set_contract_storage(env: Env) {
  let _storage = env.storage().instance();
}
```

Use instead:

```rust
fn set_contract_storage(env: Env, user: Address) {
  user.require_auth();
  let _storage = env.storage().instance();
}
```

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/set-contract-storage).
