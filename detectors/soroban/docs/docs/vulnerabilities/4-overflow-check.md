# Overflow check

## Description

- Vulnerability Category: `Arithmetic`
- Vulnerability Severity: `Critical`
- Detectors: [`overflow-check`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/overflow-check)
- Test Cases: [`overflow-check-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/overflow-check/overflow-check-1)

Checks that `overflow-checks` is enabled in the `[profile.release]` section of the `Cargo.toml`.

Integer overflow will trigger a panic in debug builds or will wrap in
release mode. Division by zero will cause a panic in either mode. In some applications one
wants explicitly checked, wrapping or saturating arithmetic.


## Exploit Scenario

Consider the following `Cargo.toml`, in a `Soroban` contract:

```toml
[profile.release]
    overflow-checks = false
```

Problems can arise if `overflow-checks` is disabled.

The vulnerable code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/overflow-check/overflow-check-1/vulnerable-example).

## Remediation

```toml
[profile.release]
    overflow-checks = true
```

The remediated code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/overflow-check/overflow-check-1/remediated-example).