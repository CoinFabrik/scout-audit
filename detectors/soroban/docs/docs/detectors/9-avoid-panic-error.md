# Avoid panic error 

## Description 

- Category: `Validations and error handling`
- Severity: `Enhancement`
- Detector: [`avoid-panic-error`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-panic-error)
- Test Cases: [`avoid-panic-error-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1) 

The panic! macro is used to stop execution when a condition is not met. This is useful for testing and prototyping, but should be avoided in production code.

Using `Result` as return type for functions that can fail is the idiomatic way to handle errors in Rust. The `Result` type is an enum that can be either `Ok` or `Err`. The `Err` variant can contain an error message. The `?` operator can be used to propagate the error message to the caller.

This way, the caller can decide how to handle the error, although the state of the contract is always reverted on the callee.

## Why is this bad? 

The usage of `panic!` is not recommended because it will stop the execution of the caller contract. This could lead the contract to an inconsistent state if the execution stops in the middle of state changes. Additionally, if execution stops, it could cause a transaction to fail.

## Issue example 

In the following example, the `panic!` command is being used to handle errors, disallowing the caller to handle the error in a different way, and completely stopping execution of the caller contract.

Consider the following `Soroban` contract:

```rust
pub fn add(env: Env, value: u32) -> u32 {
        let storage = env.storage().instance();
        let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);
        match count.checked_add(value) {
            Some(value) => count = value,
            None => panic!("Overflow error"),
        }
        storage.set(&COUNTER, &count);
        storage.extend_ttl(100, 100);
        count
    }	
```
The add function takes a value as an argument and adds it to the value stored in the contract's storage. The function first checks if the addition will cause an overflow. If the addition will cause an overflow, the function will panic. If the addition will not cause an overflow, the function will add the value to the contract's storage.

The usage of panic! in this example, is not recommended because it will stop the execution of the caller contract. If the method was called by the user, then he will receive ContractTrapped as the only error message.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1/vulnerable-example).


## Remediated example

A possible remediation goes as follows:

```rust
 pub fn add(env: Env, value: u32) -> Result<u32, Error> {
    let storage = env.storage().instance();
    let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);
    match count.checked_add(value) {
        Some(value) => count = value,
        None => return Err(Error::OverflowError),
    }
    storage.set(&COUNTER, &count);
    storage.extend_ttl(100, 100);
    Ok(count)
}
        
```
And adding the following Error enum:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    OverflowError = 1,
}
```
By first defining the Error enum and then returning a Result<(), Error>, more information is added to the caller and, e.g. the caller contract could decide to revert the transaction or to continue execution.

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1/remediated-example).

## How is it detected?

Checks the use of the macro `panic!`.


    
