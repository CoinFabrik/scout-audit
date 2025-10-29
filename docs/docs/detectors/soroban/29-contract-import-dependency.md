# Contract import dependency

## Description

- Category: `Best practices`
- Severity: `Low`
- Detector: [`contract-import-dependency`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/contract-import-dependency)
- Test Cases: [`contract-import-dependency-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/contract-import-dependency/vulnerable/vulnerable-1)

The Soroban `contractimport!` macro allows embedding a compiled WASM artifact directly into a contract. When the imported artifact is not tracked as a dependency, the project can compile and pass tests even if it ships an outdated version of that binary. Missing the `/release/deps/` path convention also bypasses cargo's dependency resolution, leaving the project compiling a stale or incorrect contract.

## Why is this bad?

If the embedded contract is not declared in `Cargo.toml`, a developer may update the dependency source without rebuilding the binary that gets shipped. This causes the deployed contract to behave differently than the one exercised in tests, which hides bugs and can lead to production incidents. Using a path outside `/release/deps/` has a similar effect because the WASM file is no longer tied to the dependency's build output.

## Issue example

```rust
use soroban_sdk::{contract, contractimpl, contractimport, Env};

contractimport!(file = "../../../target/wasm32v1-none/release/dependency_contract.wasm");

#[contract]
pub struct DependencyConsumer;

#[contractimpl]
impl DependencyConsumer {
    pub fn noop(_: Env) {}
}
```

This example embeds `dependency_contract.wasm` directly from the `target/release` folder without declaring or rebuilding the dependency. The full vulnerable contract lives [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/contract-import-dependency/vulnerable/vulnerable-1).

## Remediated example

```rust
mod contract_1 {
    use soroban_sdk::contractimport;

    contractimport!(file = "../../../target/wasm32v1-none/release/deps/dependency_contract.wasm");
}
```

With the dependency added to `Cargo.toml`, the generated WASM now lives under `release/deps/` and stays in sync with the source crate. The remediated contract that calls into the generated client can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/contract-import-dependency/remediated/remediated-1).

## How is it detected?

The detector scans every `contractimport!` invocation that is not part of a unit test. It then verifies two conditions:

1. The imported contract name appears in `Cargo.toml` (either verbatim or using hyphenated naming).
2. The path passed to the macro includes `/release/deps/`.

If either check fails, Scout flags the macro call so the developer can make the dependency explicit and use the canonical output location.
