# Divide before multiply

## Description

- Category: `Arithmetic`
- Severity: `Medium`
- Detectors: [`divide-before-multiply`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/rust/divide-before-multiply/src/lib.rs)

In Rust, the order of operations can influence the precision of the result, especially in integer arithmetic.

## Why is this bad?

Performing a division operation before a multiplication can lead to a loss of precision as division between integers might return zero.

## Issue example

Consider the following snippet code:

```rust
 pub fn split_profit(percentage: u64, total_profit: u64) -> u64 {
    (percentage / 100) * total_profit
}
```

In this contract, the `split_profit` function divides the `percentage` by `100` before multiplying it with `total_profit`. This could lead to a loss of precision if `percentage` is less than `100` as the division would return `0`. This could lead to incorrect calculations and potential financial loss in a real-world smart contract.

## Remediated example

Reverse the order of operations to ensure multiplication occurs before division.

```rust
pub fn split_profit(&self, percentage: u64, total_profit: u64) -> u64 {
    (percentage * total_profit) / 100
}
```

## How is it detected?

Checks the existence of a division before a multiplication.

## References

[Rust documentation: `Integer Division`](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators)
