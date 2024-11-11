# Iterators over indexing

## Description 

- Category: `Best practices`
- Severity: `Enhancement`
- Detector: [`iterators-over-indexing`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/iterators-over-indexing)
- Test Cases: [`iterators-over-indexing-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/iterators-over-indexing/iterators-over-indexing-1)

In Rust, sequences can be traversed using iterators or direct indexing. However, the least efficient way is through direct indexing.

## Why is this bad? 

When you iterate over a data structure with fixed limits in a Soroban smart contract, exceeding those limits can cause the contract to panic, potentially leading to errors or unexpected behavior.

## Issue example 

Consider the following `Soroban` contract:

```rust
   pub fn sum(e: Env) -> Result<i32, Error> {
        let mut ret = 0_i32;
        let vec = e
            .storage()
            .instance()
            .get::<DataKey, Vec<i32>>(&DataKey::Data)
            .ok_or(Error::NoData)?;
        for i in 0..4 {
            ret = ret
                .checked_add(vec.get(i).ok_or(Error::NoData)?)
                .ok_or(Error::IntegerOverflow)?;
        }
        Ok(ret)
    }
```
The problem arises in the for loop. If `vec` has less than 4 elements, the contract will panic.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/iterators-over-indexing/iterators-over-indexing-1/vulnerable-example).

## Remediated example

```rust
     pub fn sum(e: Env) -> Result<i32, Error> {
        let mut ret = 0_i32;
        let vec = e
            .storage()
            .instance()
            .get::<DataKey, Vec<i32>>(&DataKey::Data)
            .ok_or(Error::NoData)?;
        for i in vec {
            ret = ret.checked_add(i).ok_or(Error::IntegerOverflow)?;
        }
        Ok(ret)
    }
```

Instead of using a fixed loop, iterate through the vector itself using `for i in vec`. This ensures the loop iterates only for valid elements present in the vector.

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/iterators-over-indexing/iterators-over-indexing-1/remediated-example).

## How is it detected?

It warns if the for loop uses indexing instead of iterator. If the indexing goes to length it will not raise a warning.
