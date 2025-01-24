# Saturating arithmetic

## Description

- Issue Category: `Arithmetic`
- Issue Severity: `Critical`
- Detectors: [`saturating-arithmetic`](https://github.com/CoinFabrik/scout-audit/tree/main/detectors/substrate-pallets/saturating-arithmetic/src/lib.rs)

Saturating arithmetic operations adjust the result to the maximum or minimum value allowed by the data type instead of causing an overflow. While this behavior prevents crashes, it can produce incorrect results. Consider checked arithmetic instead

## Why is it bad?

Saturating arithmetic clamps the result to the representation limit for the data type instead of overflowing. By doing this, it can generate logical errors in calculations, leading to unintended behavior without throwing any error or warning.

## Issue example

Consider the following `Substrate pallet`:

```rust
pub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {
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

By calling `d.saturating_add(increase_by)`, it can lead tu unexpected behavior.

## Remediated example

Using checked arithmetic, which explicitly handles overflows by returning an error or panic, ensures that unexpected conditions are caught and addressed.

```rust
pub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {
    let _sender = ensure_signed(origin)?;

    let mut error = Ok(());
    <Dummy<T>>::mutate(|dummy| {
        let new_dummy = dummy.map_or(Some(increase_by), |d| d.checked_add(&increase_by));
        if new_dummy.is_none() {
            error = Err(Error::<T>::IntegerOverflow);
            return;
        }
        *dummy = new_dummy;
    });

    error?;

    Self::deposit_event(Event::AccumulateDummy {
        balance: increase_by,
    });

    Ok(())
}
```

## How is it detected?

Checks the use of saturating calls such as: `saturating_add`, `saturating_mul`, `saturating_dec`, etc.
