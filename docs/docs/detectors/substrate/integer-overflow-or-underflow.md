# Integer overflow or underflow

## Description

- Category: `Arithmetic`
- Severity: `Critical`
- Detectors: [`integer-overflow-or-underflow`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/substrate-pallets/integer-overflow-or-underflow/src/lib.rs)

In Rust, arithmetic operations can result in a value that falls outside the allowed numerical range for a given type. When the result exceeds the maximum value of the range, it's called an overflow, and when it falls below the minimum value of the range, it's called an underflow.

## Why is this bad?

If there are arithmetic operations with overflow or underflow problems, and if errors are not handled correctly, incorrect results will be generated, bringing potential problems for the contract. Additionally, these types of errors can allow attackers to drain a contract’s funds or manipulate its logic.

## Issue example

Consider the following `Substrate pallet`:

```rust
#[pallet::call_index(0)]
        pub fn accumulate_dummy(origin: OriginFor<T>, increase_by: T::Balance) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            <Dummy<T>>::mutate(|dummy| {
                let new_dummy = dummy.map_or(increase_by, |d| d + increase_by);
                *dummy = Some(new_dummy);
            });
            Self::deposit_event(Event::AccumulateDummy {
                balance: increase_by,
            });
            Ok(())
        }
```

In this example, an operation is performed on two u32 (`d` and `increase_by`) values without any safeguards against overflow if it occurs.

## Remediated example

Consider using safe operations to prevent an overflow

```rust
#[pallet::call_index(0)]
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

In this example, the `saturating_add` method is used to perform the addition. It returns the sum if no overflow occurs; otherwise, it returns `None`, with an OverflowError variant indicating that an overflow error has occurred.

## How is it detected?

Checks if there’s any numerical overflow or underflow.
