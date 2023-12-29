# DoS unbounded operation

### What it does

This detector checks that when using for or while loops, their conditions limit the execution to a constant number of iterations.

### Why is this bad?

If the number of iterations is not limited to a specific range, it could potentially cause out of gas exceptions.

### Known problems

False positives are to be expected when using variables that can only be set using controlled flows that limit the values within acceptable ranges.

### Example

```rust
pub fn unrestricted_loop(for_loop_count: u64) -> u64 {
    let mut count = 0;
    for i in 0..for_loop_count {
        count += i;
    }
    count
}
```

Use instead:

```rust
const FIXED_COUNT: u64 = 1000;

pub fn restricted_loop_with_const() -> u64 {
    let mut sum = 0;
    for i in 0..FIXED_COUNT {
        sum += i;
    }
    sum
}
```

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/dos-unbounded-operation).
