# Scout: Security Analysis Tool

<p align="center">
  <img src="https://raw.githubusercontent.com/CoinFabrik/scout/c1eb3073f85b051dc9ce2fa0ab1ebab4bde0914e/assets/scout.png" alt="Scout in a Dark Forest" width="300" center  />
</p>

Scout is an extensible open-source tool intended to assist [ink!](https://use.ink/smart-contracts-polkadot/) and [Soroboan](https://stellar.org/soroban) smart contract developers and auditors detect common security issues and deviations from best practices.

This tool will help developers write secure and more robust smart contracts.

Our interest in this project comes from our experience in manual auditing and our usage of comparable tools in other blockchains. To improve coverage and precision, we´ll persist in research efforts on static and dynamic analysis techniques.

## Quick Start

For a quick start, make sure that Cargo is installed on your computer. Then, install Scout dependencies by running the following command:

`cargo +nightly install cargo-dylint dylint-link`

Afterwards, install Scout with the following command:

`cargo +nightly install cargo-scout-audit`

To run Scout, navigate to the root directory of any ink! or Soroban project and execute the following command:

`cargo scout-audit`

For more information on installation and usage, please refer to the Getting Started section in our documentation below and to the [Scout for ink!](https://github.com/CoinFabrik/scout) and [Scout for Soroban](https://github.com/CoinFabrik/scout-soroban) respositories.

## Documentation

- [Getting Started](https://coinfabrik.github.io/scout/docs/intro)
- [Vulnerabilities](https://coinfabrik.github.io/scout/docs/vulnerabilities)
- [Detectors](https://coinfabrik.github.io/scout/docs/detectors)
- [Contribute](https://coinfabrik.github.io/scout/docs/contribute)
- [Architecture](https://coinfabrik.github.io/scout/docs/architecture)

## Tests

To validate our tool, we provide a set of code examples located in the test-cases folder.

In order to run the integration tests, navigate to apps/cargo-scout-audit and run:

`cargo test --all --all-features`

In order to run the tests for a particular test-case, run the same command on that particular test-case folder (e.g: test-cases/delegate-call/delegate-call-1/vulnerable-example).

## Acknowledgements

Scout is an open source vulnerability analyzer developed by CoinFabrik's Research and Development team.

We received support through grants from the [Web3 Foundation Grants Program](https://github.com/w3f/Grants-Program/tree/master), the [Aleph Zero Ecosystem Funding Program](https://alephzero.org/ecosystem-funding-program) and the [Stellar Community Fund](https://communityfund.stellar.org/).

## About CoinFabrik

We - CoinFabrik - are a research and development company specialized in Web3, with a strong background in cybersecurity. Founded in 2014, we have worked on over 180 blockchain-related projects, EVM based and also for Solana, Algorand, and Polkadot. Beyond development, we offer security audits through a dedicated in-house team of senior cybersecurity professionals, currently working on code in Substrate, Solidity, Clarity, Rust, and TEAL.

Our team has an academic background in computer science and mathematics, with work experience focused on cybersecurity and software development, including academic publications, patents turned into products, and conference presentations. Furthermore, we have an ongoing collaboration on knowledge transfer and open-source projects with the University of Buenos Aires.
