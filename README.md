# Scout Substrate: Security Analysis Tool

![https://img.shields.io/badge/license-MIT-green](https://img.shields.io/badge/license-MIT-green)

<p align="center">
  <img src="/assets/scout.png" alt="Scout in a Dark Forest" width="300" center  />
</p>

Scout is an extensible open-source tool intended to assist Substrate developers and auditors detect common security issues and deviations from best practices.

This tool will help developers write secure and more robust Substrate pallets, runetime code and node code.

Our interest in this project comes from our experience in manual auditing, our usage of comparable tools, and the development of Scout for smart contracts in [Polkadot ink!](https://github.com/CoinFabrik/scout) and [other blockchains](https://github.com/CoinFabrik/scout).

To improve coverage and precision, we´ll persist in research efforts on static and dynamic analysis techniques. Find more about our ongoing research at our associated repository.

## Quick Start

**Install Scout Audit**

Make sure that Cargo is installed on your computer. Then, install Scout with the following command:

```console
cargo install cargo-scout-audit
```

**Run Scout Audit:**

To run Scout on your project execute the following command:

```console
cargo scout-audit
```

## Detectors

Currently, Scout for Substrate includes the following detectors.

| Detector | What it detects | Test cases (vulnerable/remediated) |
|----------|-----------------|------------|
| avoid-dispatch-error | Usage of `DispatchError::Other for error codes`. | [1](./test-cases/avoid-dispatcherror-other/vulnerable/vulnerable-1/) / [1](./test-cases/avoid-dispatcherror-other/remediated/remediated-1/) |
| integer-overflow-or-underflow | Potential for integer arithmetic overflow/underflow. | [1](./test-cases/integer-overflow-or-underflow/vulnerable/vulnerable-1/), [2](./test-cases/integer-overflow-or-underflow/vulnerable/vulnerable-2/) / [1](./test-cases/integer-overflow-or-underflow/remediated/remediated-1/), [2](./test-cases/integer-overflow-or-underflow/remediated/remediated-2/) |
| unsafe-expect | Unsafe usage of `expect`. | [1](./test-cases/unsafe-expect/vulnerable/vulnerable-1/) / [1](./test-cases/unsafe-expect/remediated/remediated-1/) |
| unsafe-unwrap | Unsafe usage of `unwrap`. | [1](./test-cases/unsafe-unwrap/vulnerable/vulnerable-1/) / [1](./test-cases/unsafe-unwrap/remediated/remediated-1/), [2](./test-cases/unsafe-unwrap/remediated/remediated-2/), [3](./test-cases/unsafe-unwrap/remediated/remediated-3/) |
| known-vulnerabilities | Usage of dependencies with know vulnerabilities. | [1](./test-cases/known-vulnerabilities/vulnerable/vulnerable-1/) / [1](./test-cases/known-vulnerabilities/remediated/remediated-1/) |

## Data Set

With the intention of sharing the results of our research, we have created a public repository with a collection of thoroughly analyzed audited Substrate pallets, runtime, and node code. This repository serves as a knowledge base for Substrate developers, auditors, and security researchers aiming to identify common Substrate vulnerabilities and improve the security of their projects.

In addition, we've made the data set publicly available in Hugging Face.

:point_right: [Data set repository](https://github.com/CoinFabrik/scout-substrate-dataset)

:point_right: [Hugging Face data set](TBC)

:heavy_exclamation_mark: Please note that this is a work in progress. The dataset will be updated as we continue with security audit analyses, identify issues, and refine their categories.

## Roadmap

**Milestone 1 - Initial Vulnerability Data Set and PoC Detectors** :white_check_mark:

- Data Set. An open-source GitHub repository containing annotated Substrate pallets, runtime code and node code, accompanied by a detailed document listing vulnerability classes and their locations within the code. The dataset will also be uploaded to Hugging Face data sets.
- Proof-of-concept version of the detectors. Building on our experience with Scout for ink! and Soroban, we aim to detect 4 vulnerability classes. The tool will be delivered as source code in the repository.

**Milestone 2: Extended Vulnerability Data Set and Prototype Detectors with Precision and Recall**

- Prototype version of the detectors. Given our experience developing Scout for ink! and Soroban we aim to include detection of at least 5 new issue classes. Delivered as source code in the repository.
- Extended Data Set. Extended repository of vulnerable pallets, also extending list of vulnerabilities. Updated Hugging Face data set.
- Open Source Framework, for running analysis tools on Rust code, publicly available on CoinFabrik’s GitHub repository.
- Detector’s evaluation report on benchmark Data Set. List of suggested vulnerability classes that appear as false negatives in the report, or have a high rate of false positives. (See an example of the evaluation report for Scout for Soroban)

**Milestone 3: Prototype Tool Integration with CLI, VSCode, and CI/CD, Documentation**

- A prototype tool that integrates built detectors with a CLI, a VSCode extension, and a CI/CD GitHub Action. (See existing VSCode extension and GitHub Action for ink! and Soroban)
Additional or improved detectors for problematic issues identified in Milestone 2. Given our experience developing and improving Scout for ink! and Soroban, we aim to improve or further develop 3 detectors.
- Comprehensive integration tests for all detectors and features.
- A Documentation Site (using Docusaurus or GitBook) detailing tool usage and an initial set of detectors, including nine documented detectors developed in Milestones 1 and 2. (See the documentation pages for Scout on ink! and Soroban)
- A public project GitHub repository and website, along with an alpha tool release for selected projects and users.

**Milestone 4: Final Precision and Recall Evaluation & Full Tool Release**

- Final precision and recall evaluation report. Responsible disclosure of any sensible findings to their corresponding projects.
Improved detectors based on evaluation results. Given our experience developing Scout for ink! and Soroban, we aim to improve or develop 2 detectors after this final precision and recall.
- Fully integrated tool with CLI, VSCode Extension, and/or CI/CD GitHub Action.
- Public release of the tool with full documentation, publicly available on documentation sites (Docusaurus or GitBook)(See documentation examples here and here).
- Video tutorials on how to use the tool, along with one video tutorial for each issue detected by the tool. Given our experience developing Scout for ink! and Soroban, we aim to publish between 10 and 15 video tutorials on CoinFabrik’s YouTube channel. (See Scout video tutorials for other blockchain here).
- Release Webinar.
- Posts on CoinFabrik’s social media.

## About CoinFabrik

We - [CoinFabrik](https://www.coinfabrik.com/) - are a research and development company specialized in Web3, with a strong background in cybersecurity. Founded in 2014, we have worked on over 500 blockchain-related projects, EVM based and also for Solana, Algorand, and Polkadot. Beyond development, we offer security audits through a dedicated in-house team of senior cybersecurity professionals, currently working on code in Substrate, Solidity, Clarity, Rust, and TEAL.

Our team has an academic background in computer science and mathematics, with work experience focused on cybersecurity and software development, including academic publications, patents turned into products, and conference presentations. Furthermore, we have an ongoing collaboration on knowledge transfer and open-source projects with the University of Buenos Aires.

## License

Scout is licensed and distributed under a MIT license. [Contact us](https://www.coinfabrik.com/) if you're looking for an exception to the terms.