# Divide before multiply

## Description 

- Category: `Arithmetic`
- Severity: `Medium`
- Detectors: [`divide-before-multiply`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/divide-before-multiply)
- Test Cases: [`divide-before-multiply-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/divide-before-multiply/divide-before-multiply-1) [`divide-before-multiply-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/divide-before-multiply/divide-before-multiply-2) [`divide-before-multiply-3`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/divide-before-multiply/divide-before-multiply-3)

In Rust, the order of operations can influence the precision of the result, especially in integer arithmetic. 

## Why is this bad? 

Performing a division operation before a multiplication can lead to a loss of precision as division between integers might return zero. 

## Issue example 

Consider the following `Soroban` contract:

```rust

 pub fn split_profit(percentage: u64, total_profit: u64) -> u64 {
    (percentage / 100) * total_profit
}

```

In this contract, the `split_profit` function divides the `percentage` by `100` before multiplying it with `total_profit`. This could lead to a loss of precision if `percentage` is less than `100` as the division would return `0`. This could lead to incorrect calculations and potential financial loss in a real-world smart contract.


The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/divide-before-multiply/divide-before-multiply-1/vulnerable-example).


## Remediated example

Reverse the order of operations to ensure multiplication occurs before division.

```rust

 pub fn split_profit(&self, percentage: u64, total_profit: u64) -> u64 {
            (percentage * total_profit) / 100
        }
        
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/divide-before-multiply/divide-before-multiply-1/remediated-example).

## How is it detected?

Checks the existence of a division before a multiplication.

## References

[Rust documentation: `Integer Division`](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators)
    
