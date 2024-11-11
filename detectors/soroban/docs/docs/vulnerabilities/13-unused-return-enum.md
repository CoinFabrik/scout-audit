# Unused return enum
## Description
- Vulnerability Category: `Validations and error handling`
- Vulnerability Severity: `Minor`
- Detectors: [`unused-return-enum`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unused-return-enum)
- Test Cases: [`unused-return-enum-1`](hhttps://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1/remediated-example)

Soroban	 messages can return a `Result` enum with a custom error type. This is 
useful for the caller to know what went wrong when the message fails. The
definition in Rust of the `Result` enum is:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

If any of the variants is not used, the code could be simplified or it could 
imply a bug.

## Exploit Scenario
In order to perform this exploit we work through the following example:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// An overflow was produced.
    Overflow = 1,
}



pub fn get_percentage_difference(balance1: u128, balance2: u128) -> Result<u128, Error> {
        let absolute_difference = balance1.abs_diff(balance2);
        let sum = balance1 + balance2;

        match 100u128.checked_mul(absolute_difference / sum) {
            Some(result) => result,
            None => panic!("Overflow"),
        };

        Err(Error::Overflow)
    }	
```

This is a `Soroban` message that returns the percentage difference between two values.

The function then returns an error enum variant `TradingPairErrors::Overflow`.
However, the function never returns a `Result` enum variant `Ok`, thus always 
failing.

The vulnerable code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1/remediated-example	).

## Remediation
This function could be easily fixed by returning a `Result` enum variant `Ok`
when the percentage difference is calculated successfully. By providing a check in 
the linter that ensures that all the variants of the `Result` enum are used, this 
bug could have been avoided. This is shown in the example below:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// An overflow was produced.
    Overflow = 1,
}


pub fn get_percentage_difference(balance1: u128, balance2: u128) -> Result<u128, Error> {
        let absolute_difference = balance1.abs_diff(balance2);
        let sum = balance1 + balance2;

        match 100u128.checked_mul(absolute_difference / sum) {
            Some(result) => Ok(result),
            None => Err(Error::Overflow),
        }
    }
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-panic-error/avoid-panic-error-1/remediated-example/src).
