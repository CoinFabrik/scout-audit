# Avoid core::mem::forget usage

### What it does

Checks for `core::mem::forget` usage.

### Why is this bad?

This is a bad practice because it can lead to memory leaks, resource leaks and logic errors.

### Example

```rust
pub fn forget_something(n: WithoutCopy) -> u64 {
    core::mem::forget(n);
    0
}
```

Use instead:

```rust
pub fn forget_something(n: WithoutCopy) -> u64 {
    let _ = n;
    0
}
```

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-core-mem-forget).
