---
sidebar_position: 9
---

# Scout Soroban Smart Contracts Examples

In the context of Scout's development, we engaged developers without Soroban experience to create a series of smart contracts within tight time constraints, encouraging them to introduce errors.

Following this, a senior auditor from CoinFabrik conducted a security review of these smart contracts, focusing on vulnerabilities, deviations from best practices, and potential improvements.

The objective was to obtain a set of smart contracts for Scout testing purposes, allowing us to compare the tool's results with the auditor's findings and refine Scout accordingly, such as by adding new detectors or enhancing existing ones.

Moreover, these smart contracts will serve as educational resources for the Soroban developer community. The security review findings will also help raise awareness about common issues.

## List of Implemented Smart Contracts

Follow the links to the smart contract. Each of them includes a README file with an overview, the functions implemented, and a description of each, as well as instructions on how to interact with the contract.

- [Automatic Market Maker](https://github.com/CoinFabrik/scout-soroban-examples/tree/main/amm)
- [Governance](https://github.com/CoinFabrik/scout-soroban-examples/tree/main/governance)
- [Multi Contract Caller](https://github.com/CoinFabrik/scout-soroban-examples/tree/main/multi-contract-caller)
- [Multisig](https://github.com/CoinFabrik/scout-soroban-examples/tree/main/multisig)
- [Payment Channel](https://github.com/CoinFabrik/scout-soroban-examples/tree/main/payment-channel)
- [Vesting Program](https://github.com/CoinFabrik/scout-soroban-examples/tree/main/vesting)

## Security Review

The security review reported 20 issues and 9 enhancements.

:point_right: Navigate to [this link](https://github.com/CoinFabrik/scout-soroban-examples/blob/main/security-review/README.md) to view the security review.

:warning: Take into consideration that this is not a full security audit. Use these smart contracts with caution.