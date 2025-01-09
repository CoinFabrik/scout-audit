# Assert violation

## Description 

- Category: `Validations and error handling`
- Severity: `Enhancement`
- Detector: [`assert-violation`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/assert-violation)
- Test Cases: [`assert-violation-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/assert-violation/assert-violation-1) 

The `assert!` macro is used in Rust to ensure that a certain condition holds true at a certain point in your code. 

## Why is it bad?

The `assert!` macro can cause the contract to panic. It is recommended to avoid this, because it stops its execution, which might lead the contract to an inconsistent state if the panic occurs in the middle of state changes. Additionally, the panic could cause a transaction to fail.


## Issue example

Consider the following `Soroban` contract:

```rust
    pub fn assert_if_greater_than_10(_env: Env, value: u128) -> bool {
        assert!(value <= 10, "value should be less than 10");
        true
    }
```

The problem arises from the use of the `assert!` macro, if the condition is not met, the contract panics.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/assert-violation/assert-violation-1/vulnerable-example).

## Remediated example

Avoid the use of `assert!` macro. Instead, use a proper error and return it.

```rust
    pub fn assert_if_greater_than_10(_env: Env, value: u128) -> Result<bool, AVError> {
        if value <= 10 {
            Ok(true)
        } else {
            Err(AVError::GreaterThan10)
        }
    }
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/assert-violation/assert-violation-1/remediated-example).

## How is it detected?

Checks for `assert!` macro usage.

## References

- [Assert violation](https://docs.alephzero.org/aleph-zero/security-course-by-kudelski-security/ink-developers-security-guideline#assert-violation)
