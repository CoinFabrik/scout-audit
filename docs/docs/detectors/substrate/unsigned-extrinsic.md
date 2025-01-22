# Unsigned extrinsic

## Description

- Category: `DoS`
- Severity: `Critical`
- Detectors: [`unsigned-extrinsic`](https://github.com/CoinFabrik/scout-audit/tree/develop/detectors/substrate-pallets/unsigned-extrinsic)
- Test Cases: [`unsigned-extrinsic-1`](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/unsigned-extrinsic)

Unsigned extrinsics allow transactions to be submitted without any associated fees or signatures. This can be exploited by malicious actors to flood the network with transactions at no cost, potentially causing denial of service. Consider using signed extrinsics with appropriate fee mechanisms unless there's a specific security reason for allowing unsigned transactions.

## Issue example

Consider the following function:

```rust
#[pallet::call(weight(<T as Config>::WeightInfo))]
impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    pub fn unsafe_call(origin: OriginFor<T>) -> DispatchResult {
        ensure_none(origin)?;

        Ok(())
    }
}
```

The `unsafe_call` function uses `ensure_none(origin)?`, which allows only unsigned transactions (i.e., where the origin is `None`). While this might seem harmless for internal runtime logic, it becomes problematic if this function is inadvertently exposed or misused.

The vulnerable code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/unsigned-extrinsic/vulnerable/vulnerable-1).

## Remediation

To address this issue, restrict the function to signed extrinsics by requiring a valid signature from the caller. This ensures accountability and prevents unauthorized access. Here's the corrected implementation:

```rust
#[pallet::call(weight(<T as Config>::WeightInfo))]
impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    pub fn safe_call(origin: OriginFor<T>) -> DispatchResult {
        let _ = ensure_signed(origin)?;

        Ok(())
    }
}
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/develop/test-cases/substrate-pallets/unsigned-extrinsic/remediated/remediated-1).
