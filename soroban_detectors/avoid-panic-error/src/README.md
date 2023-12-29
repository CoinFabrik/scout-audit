# Panic error

### What it does

The `panic!` macro is used to stop execution when a condition is not met.
This is useful for testing and prototyping, but should be avoided in production code.

### Why is this bad?

The usage of `panic!` is not recommended because it will stop the execution of the caller contract.

### Known problems

While this linter detects explicit calls to `panic!`, there are some ways to raise a panic such as `unwrap()` or `expect()`.

### Example

```rust
pub fn add(env: Env, value: u32) -> u32 {
    let storage = env.storage().instance();
    let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);
    match count.checked_add(value) {
        Some(value) => count = value,
        None => panic!("Overflow error"),
    }
    storage.set(&COUNTER, &count);
    storage.extend_ttl(100, 100);
    count
}
```

Use instead:

```rust
pub fn add(env: Env, value: u32) -> Result<u32, Error> {
    let storage = env.storage().instance();
    let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);
    match count.checked_add(value) {
        Some(value) => count = value,
        None => return Err(Error::OverflowError),
    }
    storage.set(&COUNTER, &count);
    storage.extend_ttl(100, 100);
    Ok(count)
}
```

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-panic-error).
