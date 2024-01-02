# Scout: Security Analysis Tool

<p align="center">
  <img src="https://raw.githubusercontent.com/CoinFabrik/scout/c1eb3073f85b051dc9ce2fa0ab1ebab4bde0914e/assets/scout.png" alt="Scout in a Dark Forest" width="300" center  />
</p>

Scout is an extensible open-source tool intended to assist smart contract developers and auditors detect common security issues and deviations from best practices. This is the vscode extension for Scout.

Visit [Scout's website](https://www.coinfabrik.com/products/scout/) to learn more about the project, currently available for [Polkadot's ink!](https://github.com/coinfabrik/scout) and [Stellar's Soroban](https://github.com/CoinFabrik/scout-soroban) smart contract languages.

## Features

- Detection of common security issues and deviations from best practices.
- Line squiggles and hover messages to highlight issues.

## Requirements

Before installing the extension, make sure you have the following requirements:

- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension installed.
- Rust components installed.
- For Polkadot's ink! version, have [cargo-scout-audit](https://github.com/CoinFabrik/scout) installed.
- For Stellar's Soroban version, have [cargo-scout-audit-soroban](https://github.com/CoinFabrik/scout-soroban) installed.

## Release Notes

### 0.1.3

Fix description and resources.

### 0.1.2

Building upon our commitment to enhancing smart contract security, we are thrilled to announce version 0.1.2 of our Scout vscode extension. This latest update introduces support for [Soroban](https://soroban.stellar.org/) smart contracts on the [Stellar](https://stellar.org/) blockchain, showcasing Scout's expanding versatility and capability to adapt to different Rust-based blockchain environments.

In this release, we are proud to present [12 new detectors and vulnerability classes](https://github.com/CoinFabrik/scout-soroban) specifically tailored for Soroban smart contracts, each accompanied by comprehensive test cases. This significant addition not only extends our coverage to another vital blockchain platform but also reinforces Scout's role as a critical tool in the smart contract development and auditing process.

### 0.1.1

Fix icon.

### 0.1.0

We're excited to announce the initial release of Scout, the vscode extension. This release lays the groundwork for smart contract developers and auditors, to efficiently identify common security issues and deviations from best practices within their ink! smart contracts.

We include in this release [14 detectors and vulnerablity classes with multiple test-cases](https://github.com/CoinFabrik/scout).

## About CoinFabrik

We - [CoinFabrik](https://www.coinfabrik.com/) - are a research and development company specialized in Web3, with a strong background in cybersecurity. Founded in 2014, we have worked on over 180 blockchain-related projects, EVM based and also for Solana, Algorand, and Polkadot. Beyond development, we offer security audits through a dedicated in-house team of senior cybersecurity professionals, currently working on code in Substrate, Solidity, Clarity, Rust, and TEAL.

Our team has an academic background in computer science and mathematics, with work experience focused on cybersecurity and software development, including academic publications, patents turned into products, and conference presentations. Furthermore, we have an ongoing collaboration on knowledge transfer and open-source projects with the University of Buenos Aires.

## License

Scout is licensed and distributed under a MIT license. [Contact us](https://www.coinfabrik.com/) if you're looking for an exception to the terms.
