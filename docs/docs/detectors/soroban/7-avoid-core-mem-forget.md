# Avoid core::mem::forget usage

## Description 

- Category: `Best practices`
- Severity: `Enhancement`
- Detector:  [`avoid-core-mem-forget`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-core-mem-forget)
- Test Cases: [`avoid-core-mem-forget-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-core-mem-forget/avoid-core-mem-forget-1)


The `core::mem::forget` function is used to forget about a value without running its destructor. 

## Why is this bad? 

Using this function is a bad practice because this can lead to memory leaks, resource leaks and logic errors.

## Issue example 

Consider the following `Soroban` contract:

```rust

 pub fn forget_something(n: WithoutCopy) -> u64 {
        core::mem::forget(n);
        0
    }	
```

The problem arises from the use of the `core::mem::forget` function. 

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-core-mem-forget/avoid-core-mem-forget-1/vulnerable-example).


## Remediated example

Use the pattern `let _ = n;` or the `.drop()` method instead of `core::mem::forget(n);`.

## How is it detected?

Checks for `core::mem::forget` usage.

## References

- [Memory Management](https://docs.alephzero.org/aleph-zero/security-course-by-kudelski-security/ink-developers-security-guideline#memory-management)
    
