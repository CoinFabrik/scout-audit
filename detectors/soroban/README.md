# Scout Soroban: Security Analysis Tool

![https://img.shields.io/badge/license-MIT-green](https://img.shields.io/badge/license-MIT-green)

<p align="center">
  <img src="/assets/scout.png" alt="Scout in a Dark Forest" width="300" center  />
</p>

Scout is an extensible open-source tool intended to assist Soroban Stellar smart contract developers and auditors detect common security issues and deviations from best practices.

This tool helps developers write secure and more robust smart contracts.

Our interest in this project comes from our experience in manual auditing and vulnerability detection in other blockchains (see [Scout for ink!](https://github.com/CoinFabrik/scout)).

## Quick Start

**Install Scout Audit:**

Make sure that [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) is installed on your computer. Then, install Scout with the following command:

```bash
cargo install cargo-scout-audit
```

**Run Scout Audit:**

To run Scout on your project execute the following command:

```bash
cargo scout-audit
```

:bulb: Scout supports [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). When run on a workspace, Scout will be executed on all packages specified as members of the workspace.

:warning: Make sure that your smart contracts compile properly. Scout won't run if any compilation errors exist.

For more information on installation and usage, please refer to the [Getting Started](https://coinfabrik.github.io/scout-soroban/docs/intro) section in our documentation section below.

## Detectors

Currently Scout includes the following detectors.

| Detector ID                                                                                                              | What it Detects                                                                                                                                                                                           | Test Cases                                                                                                                                                                                                                                               | Severity    |
| ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| [divide-before-multiply](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/divide-before-multiply)      | Performing a division operation before a multiplication, leading to loss of precision.    | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/divide-before-multiply/divide-before-multiply-1), [2](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/divide-before-multiply/divide-before-multiply-2), [3](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/divide-before-multiply/divide-before-multiply-3) | Medium    |
| [unsafe-unwrap](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-unwrap)      | Inappropriate usage of the unwrap method, causing unexpected program crashes.    | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-1)| Medium    |
| [unsafe-expect](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-expect)       | Improper usage of the expect method, leading to unexpected program crashes.    | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-expect/unsafe-expect-1)| Medium    |
| [overflow-check](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/overflow-check)       | An arithmetic operation overflows or underflows the available memory allocated to the variable.    | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/overflow-check/overflow-check-1)| Critical    |
| [insufficiently-random-values](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/insufficiently-random-values)       | Avoid using block attributes for random number generation to prevent manipulation.    | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/insufficiently-random-values/insufficiently-random-values-1)| Critical    |
| [unprotected-update-current-contract-wasm](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unprotected-update-current-contract-wasm)       | If users are allowed to call `update_current_contract_wasm()`, they can intentionally modify the contract behaviour.    | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-update-current-contract-wasm/unprotected-update-current-contract-wasm-1)| Critical    |
| [avoid-core-mem-forget](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-core-mem-forget)                 | The use of `core::mem::forget()` could lead to memory leaks and logic errors.                                                | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-core-mem-forget/avoid-core-mem-forget-1)  | Enhancement    |
| [set-contract-storage](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/set-contract-storage)                           | Insufficient access control on `env.storage()` method.                                                         | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-1), [2](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-2), [3](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/set-contract-storage/set-contract-storage-3)                                                                                                                                                | Critical    |
| [avoid-panic-error](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-panic-error)                           | Code panics on error instead of using descriptive enum.| [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1)                                                                                                                                              | Enhancement    |
| [avoid-unsafe-block](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-unsafe-block)                           | Using unsafe blocks in risks code safety and reliability.| [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-unsafe-block/avoid-unsafe-block-1)                                                                                                                                              | Critical   |
| [dos-unbounded-operation](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/dos-unbounded-operation)                           | DoS due to unbounded operation. | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-1), [2](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-2), [3](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-3)                                                                                                                                              | Medium  |
| [soroban-version](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/soroban-version)                           | Using an old version of Soroban can be dangerous, as it may have bugs or security issues. Use the latest version available. | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/soroban-version/soroban-version-1)                 | Enhancement  |
| [unused-return-enum](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unused-return-enum)                           | Return enum from a function is not completely used. | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unused-return-enum/unused-return-enum-1), [2](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unused-return-enum/unused-return-enum-2)                   | Minor  |
| [iterators-over-indexing](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/iterators-over-indexing)             |Iterating with hardcoded indexes is slower than using an iterator. Also, if the index is out of bounds, it will panic. | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/iterators-over-indexing-1)                   | Enhancement  |
| [assert-violation](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/assert-violation)                           | Avoid the usage of the macro assert!, it can panic.| [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/assert-violation/assert-violation-1)                                                                            | Enhancement    |
| [unprotected-mapping-operation](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unprotected-mapping-operation)                           | Modifying mappings with an arbitrary key given by users can be a significant vulnerability. | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-1), [2](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unprotected-mapping-operation/unprotected-mapping-operation-2)                   | Critical  |
| [dos-unexpected-revert-with-vector](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/dos-unexpected-revert-with-vector)                           | DoS due to improper storage. | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unexpected-revert-with-vector/dos-unexpected-revert-with-vector-1), [2](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unexpected-revert-with-vector/dos-unexpected-revert-with-vector-2)            | Medium  |
| [unrestricted-transfer-from](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unrestricted-transfer-from)                           | Avoid passing an user-defined parameter as a `from` field in transfer-from. | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unrestricted-transfer-from/unrestricted-transfer-from-1)                   | Critical  |
| [unsafe-map-get](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-map-get)                           | Inappropriate usage of the `get` method for `Map` in soroban | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1)              | Medium |
| [incorrect-exponentation](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/incorrect-exponentiation)                           | Warns against incorrect usage of ´^´.  | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/incorrect-exponentiation/incorrect-exponentiation-1)                   | Critical  |
| [token-interface-events](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/token-interface-events)                           | Warns if any of the token functions does not emit an event.  | [1](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/token-interface-events/token-interface-events-1)                   | Medium  |


## Output formats

You can choose the output format that best suit your needs. Scout offers html, markdown, json, pdf and sarif reports. To specify the desired output run the following command:

```
cargo scout-audit --output-format [html|md|pdf|json|sarif]
```

**HTML report**

![Scout HTML report.](/docs/static/img/html-report.png)

## Scout VS Code extension

Add Scout to your development workspace with Scout's VS Code extension to run Scout automatically upon saving your file.

![Scout VS Code extension.](/assets/vscode-extension.png)

:bulb: Tip: To see the errors highlighted in your code, we recommend installing the [Error Lens Extension](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens).

:point_right: Download Scout VS Code from [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=CoinFabrik.scout-audit).

## Scout GitHub Action

Integrate Scout into your CI/CD pipeline! Automatically run the tool against the targeted smart contracts. This immediate feedback loop allows developers to quickly address any issues before merging the code into the main branch, reducing the risk of introducing bugs or vulnerabilities.

**Scout output as a comment in a pull request**

![Scout GitHub action output](/docs/static/img/github-action-output.jpg)

:point_right: Find Scout GitHub Action in [GitHub Marketplace](https://github.com/marketplace/actions/run-scout-action).

## Learning to Scout

Join us for an exciting series of video tutorials where you'll learn how to install and run Scout. Discover how to identify and resolve specific issues detected by the tool, and enhance your skills with our expert guidance.

- [Introduction to Scout](https://www.youtube.com/watch?v=L4kGwPDuWgA)
- [Installing Scout](https://www.youtube.com/watch?v=lStQxKQ_l2Q&t=1s)
- [How to run Scout](https://www.youtube.com/watch?v=_6F24AwscKc)
- [Learning to Scout Soroban: Divide before multiply](https://www.youtube.com/watch?v=aLtXyYvw27o)
- [Learning to Scout Soroban: Incorrect exponentiation](https://www.youtube.com/watch?v=qjnHwKCD_hM)
- [Learning to Scout Soroban: Overflow check](https://www.youtube.com/watch?v=Mi7AcJRPgvU)
- [Learning to Scout Soroban: Insufficiently random values](https://www.youtube.com/watch?v=LPBMDPXmczQ)
- [Learning to Scout Soroban: DoS - Unexpected revert with vector](https://www.youtube.com/watch?v=H79mMnnWyvA)
- [Learning to Scout Soroban: DoS - Unbounded operation](https://www.youtube.com/watch?v=DFM0yNNDiyw)
- [Learning to Scout Soroban: Set contract storage](https://www.youtube.com/watch?v=z6RNfhQt6EI)
- [Learning to Scout Soroban: Unprotected mapping operation](https://www.youtube.com/watch?v=8yayEpKeles)
- [Learning to Scout Soroban: Unprotected update current contract wasm](https://www.youtube.com/watch?v=05WnTt4gw5o)
- [Learning to Scout Soroban: Unrestricted transfer from](https://www.youtube.com/watch?v=jnorbpq3ZXk)
- [Learning to Scout Soroban: Assert violation](https://www.youtube.com/watch?v=-8iv4qXjx-M)
- [Learning to Scout Soroban: Iterators over indexing](https://www.youtube.com/watch?v=PN7sD-W0_Qg)
- [Learning to Scout Soroban: Unsafe expect](https://www.youtube.com/watch?v=sheqaOBOBfo)

:clapper: More videos comming soon!

## Tests

To validate our tool, we provide a set of code examples located in the [test-cases](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases) folder.

In order to run the integration tests, navigate to `apps/cargo-scout-audit` and run:

```console
cargo test --all --all-features
```

In order to run the tests for a particular test-case, run the same command on that particular test-case folder (e.g: `test-cases/divide-before-multiply/divide-before-multiply-1`)

## Documentation

Follow our documentation links below and learn more about the vulnerabilities detected by Scout, how the tool works and how to contribute to the project!

- [Getting Started](https://coinfabrik.github.io/scout-soroban/docs/intro)
- [Vulnerabilities](https://coinfabrik.github.io/scout-soroban/docs/vulnerabilities)
- [Detectors](https://coinfabrik.github.io/scout-soroban/docs/detectors)
- [Contribute](https://coinfabrik.github.io/scout-soroban/docs/contribute)
- [Architecture](https://coinfabrik.github.io/scout-soroban/docs/architecture)
- [Blog](https://blog.coinfabrik.com/)
- [Precision and recall](https://coinfabrik.github.io/scout-soroban/docs/precision-and-recall/first-iteration)
- [Scout GitHub Action](https://coinfabrik.github.io/scout-soroban/docs/github-action)
- [Scout VS Code Extension](https://coinfabrik.github.io/scout-soroban/docs/vscode-extension)
- [Scout Soroban Examples](https://coinfabrik.github.io/scout-soroban/docs/soroban-examples)
- [Toggle detections on and off](https://coinfabrik.github.io/scout-soroban/docs/toggle-detections-on-off)

## Acknowledgements

Scout for Soroban is an open source vulnerability analyzer developed by [CoinFabrik's](https://www.coinfabrik.com/) Research and Development team.

We received support through a grant from the [Stellar Community Fund (SCF)](https://communityfund.stellar.org/).

## About CoinFabrik

We - [CoinFabrik](https://www.coinfabrik.com/) - are a research and development company specialized in Web3, with a strong background in cybersecurity. Founded in 2014, we have worked on over 180 blockchain-related projects, EVM based and also for Solana, Algorand, Stellar and Polkadot. Beyond development, we offer security audits through a dedicated in-house team of senior cybersecurity professionals, currently working on code in Substrate, Solidity, Clarity, Rust, TEAL and Stellar Soroban.

Our team has an academic background in computer science and mathematics, with work experience focused on cybersecurity and software development, including academic publications, patents turned into products, and conference presentations. Furthermore, we have an ongoing collaboration on knowledge transfer and open-source projects with the University of Buenos Aires.

## License

Scout is licensed and distributed under a MIT license. [Contact us](https://www.coinfabrik.com/) if you're looking for an exception to the terms.
