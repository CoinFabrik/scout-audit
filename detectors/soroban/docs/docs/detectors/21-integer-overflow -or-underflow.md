# Integer overflow or underflow

## Description 

- Category: `Arithmetic`
- Severity: `Critical`
- Detectors: [`integer-overflow-or-underflow`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/integer-overflow-or-underflow)
- Test Cases: [`integer-overflow-or-underflow-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-1)
[`integer-overflow-or-underflow-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-2)
[`integer-overflow-or-underflow-3`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-3)
[`integer-overflow-or-underflow-4`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-4)
[`integer-overflow-or-underflow-5`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-5)

In Rust, arithmetic operations can result in a value that falls outside the allowed numerical range for a given type. When the result exceeds the maximum value of the range, it's called an overflow, and when it falls below the minimum value of the range, it's called an underflow.

## Why is this bad? 

If there are arithmetic operations with overflow or underflow problems, and if errors are not handled correctly, incorrect results will be generated, bringing potential problems for the contract. Additionally, these types of errors can allow attackers to drain a contract’s funds or manipulate its logic.

## Issue example 

Consider the following `Soroban` contract:

```rust

 pub fn add(env: Env, value: u32) {
        let current: u32 = env.storage().temporary().get(&Self::VALUE).unwrap_or(0);
        let new_value = current + value;
        env.storage().temporary().set(&Self::VALUE, &new_value);
    }

```

In this example, an operation is performed on two u32 values without any safeguards against overflow if it occurs.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-1/vulnerable-example).


## Remediated example

```rust
pub fn add(env: Env, value: u32) -> Result<(), Error> {
        let current: u32 = env.storage().temporary().get(&Self::VALUE).unwrap_or(0);
        let new_value = match current.checked_add(value) {
            Some(value) => value,
            None => return Err(Error::OverflowError),
        };
        env.storage().temporary().set(&Self::VALUE, &new_value);
        Ok(())
    }       
```
In this example, the `checked_add` method is used to perform the addition. It returns the sum if no overflow occurs; otherwise, it returns `None`, with an OverflowError variant indicating that an overflow error has occurred.



The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/integer-overflow-or-underflow/integer-overflow-or-underflow-1/remediated-example).

## How is it detected?

Checks if there’s any numerical overflow or underflow.



    
