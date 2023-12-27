# Overflow-check

### What it does

Checks that `overflow-checks` is enabled in the `[profile.release]` section of the `Cargo.toml`.

### Why is this bad?

Integer overflow will trigger a panic in debug builds or will wrap in
release mode. Division by zero will cause a panic in either mode. In some applications one
wants explicitly checked, wrapping or saturating arithmetic.


### Example

```toml

[package]
name = "overflow-check-vulnerable-1"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk = "20.0.0-rc2"

[dev_dependencies]
soroban-sdk = { version = "=20.0.0", features = ["testutils"] }

[features]
testutils = ["soroban-sdk/testutils"]

[profile.release]
opt-level = "z"
overflow-checks = false
debug = 0
strip = "symbols"
debug-assertions = false
panic = "abort"
codegen-units = 1
lto = true

[profile.release-with-logs]
inherits = "release"
debug-assertions = true
```

Use instead:

```toml

[package]
name = "overflow-check-remediated-1"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk = "20.0.0-rc2"

[dev_dependencies]
soroban-sdk = { version = "=20.0.0", features = ["testutils"] }

[features]
testutils = ["soroban-sdk/testutils"]

[profile.release]
opt-level = "z"
overflow-checks = true
debug = 0
strip = "symbols"
debug-assertions = false
panic = "abort"
codegen-units = 1
lto = true

[profile.release-with-logs]
overflow-checks = true
inherits = "release"
debug-assertions = true

```

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/integer-overflow-or-underflow).
