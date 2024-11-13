# Assert violation

## Description

- Vulnerability Category: `Validations and error handling`
- Vulnerability Severity: `Enhancement`
- Detectors: [`assert-violation`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/assert-violation)
- Test Cases: [`assert-violation-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/assert-violation/assert-violation-1)

The `assert!` macro is used in Rust to ensure that a certain condition holds true at a certain point in your code. The `assert!` macro can cause the contract to panic. Therefore, the detector suggests replacing `assert!` constructs with `Error` enum structures.

## Exploit Scenario

Consider the following `Soroban` contract:

```rust
    pub fn assert_if_greater_than_10(_env: Env, value: u128) -> bool {
        assert!(value <= 10, "value should be less than 10");
        true
    }
```

The problem arises from the use of the `assert!` macro, if the condition is not met, the contract panics.

The vulnerable code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/assert-violation/assert-violation-1/vulnerable-example).

## Remediation

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

The remediated code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/assert-violation/assert-violation-1/remediated-example).

## References

- [Assert violation](https://docs.alephzero.org/aleph-zero/security-course-by-kudelski-security/ink-developers-security-guideline#assert-violation)