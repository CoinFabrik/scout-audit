# Uncached storage modification

## Description

- Category: `Known Bugs`
- Severity: `Medium`
- Detector: [`uncached-storage-modification`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/uncached-storage-modification)
- Test Cases: [`uncached-storage-modification-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/uncached-storage-modification/uncached-storage-modification-1)

When a storage value is read into a local variable, mutated, and the same key is read again without writing the updated value back, the second read returns stale data. This can lead to incorrect state transitions and logic errors.

## Why is this bad?

A stale re-read can cause logic that depends on the updated value to use outdated data. This is a cache coherence bug that can silently corrupt state.

## Issue example

```rust
let mut value: i32 = env
    .storage()
    .persistent()
    .get(&DataKey::Value(key.clone()))
    .unwrap_or(0);

value += 1;

let reread: i32 = env
    .storage()
    .persistent()
    .get(&DataKey::Value(key))
    .unwrap_or(0);
```

In this example, the value is modified in memory but never written back to storage before a second read.

## Remediated example

```rust
let mut value: i32 = env
    .storage()
    .persistent()
    .get(&DataKey::Value(key.clone()))
    .unwrap_or(0);

value += 1;

env.storage()
    .persistent()
    .set(&DataKey::Value(key.clone()), &value);

let reread: i32 = env
    .storage()
    .persistent()
    .get(&DataKey::Value(key))
    .unwrap_or(0);
```

The modified value is written back to storage before the key is read again.

## How is it detected?

The detector tracks storage reads into locals, marks them dirty on mutation, and reports when the same key is read again without a write in between.
