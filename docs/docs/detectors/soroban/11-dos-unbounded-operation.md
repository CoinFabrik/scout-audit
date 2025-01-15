# DoS unbounded operation

## Description 

- Category: `Denial of Service`
- Severity: `Medium`
- Detector: [`dos-unbounded-operation`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/dos-unbounded-operation)
- Test Cases: [`dos-unbounded-operation-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-1) [`dos-unbounded-operation-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-2) [`dos-unbounded-operation-3`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-3)

Each block in a Stellar Blockchain has an upper bound on the amount of gas that can be spent, and thus the amount computation that can be done. This is the Block Gas Limit. 

## Why is this bad? 

If the number of iterations is not limited to a specific range, it could potentially cause out of gas exceptions. If this happens, gas will leak, the transaction will fail, and there will be a risk of a potential attack on the contract.

## Issue example

In the following example, a contract has a function ´unsafe_loop_with_array´, which contains a for loop that iterates over a range of numbers from 0 to the lenght of the array ´unknown_array´. The issue is that if the length of the array is extremely large, it would cause the loop to execute many times, potentially leading to an unusable state of the contract.

Consider the following `Soroban` contract:

```rust
 pub fn unsafe_loop_with_array(unknown_array: BytesN<8>) -> u32 {
        let mut sum = 0;
        for i in 0..unknown_array.len() {
            sum += i;
        }
        sum
    }
```
The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-3/vulnerable-example).


## Remediated example

To solve this, instead of relying on an external parameter, we should introduce a known value directly into the loop.
```rust
 pub fn safe_loop_with_array() -> u64 {
        let mut sum = 0;
        let known_array = [0; 8];
        for i in 0..known_array.len() {
            sum += i;
        }
        sum as u64
    }
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-3/remediated-example).

## How is it detected?

This detector checks that when using for or while loops, their conditions limit the execution to a constant number of iterations.



    
