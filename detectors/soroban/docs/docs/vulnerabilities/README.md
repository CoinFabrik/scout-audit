---
sidebar_position: 2
---

# Vulnerabilities

This section lists relevant security-related issues typically introduced during the development of smart contracts. The list, though non-exhaustive, features highly relevant items. Each issue is assigned a severity label based on the taxonomy presented below.

## Vulnerability Severity

This severity classification, although arbitrary, has been used in hundreds
of security audits and helps to understand the vulnerabilities we introduce
and measure the utility of this proof of concept.

- **Critical**: These issues seriously compromise the system and must be addressed immediately.
- **Medium**: These are potentially exploitable issues which might represent
  a security risk in the near future. We suggest fixing them as soon as possible.
- **Minor**: These issues represent problems that are relatively small or difficult to exploit, but might be exploited in combination with other issues. These kinds of issues do not block deployments in production environments. They should be taken into account and fixed when possible.
- **Enhancement**: This class relates to issues stemming from deviations from best practices or stylistic conventions, which could escalate into higher-priority issues due to other changes. For instance, these issues may lead to development errors in future updates.

## Vulnerability Categories

We follow with a taxonomy of Vulnerabilities. Many "top vulnerability" lists
can be found covering Ethereum/Solidity smart contracts. This list below is
used by the Coinfabrik Audit Team, when source code (security) audits in
Ethereum/Solidity, Stacks/Clarity, Algorand/PyTEAL /TEAL, Solana/RUST, etc.
The team discusses the creation of the list in this
[blogpost](https://blog.coinfabrik.com/analysis-categories/).

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

We used the above Vulnerability Categories, along with common examples of vulnerabilities detected within each category in other blockchains, as a guideline for finding and developing vulnerable examples of Stellar Soroban smart contracts.

## Vulnerability Classes

As a result of our research, we have so far identified 4 types of vulnerabilities.

What follows is a description of each vulnerability in the context of Stellar Soroban smart contracts. In each case, we have produced at least one [test-case](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases) smart contract that exposes one of these vulnerabilities.

Check our
[test-cases](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases)
for code examples of these vulnerabilities and their respective remediations.


### Divide before multiply

This vulnerability class relates to the order of operations in Rust, specifically in integer arithmetic. Performing a division operation before a multiplication can lead to a loss of precision. This issue becomes significant in programs like smart contracts where numerical precision is crucial.

This vulnerability falls under the [Arithmetic](#vulnerability-categories) category
and has a Medium Severity.


### Unsafe unrwap

This vulnerability class pertains to the inappropriate usage of the `unwrap` method in Rust, which is commonly employed for error handling. The `unwrap` method retrieves the inner value of an `Option` or `Result`, but if an error or `None` occurs, it triggers a panic and crashes the program.

This vulnerability again falls under the [Validations and error handling](#vulnerability-categories) category and has a Medium severity.

In our example, we consider an contract that utilizes the `unwrap` method to retrieve the balance of an account from a mapping. If there is no entry for the specified account, the contract will panic and abruptly halt execution, opening avenues for malicious exploitation.


### Unsafe expect

In Rust, the `expect` method is commonly used for error handling. It retrieves the value from a `Result` or `Option` and panics with a specified error message if an error occurs. However, using `expect` can lead to unexpected program crashes.

This vulnerability falls under the [Validations and error handling](#vulnerability-categories) category
and has a Medium severity.

In our example, we see an exploit scenario involving a contract using the `expect` method in a function that retrieves the balance of an account. If there is no entry for the account, the contract panics and halts execution, enabling malicious exploitation.

### Integer overflow or underflow

This type of vulnerability occurs when an arithmetic operation attempts to
create a numeric value that is outside the valid range in substrate, e.g,
a `u8` unsigned integer can be at most _M:=2^8-1=255_, hence the sum `M+1`
produces an overflow.

An overflow/underflow is typically caught and generates an error. When it
is not caught, the operation will result in an inexact result which could
lead to serious problems.

We classified this type of vulnerability under
the [Arithmetic](#vulnerability-categories) category and assigned it a
Critical severity.

In the context of Soroban, we found that this vulnerability could only be
realized if `overflow-checks` is set to `False` in the `[profile.release]` section of the `Cargo.toml`.
Notwithstanding, there are contexts where developers do turn off checks for
valid reasons and hence the reason for including this vulnerability in the
list.


### Insufficiently random values

Using block attributes like ledger `timestamp()` and ledger `sequence()` for random number generation in Soroban smart contracts is not recommended due to the predictability of these values. Block attributes are publicly visible and deterministic, making it easy for malicious actors to anticipate their values and manipulate outcomes to their advantage. Furthermore, validators could potentially influence these attributes, further exacerbating the risk of manipulation. For truly random number generation, it's important to use a source that is both unpredictable and external to the blockchain environment, reducing the potential for malicious exploitation.

This vulnerability again falls under the [Block attributes](#vulnerability-categories) category
and has a Critical severity.

### Unprotected update of current contract wasm

If users are allowed to call `update_current_contract_wasm()`, they can intentionally modify the contract behaviour, leading to the loss of all associated data/tokens and functionalities given by this contract or by others that depend on it. To prevent this, the function should be restricted to administrators or authorized users only.

This vulnerability falls under the [Authorization](#vulnerability-categories) category and has a Critical severity.

### Avoid core::mem::forget

The `core::mem::forget` function is used to forget about a value without running its destructor. This could lead to memory leaks and logic errors.

We classified this issue, a deviation from best practices which could have
security implications, under the [Best practices](#vulnerability-categories) category and assigned it an Enhancement severity.

### Set contract storage

Smart contracts can store important information in memory which changes through the contract's lifecycle. Changes happen via user interaction with the smart contract. An _unauthorized_ set contract storage vulnerability happens when a smart contract call allows a user to set or modify contract memory when they were not supposed to be authorized.

Common practice is to have functions with the ability to change
security-relevant values in memory to be only accessible to specific roles,
e.g, only an admin can call the function `reset()` which resets auction values.
When this does not happen, arbitrary users may alter memory which may impose
great damage to the smart contract users.

In `Soroban`, the method `env.storage()` can be used
to modify the contract storage under a given key. When a smart contract uses
this method, the contract needs to check if the caller should be able to
alter this storage. If this does not happen, an arbitary caller may modify
balances and other relevant contract storage.

We classified this type of vulnerability under
the [Authorization](#vulnerability-categories) category and assigned it a
Critical severity.

### Avoid panic error

The use of the `panic!` macro to stop execution when a condition is not met is
useful for testing and prototyping but should be avoided in production code.
Using `Result` as the return type for functions that can fail is the idiomatic
way to handle errors in Rust.

We classified this issue, a deviation from best practices which could have
security implications, under the [Validations and error handling](#vulnerability-categories) category and assigned it an Enhancement severity.

### Avoid unsafe block

The use of `unsafe` blocks in Rust is generally discouraged due to the potential risks it poses to the safety and reliability of the code. Rust's primary appeal lies in its ability to provide memory safety guarantees, which are largely enforced through its ownership and type systems. When you enter an `unsafe` block, you're effectively bypassing these safety checks. This can lead to various issues, such as undefined behavior, memory leaks, or security vulnerabilities. These blocks require the programmer to manually ensure that memory is correctly managed and accessed, which is prone to human error and can be challenging even for experienced developers. Therefore, unsafe blocks should only be used when absolutely necessary and when the safety of the operations within can be assured.

We classified this issue, a deviation from best practices which could have
security implications, under the [Validations and error handling](#vulnerability-categories) category and assigned it a Critical severity.

### DoS unbounded operation

Each block in Soroban Stellar has an upper bound on the amount of gas
that can be spent, and thus the amount of computation that can be done. This
is the Block Gas Limit. If the gas spent by a function call on a Soroban smart
contract exceeds this limit, the transaction will fail. Sometimes it is the
case that the contract logic allows a malicious user to modify conditions
so that other users are forced to exhaust gas on standard function calls.

In order to prevent a single transaction from consuming all the gas in a block,
unbounded operations must be avoided. This includes loops that do not have a
bounded number of iterations, and recursive calls.

A denial of service vulnerability allows the exploiter to hamper the
availability of a service rendered by the smart contract. In the context
of Soroban smart contracts, it can be caused by the exhaustion of gas,
storage space, or other failures in the contract's logic.

We classified this type of vulnerability under
the [Denial of Service](#vulnerability-categories) category and assigned it a
Medium severity.

### Soroban version

Using an old version of Soroban can be dangerous, as it may have bugs or security issues. Use the latest version available.

We classified this issue, a deviation from best practices which could have
security implications, under the [Best practices](#vulnerability-categories) category and assigned it an Enhancement severity.

### Unused return enum

`Rust` messages can return a `Result` `enum` with a custom error type. This is
useful for the caller to know what went wrong when the message fails. The
definition of the `Result` type enum consists of two variants: Ok and Err. If
any of the variants is not used, the code could be simplified or it could imply
a bug.

We put this vulnerability under the [Validations and error handling category](#vulnerability-categories) with a Minor severity.

###  Iterators-over-indexing

Accessing a vector by index is slower than using an iterator. Also, if the index is out of bounds, it will panic. 

This could lead to potential integer overflow vulnerabilities, which would trigger a panic in debug builds or wrap in release mode, jeopardizing the integrity and security of the smart contract. Additionally, failing to verify the existence of data in storage before operations could result in unexpected errors or runtime failures, compromising the reliability of the contract execution.

This vulnerability falls under the [Best practices](#vulnerability-categories) category and has an Enhancement severity.

### Assert violation

The assert! macro is used in Rust to ensure that a certain condition holds true at a certain point in your code. The `assert!` macro can cause the contract to panic. Therefore, the detector suggests replacing `assert!` constructs with `Error` enum structures.

This vulnerability falls under the category [Validations and error handling](#vulnerability-categories) and has an Enhancement severity.

### Unprotected mapping operation

Modifying mappings with an arbitrary key given by the user could lead to unintented modifications of critical data, modifying data belonging to other users, causing denial of service, unathorized access, and other potential issues.

This vulnerability falls under the [Validations and error handling category](#vulnerability-categories) and assigned it a Critical severity.

### Unrestricted transfer from

In a smart contract, allowing unrestricted `transfer_from` operations poses a significant vulnerability. When `from` arguments for that function is provided directly by the user, this might enable the withdrawal of funds from any actor with token approval on the contract. This could result in unauthorized transfers and loss of funds. To mitigate this vulnerability, instead of allowing an arbitrary `from` address, the `from` address should be restricted.

This vulnerability falls under the [Validations and error handling](#vulnerability-categories) category
and has a Critical severity.

### DoS unexpected revert with vector

Another type of Denial of Service attack is called unexpected revert. It occurs
by preventing transactions by other users from being successfully executed,
forcing the blockchain state to revert to its original state.

A Denial of Service through unexpected revert can
accomplished by exploiting a smart contract that does not manage storage size
errors correctly. It can be prevented by using Mapping instead of Vec to avoid
storage limit problems.

This vulnerability again falls under the [Denial of Service](#vulnerability-categories) category
and has a Medium severity.

### Unsafe map get

The use of certain methods (`get`, `get_unchecked`, `try_get_unchecked`) on a `Map` object in the Soroban environment without appropriate error handling can lead to potential runtime panics. This vulnerability stems from accessing the map's values with keys that may not exist, without using safer alternatives that check the existence of the key. Such practices can compromise the robustness of the smart contract by causing it to terminate unexpectedly, which may lead to denial of service or inconsistent state within the contract.

This vulnerability falls under the [Validations and error handling category](#vulnerability-categories) category and is assigned a Medium severity level.

### Zero or test address

The assignment of the zero address to a variable in a smart contract represents a critical vulnerability because it can lead to loss of control over the contract. This stems from the fact that the zero address does not have an associated private key, which means it's impossible to claim ownership, rendering any contract assets or functions permanently inaccessible.

Assigning a test address can also have similar implications, including the loss of access or granting access to a malicious actor if its private keys are not handled with care.

This vulnerability falls under the [Validations and error handling](#vulnerability-categories) category
and has a Medium severity.

### Incorrect Exponentiation

It's common to use `^` for exponentiation. However in Rust, `^` is the XOR operator. If the `^` operator is used, it could lead to unexpected behavior in the contract. It's recommended to use the method `pow()` for exponentiation or `.bitxor()` for XOR operations.

This vulnerability falls under the [Arithmetic](#vulnerability-categories) category and has a Critical severity.