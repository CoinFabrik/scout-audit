# Delegated spending should not require owner auth

## Description

- Category: `Best practices`
- Severity: `Low`
- Detector: [`init-instead-of-constructor`](https://github.com/CoinFabrik/scout-audit/tree/main/nightly/2026-04-16/detectors/soroban/init-instead-of-constructor)
- Test Cases: [`delegated-spending-from-auth`](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/init-instead-of-constructor)

Since version 22.0.0 of the Soroban CLI, constructors are supported to automatically initialize contracts the moment they're deployed. Use constructors instead of initializers.

## Why is this bad?

Initializers are error-prone and subject to unauthorized initializations, where someone other than the person who deployed the contract calls the initializer, taking control of the deployment and requiring a second deployment. A more serious issue is that initializer logic is easy to get wrong, especially with regards to preventing double initialization. Constructors solve both of these issues automatically by handing the responsibility over to the blockchain.

## Issue example

```rust
pub fn init(env: Env, admin: Address) {
	admin.require_auth();
	// Store admin in the contract.
}
```

The code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/init-instead-of-constructor/vulnerable/vulnerable-1).

## Remediated example

```rust
pub fn __constructor(env: Env, admin: Address) {
	admin.require_auth();
	// Store admin in the contract.
}
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/init-instead-of-constructor/remediated/remediated-4).

## How is it detected?

The detectos looks for functions named with common initializer names (`init` and `initialize`) and suggests a renaming to `__constructor`.
