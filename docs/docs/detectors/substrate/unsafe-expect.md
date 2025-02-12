# Unsafe expect

## Description

- Category: `Validations and error handling`
- Severity: `Minor`
- Detectors: [`unsafe-expect`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/substrate-pallets/unsafe-expect/src/lib.rs)

In Rust, the `expect` method is often used for error handling. It returns the contained `Ok` value for a `Result` or `Some` value for an `Option`. If an error occurs, it calls `panic!` with a provided error message.

## Why is this bad?

`.expect()` might panic if the result value is an error or `None`. It is recommended to avoid the panic of a contract because it stops its execution, which might lead the contract to an inconsistent state if the panic occurs in the middle of state changes. Additionally, the panic could cause a transaction to fail.

## Issue example

Consider the following `Substrate Pallet`:

```rust
#[pallet::call_index(0)]
        pub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let value = ExampleStorage::<T>::get().expect("Storage is not initialized");
            Self::deposit_event(Event::UnsafeGetStorage { who, value });
            Ok(())
        }
```

In this contract, the `unsafe_get_storage` function uses the expect method to retrieve the storage. If the storage is not initialized, the contract will panic and halt execution, which could be exploited maliciously to disrupt the pallet's operation.

## Remediated example

Ensure that expect won't fail before using it. You can achieve this by calling expect inside an if statement that checks if `example_storage` is `Some`, or by returning an error if it is `None`.

```rust
#[pallet::call_index(0)]
        pub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let example_storage = ExampleStorage::<T>::get();
            if example_storage.is_some() {
                let value = example_storage.expect("Storage is not initialized");
                Self::deposit_event(Event::UnsafeGetStorage { who, value });
            }
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
            let value = example_storage.expect("Storage is not initialized");
            Self::deposit_event(Event::UnsafeGetStorage { who, value });
            Ok(())
        }
```

## How is it detected?

Checks for usage of `.expect()`.
