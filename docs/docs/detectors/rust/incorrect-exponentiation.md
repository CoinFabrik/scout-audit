# Incorrect exponentiation

## Description

- Issue Category: `Arithmetic`
- Issue Severity: `Critical`
- Detectors: [`incorrect-exponentiation`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/rust/incorrect-exponentiation/src/lib.rs)

The operator `^` is not an exponential operator, it is a bitwise XOR. Make sure to use `pow()` instead for exponentiation. In case of performing a XOR operation, use `.bitxor()` for clarity.

## Why is it bad?

It can produce unexpected behaviour in the smart contract.

## Issue example

In the following example, the `^` operand is being used for exponentiation. But in Rust, `^` is the operand for an XOR operation. If misused, this could lead to unexpected behaviour in our contract.

Consider the following `Substrate pallet`:

```rust
#[pallet::call_index(0)]
        pub fn set_balance(origin: OriginFor<T>, new_value: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let calculated_value = new_value ^ 3;
            Value::<T>::put(calculated_value);
            Self::deposit_event(Event::BalanceSet {
                who,
                value: calculated_value,
            });
            Ok(())
        }
```

## Remediated example

A possible solution is to use the method `pow()`. But, if a XOR operation is wanted, `.bitxor()` method is recommended.

```rust
#[pallet::call_index(0)]
pub fn set_balance(origin: OriginFor<T>, new_value: u32) -> DispatchResult {
let who = ensure_signed(origin)?;
let calculated_value = new_value.pow(3);
Value::<T>::put(calculated_value);
Self::deposit_event(Event::BalanceSet {
who,
value: calculated_value,
});
Ok(())
}
```

## How is it detected?

Warns about `^` being a `bit XOR` operation instead of an exponentiation.
