# Empty expect

## Description

- Category: `Best Practices`
- Severity: `Medium`
- Detectors: [`empty-expect`](https://github.com/CoinFabrik/scout-audit/tree/develop/detectors/rust/empty-expect)
- Test Cases: [`empty-expect-1`](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/empty-expect)

An empty `.expect()` creates a panic without any explanatory message, leaving developers without information to diagnose the error or trace its origin. This lack of clarity can lead to longer resolution times, poor maintenance practices, and potentially even security issues if sensitive operations fail without explanation.

## Issue example

Consider the following function:

```rust
#[pallet::call_index(0)]
pub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let example_storage = ExampleStorage::<T>::get();
    if example_storage.is_some() {
        let value = example_storage.expect("");
        Self::deposit_event(Event::UnsafeGetStorage { who, value });
    }
    Ok(())
}
```

In the the `unsafe_get_storage` function, the line `example_storage.expect("")` uses an empty string in the `.expect()` method. This is problematic because it provides no context for the panic that occurs if the `Option` is `None`. If a panic is triggered, debugging the issue becomes significantly harder, as there is no information to explain what went wrong or why the code expected a value in the storage.

The vulnerable code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/empty-expect/vulnerable/vulnerable-1).

## Remediation

Make the `.expect()` method include a descriptive message. This change ensures that if the `Option` is `None` and a panic occurs, the message clearly explains the problem.

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

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/empty-expect/remediated/remediated-1).
