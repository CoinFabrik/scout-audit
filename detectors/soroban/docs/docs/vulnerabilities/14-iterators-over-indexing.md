# Iterators over indexing

## Description

- Vulnerability Category: `Best practices`
- Vulnerability Severity: `Enhacement`
- Detectors: [`iterators-over-indexing`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/iterators-over-indexing)
- Test Cases: [`iterators-over-indexing-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/iterators-over-indexing/iterators-over-indexing-1)

Iterating with hardcoded indexes is slower than using an iterator. Also, if the index is out of bounds, it will panic.

This could lead to potential integer overflow vulnerabilities, which would trigger a panic in debug builds or wrap in release mode, jeopardizing the integrity and security of the smart contract. Additionally, failing to verify the existence of data in storage before operations could result in unexpected errors or runtime failures, compromising the reliability of the contract execution.

## Exploit Scenario

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

The vulnerable code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/iterators-over-indexing/iterators-over-indexing-1/vulnerable-example).

## Remediation

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

The remediated code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/iterators-over-indexing/iterators-over-indexing-1/remediated-example).