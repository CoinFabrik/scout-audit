# Unsafe unwrap

## Description

- Category: `Validations and error handling`
- Severity: `Minor`
- Detectors: [`unsafe-unwrap`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/substrate-pallets/unsafe-unwrap/src/lib.rs)

In Rust, the `unwrap` method is commonly used for error handling. It retrieves the inner value of an `Option` or `Result`. If an error or `None` occurs, it calls `panic!` without a custom error message.

## Why is this bad?

`.unwrap()` might panic if the result value is an error or `None`. It is recommended to avoid the panic of a pallet because it stops its execution, which might lead the pallets to an inconsistent state if the panic occurs in the middle of state changes. Additionally, the panic could cause a transaction to fail.

## Issue example

Consider the following `Substrate pallet`:

```rust
#[pallet::call_index(0)]
        pub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let value = ExampleStorage::<T>::get().unwrap();
            Self::deposit_event(Event::UnsafeGetStorage { who, value });
            Ok(())
        }
```

In this example, the `unsafe_get_storage` function uses the `unwrap` method to save the result of the `ExampleStorage` struct. If the function returns `Err`, the contract will panic and halt execution, potentially leading to malicious exploitation to disrupt the contract's operation.

## Remediated example

Instead of using `unwrap`, use a safer method for error handling like `unwrap_or_default`, or ensure that `.get()` is always `some` by adding a conditional.

```rust
#[pallet::call_index(0)]
        pub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let value = ExampleStorage::<T>::get().unwrap_or_default();
            Self::deposit_event(Event::UnsafeGetStorage { who, value });
            Ok(())
        }
```

```rust
#[pallet::call_index(0)]
        pub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let example_storage = ExampleStorage::<T>::get();
            if example_storage.is_none() {
                return Err(Error::<T>::NotInitialized.into());
            }
            let value = example_storage.unwrap();
            Self::deposit_event(Event::UnsafeGetStorage { who, value });
            Ok(())
        }
```

## How is it detected?

Checks for usage of .unwrap()
