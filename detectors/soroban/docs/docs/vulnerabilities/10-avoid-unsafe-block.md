# Avoid unsafe block

## Description
- Vulnerability Category: `Validations and error handling`
- Severity: `Critical`
- Detectors: [`avoid-unsafe-block`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-unsafe-block)
- Test Cases: [`avoid-unsafe-block-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-unsafe-block/avoid-unsafe-block-1)

The use of unsafe blocks in Rust is generally discouraged due to the potential risks it poses to the safety and reliability of the code. Rust's primary appeal lies in its ability to provide memory safety guarantees, which are largely enforced through its ownership and type systems. When you enter an unsafe block, you're effectively bypassing these safety checks. This can lead to various issues, such as undefined behavior, memory leaks, or security vulnerabilities. These blocks require the programmer to manually ensure that memory is correctly managed and accessed, which is prone to human error and can be challenging even for experienced developers. Therefore, unsafe blocks should only be used when absolutely necessary and when the safety of the operations within can be assured.

## Exploit Scenario
In this example we can see that it creates a raw pointer named `result_ptr`. Then `(*result_ptr).to_bits()` dereferences the raw pointer. This directly accesses the memory location and calls the `to_bits` method on the value stored at that location.

Raw pointers bypass Rust's type safety system and memory management features. If something goes wrong with the calculations or the value of n, dereferencing the pointer could lead to a memory access violations or undefined behavior.

```rust
#[contractimpl]
impl AvoidUnsafeBlock {
    pub fn unsafe_function(n: u64) -> u64 {
        unsafe {
            let mut i = n as f64;
            let mut y = i.to_bits();
            y = 0x5fe6ec85e7de30da - (y >> 1);
            i = f64::from_bits(y);
            i *= 1.5 - 0.5 * n as f64 * i * i;
            i *= 1.5 - 0.5 * n as f64 * i * i;

            let result_ptr: *mut f64 = &mut i;

            (*result_ptr).to_bits()
        }
    }
}
```

The vulnerable code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-unsafe-block/avoid-unsafe-block-1/vulnerable-example).

## Remediation
By removing the raw pointer, the following version eliminates the vulnerability associated with dereferencing memory in an unsafe way. Rust's type safety checks ensure memory is accessed correctly, preventing the potential issues mentioned earlier. 

```rust
#[contractimpl]
impl AvoidUnsafeBlock {
    pub fn unsafe_function(n: u64) -> u64 {
        let mut i = n as f64;
        let mut y = i.to_bits();
        y = 0x5fe6ec85e7de30da - (y >> 1);
        i = f64::from_bits(y);
        i *= 1.5 - 0.5 * n as f64 * i * i;
        i *= 1.5 - 0.5 * n as f64 * i * i;
        i.to_bits()
    }
}
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-unsafe-block/avoid-unsafe-block-1/remediated-example).
