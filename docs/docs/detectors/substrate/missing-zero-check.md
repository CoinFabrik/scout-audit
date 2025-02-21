# Missing Zero Check

## Description

- Category: `Best Practices`
- Severity: `Minor`
- Detectors: [`missing-zero-check`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/substrate-pallets/missing-zero-check/src/lib.rs)

Failing to check for a zero value in the parameters of a function may lead to unnecessary operations, potentially increasing resource usage and reducing the efficiency of the function

## Why is this bad?

Failing to check if a Balance parameter is zero can lead to unintended side effects. For example, performing arithmetic operations with a zero balance might result in redundant storage writes or unnecessary event emissions.

## Issue example

Consider the following `Substrate pallet`:

```rust
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub fn set_balance(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let amount_u32: u32 = amount.try_into().unwrap_or(u32::MAX);
    let sender_balance = Self::balance_of(&who);

    ensure!(sender_balance >= amount_u32, "Insufficient balance");

    Self::deposit_event(Event::BalanceSet { who, value: amount });
    Ok(())
}
```

In this pallet, the set_balance function receives amount as a parameter, but it never checks if amount is zero.

## Remediated example

Check if the parameter can be zero

```rust
pub fn set_balance(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let sender_balance = Self::balance_of(&who);
    let amount_u32: u32 = amount.try_into().unwrap_or(u32::MAX);
    ensure!(sender_balance >= amount_u32, "Insufficient balance");

    if amount == Zero::zero() {
        return Err(Error::<T>::ZeroBalance.into());
    }
    Self::deposit_event(Event::BalanceSet { who, value: amount });
    Ok(())
}
```

## How is it detected?

This detector operates in three stages.

HIR Analysis (`EarlyLintPass`): In this stage, the detector analyzes the contract’s functions using the HIR representation. It identifies and stores all `extrinsic` functions in a vector of structs, allowing it to filter out cases where the vulnerability is not relevant.

MIR Analysis (`LateLintPass`): Here, the detector examines the MIR representation, checking whether any function parameter is of type `Balance`. If such a parameter is found, it is stored in a struct. The detector then looks for expressions that compare this variable with zero. If found, the parameter is removed from the struct, otherwise, it remains unchanged.

Final Check (`check_crate_post`): After all detections have completed, this stage reviews the stored Balance parameters and issues a warning for each remaining one.
