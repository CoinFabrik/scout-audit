# Missing new admin signature

## Description

- Category: `Authorization`
- Severity: `Medium`
- Detector: [`missing-new-admin-auth`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/missing-new-admin-auth)
- Test Cases: [`missing-new-admin-auth-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/missing-new-admin-auth/missing-new-admin-auth-1)

When updating an admin or owner address, the incoming address should also sign. Otherwise, a mistaken or unintended address can become stuck in storage and effectively brick the contract or pool.

## Why is this bad?

If the new admin/owner does not authorize the change, a typo or incorrect address can permanently remove administrative control. That can prevent upgrades, configuration changes, or emergency actions.

## Issue example

```rust
pub fn set_admin(e: Env, new_admin: Address) {
    let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
    e.storage().instance().set(&DataKey::Admin, &new_admin);
}
```

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/missing-new-admin-auth/missing-new-admin-auth-1/vulnerable-example).

## Remediated example

```rust
pub fn set_admin(e: Env, new_admin: Address) {
    let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
    new_admin.require_auth();
    e.storage().instance().set(&DataKey::Admin, &new_admin);
}
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/missing-new-admin-auth/missing-new-admin-auth-1/remediated-example).

## How is it detected?

The detector flags writes to `Admin`, `Owner`, `NewAdmin`, or `NewOwner` storage keys when the incoming address does not call `require_auth()` (or `require_auth_for_args`) along the call path from a Soroban entrypoint.
