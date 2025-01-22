# Avoid unsafe block

## Description

- Category: `Validations and error handling`
- Severity: `Critical`
- Detector: [`avoid-unsafe-block`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/rust/avoid-unsafe-block/src/lib.rs)

The use of unsafe blocks in Rust is generally discouraged due to the potential risks it poses to the safety and reliability of the code. Rust's primary appeal lies in its ability to provide memory safety guarantees, which are largely enforced through its ownership and type systems. When you enter an unsafe block, you're effectively bypassing these safety checks. These blocks require the programmer to manually ensure that memory is correctly managed and accessed, which is prone to human error and can be challenging even for experienced developers. Therefore, unsafe blocks should only be used when absolutely necessary and when the safety of the operations within can be assured.

## Why is this bad?

`unsafe` blocks should not be used unless absolutely necessary. The use of unsafe blocks in Rust is discouraged because they bypass Rust's memory safety checks, potentially leading to issues like undefined behavior and security vulnerabilities.

## Issue example

Consider the following `substrate pallet`:

```rust
#[pallet::call_index(0)]
        pub fn process_data(origin: OriginFor<T>, input: u8) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let result = unsafe {
                let ptr: *const u8 = &input;
                let value = *ptr;
                value.rotate_left(2).wrapping_add(1)
            };

            DataStorage::<T>::set(result);

            Self::deposit_event(Event::DataProcessed { who, value: result });

            Ok(())
        }
```

In this example we can see that it creates a raw pointer named `ptr`. Then `value` dereferences the raw pointer. This directly accesses the memory location and calls the `rotate_left` method on the value stored at that location.

Raw pointers bypass Rust's type safety system and memory management features. If something goes wrong with the calculations or the value of input, dereferencing the pointer could lead to a memory access violations or undefined behavior.

## Remediated example

By removing the raw pointer, the following version eliminates the issue associated with dereferencing memory in an unsafe way. Rust's type safety checks ensure memory is accessed correctly, preventing the potential issues mentioned earlier.

```rust
#[pallet::call_index(0)]
        pub fn process_data(origin: OriginFor<T>, input: u8) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let result = input.rotate_left(2).wrapping_add(1);

            DataStorage::<T>::set(result);

            Self::deposit_event(Event::DataProcessed { who, value: result });

            Ok(())
        }
```

## How is it detected?

Checks for usage of `unsafe` blocks.
