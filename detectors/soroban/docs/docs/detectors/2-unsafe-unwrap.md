# Unsafe unwrap

## Description 

- Category: `Validations and error handling`
- Severity: `Minor`
- Detectors: [`unsafe-unwrap`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-unwrap)
- Test Cases: [`unsafe-unwrap-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-1) [`unsafe-unwrap-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-2) [`unsafe-unwrap-3`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-3)  [`unsafe-unwrap-4`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-4)  [`unsafe-unwrap-5`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-5)  [`unsafe-unwrap-6`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-6) 


In Rust, the `unwrap` method is commonly used for error handling. It retrieves the inner value of an `Option` or `Result`. If an error or `None` occurs, it calls `panic!` without a custom error message.

## Why is this bad? 

`.unwrap()` might panic if the result value is an error or `None`. It is recommended to avoid the panic of a contract because it stops its execution, which might lead the contract to an inconsistent state if the panic occurs in the middle of state changes. Additionally, the panic could cause a transaction to fail.

## Issue example 

Consider the following `Soroban` contract:

```rust
#[contractimpl]
impl UnsafeUnwrap {
    pub fn unwrap(n: u64) -> u64 {
        let result = Self::non_zero_or_error(n);
        result.unwrap()
    }

    pub fn non_zero_or_error(n: u64) -> Result<u64, Error> {
        if n == 0 {
            return Err(Error::CustomError);
        }
        Ok(n)
    }
}
```

In this contract, the `unwrap` function uses the `unwrap` method to save the result of the `non_zero_or_error` function. If the function returns `Err`, the contract will panic and halt execution, potentially leading to malicious exploitation to disrupt the contract's operation.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-1/vulnerable-example).

## Remediated example

Instead of using `unwrap`, use a safer method for error handling. In this case, if the function returns `Err`, it will return a default value (like `0`).

```rust
#[contractimpl]
impl UnsafeUnwrap {
    pub fn unwrap_or_default(n: u64) -> u64 {
        let result = Self::non_zero_or_error(n);
        result.unwrap_or(0)
    }

    pub fn non_zero_or_error(n: u64) -> Result<u64, Error> {
        if n == 0 {
            return Err(Error::CustomError);
        }
        Ok(n)
    }
}
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-unwrap/unsafe-unwrap-1/remediated-example).

## How is it detected?

Checks for usage of .unwrap()

## References

[Rust documentation: `unwrap`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap)




    
