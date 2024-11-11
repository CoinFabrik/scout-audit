# Soroban version

## Description

- Vulnerability Category: `Best practices`
- Vulnerability Severity: `Enhacement`
- Detectors: [`soroban-version`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/soroban-version)
- Test Cases: [`soroban-version-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/soroban-version/soroban-version-1)

Using an old version of Soroban can be dangerous, as it may have bugs or security issues. Use the latest version available.

## Exploit Scenario

Consider the following `Cargo.toml`:

```toml
    [dependencies]
    soroban-sdk = { version = "=19.0.0" }

    [dev-dependencies]
    soroban-sdk = { version = "=19.0.0", features = ["testutils"] }
```

Problems can arise if the version is not updated to the latest available.

The vulnerable code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/soroban-version/soroban-version-1/vulnerable-example).

## Remediation

```toml
    [dependencies]
    // Use the latest version available.
    soroban-sdk = { workspace = true }

    [dev-dependencies]
    soroban-sdk = { workspace = true, features = ["testutils"] }
```

The remediated code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/soroban-version/soroban-version-1/remediated-example).

## References

- [Floating Pragma](https://swcregistry.io/docs/SWC-103/)
- [outdated Compiler Version](https://swcregistry.io/docs/SWC-102/)
