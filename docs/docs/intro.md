---
sidebar_position: 1
---

# Getting Started

Start scouting your project for security issues **in less than 5 minutes**.

## About Scout

Scout is an extensible open-source tool intended to assist smart contract developers and auditors detect common security issues and deviations from best practices. This tool helps developers write secure and more robust smart contracts.

## Supported Technologies

Currently, Scout can be executed on the following technlogies, with tailor-made detectors for each one.

- [Stellar's Soroban](https://stellar.org/soroban)
- [Polkadot ink!](https://use.ink/)
- [Substrate](https://substrate.io/)

## Features

- A list of vulnerabilities, best practices and enhancements, together with associated detectors to identify these issues in your code.
- Command Line Interface (CLI).
- Scout VS Code Extension.
- Scout GitHub Action.

## Install and execute Scout

Make sure that [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) is installed on your computer. Then, install Scout with the following command:

```bash
cargo install cargo-scout-audit
```

To run Scout on your project execute the following command:

```bash
cargo scout-audit
```

💡 Scout supports Cargo Workspaces. When run on a workspace, Scout will be executed on all packages specified as members of the workspace.

⚠️ Make sure that your smart contracts compile properly. Scout won't run if any compilation errors exist.
