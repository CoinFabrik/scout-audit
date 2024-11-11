# Soroban version

## Description

- Category: `Best practices`
- Severity: `Enhacement`
- Detector: [`soroban-version`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/soroban-version)
- Test Cases: [`soroban-version-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/soroban-version/soroban-version-1)

Using an outdated version of Soroban can lead to issues in our contract. It's a good practice to use the latest version.

## Why is this bad?

Using an old version of Soroban can be dangerous, as it may have bugs or security issues.

## Issue example


Consider the following `Cargo.toml`:

```toml
    [dependencies]
    soroban-sdk = { version = "=19.0.0" }

    [dev_dependencies]
    soroban-sdk = { version = "=19.0.0", features = ["testutils"] }
```

Problems can arise if the version is not updated to the latest available.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/soroban-version/soroban-version-1/vulnerable-example).


## Remediated example

```toml
    [dependencies]
    // Use the latest version available.
    soroban-sdk = { workspace = true }

    [dev_dependencies]
    soroban-sdk = { workspace = true, features = ["testutils"] }
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/soroban-version/soroban-version-1/remediated-example)

## How is it detected?

Warns you if you are using an old version of Soroban in the `Cargo.toml`.

## References

- [Floating Pragma](https://swcregistry.io/docs/SWC-103/)
- [outdated Compiler Version](https://swcregistry.io/docs/SWC-102/)


