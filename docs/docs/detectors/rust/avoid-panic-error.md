# Avoid panic error

## Description

- Category: `Validations and error handling`
- Severity: `Enhancement`
- Detector: [`avoid-panic-error`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/substrate-pallets/avoid-panic-error/src/lib.rs)

The panic! macro is used to stop execution when a condition is not met. This is useful for testing and prototyping, but should be avoided in production code.

Using `Result` as return type for functions that can fail is the idiomatic way to handle errors in Rust. The `Result` type is an enum that can be either `Ok` or `Err`. The `Err` variant can contain an error message. The `?` operator can be used to propagate the error message to the caller.

This way, the caller can decide how to handle the error, although the state of the contract is always reverted on the callee.

## Why is this bad?

The usage of `panic!` is not recommended because it will stop the execution of the caller contract. This could lead the contract to an inconsistent state if the execution stops in the middle of state changes. Additionally, if execution stops, it could cause a transaction to fail.

## Issue example

Consider the following snippet code:

```rust
pub fn unsafe_check_value(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let stored_value = Value::<T>::get().unwrap_or_default();
            if stored_value < threshold {
                panic!("Value is too low!");
            }

            Self::deposit_event(Event::ValueChecked {
                who,
                value: stored_value,
            });
            Ok(())
        }
```

This function panics if `stored_value` is less than `threshold`, disallowing the caller to handle the error in a different way, and completely stopping execution of the caller contract.
The usage of panic! in this example, is not recommended because it will stop the execution of the caller contract.

## Remediated example

A possible remediation goes as follows:

```rust
pub fn unsafe_check_value(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let stored_value = Value::<T>::get().unwrap_or_default();
            if stored_value < threshold {
                return Err(Error::<T>::ValueTooLow.into());
            }

            Self::deposit_event(Event::ValueChecked {
                who,
                value: stored_value,
            });
            Ok(())
        }
```

## How is it detected?

Checks the use of the macro `panic!`.
