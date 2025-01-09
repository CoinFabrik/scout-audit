# Avoid DispatchError::Other()

## Description

- Category: `Error handling`
- Severity: `Enhancement`
- Detectors: [`avoid-dispatch-error-other`](https://github.com/CoinFabrik/scout-audit/tree/main/detectors/substrate-pallets/avoid-dispatcherror-other)
- Test Cases: [`avoid-dispatch-error-other-1`](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/avoid-dispatcherror-other)

Using `DispatchError::Other()` makes error handling challenging, particularly for developers working with smart contracts. The indiscriminate use of this error type makes it difficult to monitor and diagnose specific errors, impeding efficient troubleshooting and code improvement.

## Issue example

Consider the following function:

```rust
#[pallet::call_index(0)]
pub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {
    if increase_by > T::Balance::from(1000u32) {
        return Err(DispatchError::Other("increase_by is too large"));
    }

    let _sender = ensure_signed(origin)?;

    <Dummy<T>>::mutate(|dummy| {
        let new_dummy = dummy.map_or(increase_by, |d| d.saturating_add(increase_by));
        *dummy = Some(new_dummy);
    });

    Self::deposit_event(Event::AccumulateDummy {
        balance: increase_by,
    });

    Ok(())
}
```

In this code, using `DispatchError::Other("increase_by is too large")` creates a vague error message that does not clearly identify the problem. This generic error handling approach reduces the ability to effectively monitor and debug the code, hindering developers from quickly identifying and resolving the issue.

The vulnerable code example can be found [`here`](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/avoid-dispatcherror-other/vulnerable/vulnerable-1).

## Remediation

To improve error handling, use a specific error variant defined in your pallet. This way, the error is not only more descriptive but also tied to a well-defined variant, which makes it easier for developers to pinpoint the cause of a failure and address it efficiently.

```rust
#[pallet::call_index(0)]
pub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {
    if increase_by > T::Balance::from(1000u32) {
        return Err(Error::<T>::IncreaseByTooLarge.into());
    }

    let _sender = ensure_signed(origin)?;

    <Dummy<T>>::mutate(|dummy| {
        let new_dummy = dummy.map_or(increase_by, |d| d.saturating_add(increase_by));
        *dummy = Some(new_dummy);
    });

    Self::deposit_event(Event::AccumulateDummy {
        balance: increase_by,
    });

    Ok(())
}
```

The remediated code example can be found [`here`](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/avoid-dispatcherror-other/remediated/remediated-1).
