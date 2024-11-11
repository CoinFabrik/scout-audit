# Incorrect exponentiation

## Description

- Vulnerability Category: `Arithmetic`
- Vulnerability Severity: `Critical`
- Detectors: [`incorrect-exponentiation`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/incorrect-exponentiation)
- Test Cases: [`incorrect-exponentiation-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/incorrect-exponentiation/incorrect-exponentiation-1)

The operator `^` is not an exponential operator, it is a bitwise XOR. Make sure to use `pow()` instead for exponentiation. In case of performing a XOR operation, use `.bitxor()` for clarity.

## Why is it bad?

It can produce unexpected behaviour in the smart contract.

## Issue example

In the following example, the `^` operand is being used for exponentiation. But in Rust, `^` is the operand for an XOR operation. If misused, this could lead to unexpected behaviour in our contract.

Consider the following `Soroban` contract:

```rust
   pub fn exp_data_3(e: Env) -> u128 {
        let mut data = e.storage()
        .instance()
        .get::<DataKey, u128>(&DataKey::Data)
        .expect("Data not found");
        
        data ^= 3;
        data
    }
```

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/incorrect-exponentiation/incorrect-exponentiation-1/vulnerable-example).

## Remediated example

A possible solution is to use the method `pow()`. But, if a XOR operation is wanted, `.bitxor()` method is recommended.

```rust
    pub fn exp_data_3(e: Env) -> u128 {
        let data = e.storage()
        .instance()
        .get::<DataKey, u128>(&DataKey::Data)
        .expect("Data not found");

        data.pow(3)
    }
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/incorrect-exponentiation/incorrect-exponentiation-1/remediated-example).

## How is it detected?

Warns about `^` being a `bit XOR` operation instead of an exponentiation.  


## References

- https://doc.rust-lang.org/std/ops/trait.BitXor.html
