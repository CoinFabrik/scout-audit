# Soroban version

### What it does

Warns you if you are using an old version of Soroban in the `Cargo.toml`.

### Why is this bad?

Using an old version of Soroban can be dangerous, as it may have bugs or security issues.

### Example

```toml
[dependencies]
soroban-sdk = { version = "=20.0.0" }

[dev_dependencies]
soroban-sdk = { version = "=20.0.0", features = ["testutils"] }
```

Instead, use the latest available version in the `Cargo.toml`.

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/soroban-version).
