# Assert violation

## Description

- Category: `Validations and error handling`
- Severity: `Enhancement`
- Detectors: [assert-violation](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/rust/assert-violation/src/lib.rs)

The `assert!` macro is used in Rust to ensure that a certain condition holds true at a certain point in your code.

### Why is this bad?

The `assert!` macro can cause the contract to panic. It is recommended to avoid this, because it stops its execution, which might lead the contract to an inconsistent state if the panic occurs in the middle of state changes. Additionally, the panic could cause a transaction to fail.

### Example

Consider the following snippet code:

```rust
pub fn assert_if_greater_than_10(_env: Env, value: u128) -> bool {
    assert!(value <= 10, "value should be less than 10");
    true
}
```

## Remediated example

Consider using a proper error and return it:

```rust
pub fn assert_if_greater_than_10(_env: Env, value: u128) -> Result<bool, AVError> {
    if value <= 10 {
        Ok(true)
    } else {
        Err(AVError::GreaterThan10)
    }
}
```

## How is it detected?

Checks for `assert!` macro usage.
