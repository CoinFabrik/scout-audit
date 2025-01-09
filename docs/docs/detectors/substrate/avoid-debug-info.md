# Avoid debug!() info!()

## Description

- Category: `Best Practices`
- Severity: `Minor`
- Detectors: [`avoid-debug-info`](https://github.com/CoinFabrik/scout-audit/tree/develop/detectors/substrate-pallets/avoid-debug-info)
- Test Cases: [`avoid-debug-info-1`](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/avoid-debug-info)

The use of debugging macros, such as `debug!()` and `info!()`, is useful during development and testing; however, these macros are not recommended for production and are considered a bad practice. Additionally, each operation that stores data in memory requires the virtual machine to perform additional work, which increases the gas costs needed for the transaction. Instead, consider using events emitting to log relevant data more efficiently and reduce unnecessary gas costs.

## Issue example

Consider the following function:

```rust
#[pallet::call_index(0)]
pub fn unsafe_check_value(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
    let who = ensure_signed(origin)?;

    let stored_value = Value::<T>::get().unwrap_or_default();

    debug!(
        "Stored value: {:?}, Threshold: {:?}",
        stored_value, threshold
    );
    info!("Consider providing a threshold lower than the actual stored value");

    Self::deposit_event(Event::ValueChecked {
        who,
        value: stored_value,
    });
    Ok(())
}
```

The `unsafe_check_value` function logs data and provides a suggestion using `debug!()` and `info!()` macros. These macros, while helpful during development, are inefficient in production because they increase gas costs due to unnecessary resource consumption.

The vulnerable code example can be found [`here`](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/avoid-debug-info/vulnerable/vulnerable-1).

## Remediation

Replace the macros with structured events to reduce gas costs in production. If the `stored_value` is less than the threshold, emit a `ValueTooLow` event with the relevant details and return an error. This ensures proper handling of invalid conditions while maintaining efficiency and aligning the code with best practices for production environments.

```rust
#[pallet::call_index(0)]
pub fn unsafe_check_value(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
    let who = ensure_signed(origin)?;

    let stored_value = Value::<T>::get().unwrap_or_default();
    if stored_value < threshold {
        Self::deposit_event(Event::ValueTooLow {
            who: who.clone(),
            stored_value,
            threshold,
        });
        return Err(Error::<T>::ValueTooLow.into());
    }
    Self::deposit_event(Event::ValueChecked {
        who,
        value: stored_value,
    });
    Ok(())
}
```

The remediated code example can be found [`here`](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/avoid-debug-info/remediated/remediated-1).
