# Assert violation

## Description

- Category: `Validations and error handling`
- Severity: `Enhancement`
- Detectors: [assert-violation](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/rust/assert-violation/src/lib.rs)

The `assert!` macro is used in Rust to ensure that a certain condition holds true at a certain point in your code.

### Why is this bad?

The `assert!` macro can cause the contract to panic. It is recommended to avoid this, because it stops its execution, which might lead the contract to an inconsistent state if the panic occurs in the middle of state changes. Additionally, the panic could cause a transaction to fail.

### Example

Consider the following substrate pallet:

```rust
pub fn unsafe_check_balance(origin: OriginFor<T>, amount: u32) -> DispatchResult {
    let who = ensure_signed(origin)?;
    assert!(
        BalanceStorage::<T>::get().unwrap_or(0) >= amount,
        "Insufficient balance"
    );

    Self::deposit_event(Event::BalanceChecked { who, amount });
    Ok(())
}
```

## Remediated example

Consider using a proper error and return it:

```rust
pub fn unsafe_check_balance(origin: OriginFor<T>, amount: u32) -> DispatchResult {
    let who = ensure_signed(origin)?;
    if BalanceStorage::<T>::get().unwrap_or(0) < amount {
        return Err(Error::<T>::InvalidBalance.into());
    }

    Self::deposit_event(Event::BalanceChecked { who, amount });
    Ok(())
}
```

## How is it detected?

Checks for `assert!` macro usage.
