# Scout Audit Detectors

This section provides a detailed description of the issues detected by Scout Audit for each supported technology—Ink!, Soroban, and Substrate Pallets. It also includes guidance on how to address these issues to make your code more robust and secure.

## Issue Severity

This severity classification, although arbitrary, has been used in hundreds of security audits and helps to understand the issues detected by Scout.

- **Critical**: These issues seriously compromise the system and must be addressed immediately.
- **Medium**: These are potentially exploitable issues which might represent
  a security risk in the near future. We suggest fixing them as soon as possible.
- **Minor**: These issues represent problems that are relatively small or difficult to exploit, but might be exploited in combination with other issues. These kinds of issues do not block deployments in production environments. They should be taken into account and fixed when possible.
- **Enhancement**: This class relates to issues stemming from deviations from best practices or stylistic conventions, which could escalate into higher-priority issues due to other changes. For instance, these issues may lead to development errors in future updates.

## Issue Category

Below is a taxonomy of issues commonly identified in smart contract audits. While there are many "top vulnerability" lists for Ethereum/Solidity smart contracts, the list provided here is used by the Coinfabrik Audit Team during source code security audits for various platforms, including Ethereum/Solidity, Stacks/Clarity, Algorand/PyTEAL/TEAL, Solana/Rust, and others.

The team discusses the creation of the list in this [blogpost](https://blog.coinfabrik.com/analysis-categories/).

| Category                       | Description                                                                                       |
| ------------------------------ | ------------------------------------------------------------------------------------------------- |
| Arithmetic                     | Proper usage of arithmetic and number representation.                                             |
| Assembly Usage                 | Detailed analysis of implementations using assembly.                                              |
| Authorization                  | Vulnerabilities related to insufficient access control or incorrect authorization implementation. |
| Best practices                 | Conventions and best practices for improved code quality and vulnerability prevention.            |
| Block attributes               | Appropriate usage of block attributes, especially when used as a source of randomness.            |
| Centralization                 | Analysis of centralization and single points of failure.                                          |
| Denial of Service              | Denial of service. attacks.                                                                       |
| Gas Usage                      | Performance issues, enhancements and vulnerabilities related to use of gas.                       |
| Known Bugs                     | Known issues that remain unresolved.                                                              |
| MEV                            | Patterns that could lead to the exploitation of Maximal Extractable Value.                        |
| Privacy                        | Patterns revealing sensible user or state data.                                                   |
| Reentrancy                     | Consistency of contract state under recursive calls.                                              |
| Unexpected transfers           | Contract behavior under unexpected or forced transfers of tokens.                                 |
| Upgradability                  | Proxy patterns and upgradable smart contracts.                                                    |
| Validations and error handling | Handling of errors, exceptions and parameters.                                                    |

We used the above Vulnerability Categories, along with common examples of vulnerabilities detected within each category in other blockchains, as a guideline for finding and developing vulnerable examples smart contracts.
