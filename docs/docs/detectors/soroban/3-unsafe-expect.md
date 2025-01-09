# Unsafe expect

## Description 

- Category: `Validations and error handling`
- Severity: `Minor`
- Detectors: [`unsafe-expect`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-expect)
- Test Cases: [`unsafe-expect-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-expect/unsafe-expect-1) [`unsafe-expect-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-expect/unsafe-expect-2) [`unsafe-expect-3`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-expect/unsafe-expect-3) [`unsafe-expect-4`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-expect/unsafe-expect-4) [`unsafe-expect-5`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-expect/unsafe-expect-5)

In Rust, the `expect` method is often used for error handling. It returns the contained `Ok` value for a `Result` or `Some` value for an `Option`. If an error occurs, it calls `panic!` with a provided error message.

## Why is this bad? 

`.expect()` might panic if the result value is an error or `None`. It is recommended to avoid the panic of a contract because it stops its execution, which might lead the contract to an inconsistent state if the panic occurs in the middle of state changes. Additionally, the panic could cause a transaction to fail.


## Issue example 

Consider the following `Soroban` contract:

```rust

 pub fn balance_of(env: Env, owner: Address) -> i128 {
    let state = Self::get_state(env);
    state.balances.get(owner).expect("could not get balance")
}
  
```

In this contract, the `balance_of` function uses the expect method to retrieve the balance of an account. If there is no entry for this account in the balances mapping, the contract will panic and halt execution, which could be exploited maliciously to disrupt the contract's operation.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-expect/unsafe-expect-1/vulnerable-example).


## Remediated example

Instead of using `expect`, use a safer method for error handling. In this case, if there is no entry for an account in the `balances` mapping, return a default value (like `0`).

```rust

pub fn balance_of(env: Env, owner: Address) -> i128 {
    let state = Self::get_state(env);
    state.balances.get(owner).unwrap_or(0)
}

```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-expect/unsafe-expect-1/remediated-example).


## How is it detected?

Checks for usage of .expect().


    
