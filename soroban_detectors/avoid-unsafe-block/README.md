# Avoid unsafe block

### What it does

Checks for usage of `unsafe` blocks.

### Why is this bad?

`unsafe` blocks should not be used unless absolutely necessary. The use of unsafe blocks in Rust is discouraged because they bypass Rust's memory safety checks, potentially leading to issues like undefined behavior and security vulnerabilities.

### Example

 ```rust
pub fn unsafe_function(n: u64) -> u64 {
    unsafe {
        let mut i = n as f64;
        let mut y = i.to_bits();
        y = 0x5fe6ec85e7de30da - (y >> 1);
        i = f64::from_bits(y);
        i *= 1.5 - 0.5 * n as f64 * i * i;
        i *= 1.5 - 0.5 * n as f64 * i * i;

        let result_ptr: *mut f64 = &mut i;
        let result = *result_ptr;

        result.to_bits()
     }
}
```

Use instead:

 ```rust
pub fn unsafe_function(n: u64) -> u64 {
        let mut i = n as f64;
        let mut y = i.to_bits();
        y = 0x5fe6ec85e7de30da - (y >> 1);
        i = f64::from_bits(y);
        i *= 1.5 - 0.5 * n as f64 * i * i;
        i *= 1.5 - 0.5 * n as f64 * i * i;
        result.to_bits()
}
```

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-unsafe-block).
