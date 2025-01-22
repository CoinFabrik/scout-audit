# Divide before multiply

## Description

- Category: `Arithmetic`
- Severity: `Medium`
- Detectors: [`divide-before-multiply`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/rust/divide-before-multiply/src/lib.rs)

In Rust, the order of operations can influence the precision of the result, especially in integer arithmetic.

## Why is this bad?

Performing a division operation before a multiplication can lead to a loss of precision as division between integers might return zero.

## Issue example

Consider the following `Substrate pallet`:

```rust
#[pallet::call_index(0)]
        pub fn accumulate_dummy(
            origin: OriginFor<T>,
            increase_by: T::Balance,
            numerator: T::Balance,
            denominator: T::Balance,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            <Dummy<T>>::mutate(|dummy| {
                let new_dummy = dummy.map_or(increase_by, |d| {
                    d.saturating_add(increase_by / denominator * numerator)
                });
                *dummy = Some(new_dummy);
            });

            Self::deposit_event(Event::AccumulateDummy {
                balance: increase_by,
            });

            Ok(())
        }
```

In this contract, the `accumulate_dummy` function creates a Dummy struct array by adding each element with the calculation of `increase_by / denominator * numerator`. This last calculation divides the `increase_by` by `denominator` before multiplying it with `numerator`. This could lead to a loss of precision if `increase_by` is less than `denominator` as the division would return `0`. This could lead to incorrect calculations and potential financial loss in a real-world smart contract.

## Remediated example

Reverse the order of operations to ensure multiplication occurs before division.

```rust
#[pallet::call_index(0)]
        pub fn accumulate_dummy(
            origin: OriginFor<T>,
            increase_by: T::Balance,
            numerator: T::Balance,
            denominator: T::Balance,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            <Dummy<T>>::mutate(|dummy| {
                let new_dummy = dummy.map_or(increase_by, |d| {
                    d.saturating_add(increase_by * numerator / denominator)
                });
                *dummy = Some(new_dummy);
            });

            Self::deposit_event(Event::AccumulateDummy {
                balance: increase_by,
            });

            Ok(())
        }
```

## How is it detected?

Checks the existence of a division before a multiplication.

## References

[Rust documentation: `Integer Division`](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators)
