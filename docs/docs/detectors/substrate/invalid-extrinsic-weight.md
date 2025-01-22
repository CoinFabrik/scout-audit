# Invalid extrinsic weight

## Description

- Category: `Known Bugs`
- Severity: `Enhancement`
- Detectors: [`invalid-extrinsic-weight`](https://github.com/CoinFabrik/scout-audit/tree/main/detectors/substrate-pallets/invalid-extrinsic-weight)
- Test Cases: [`invalid-extrinsic-weight-1`](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/invalid-extrinsic-weight)

The weight attribute is using a weight calculation function that doesn't match the extrinsic name. Each extrinsic must have its own dedicated weight calculation to accurately reflect its resource consumption. Reusing weight calculations from other functions can lead to incorrect resource estimation and potential issues in production.

## Issue scenario

Consider the following functions:

```rust
#[pallet::call(weight(<T as Config>::WeightInfo))]
impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    pub fn dummy_call(_origin: OriginFor<T>) -> DispatchResult {
        Ok(())
    }

    #[pallet::call_index(1)]
    pub fn another_dummy_call(_origin: OriginFor<T>) -> DispatchResult {
        Ok(())
    }
}
```

In the provided implementation, `another_dummy_call` reuses the weight calculation function intended for another context. By not having a unique weight definition, this extrinsic introduces vulnerabilities into the system. Specifically, reusing weight functions may result in underestimating or overestimating resource consumption, leaving the network susceptible to Denial-of-Service (DoS) attacks.

The vulnerable code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/invalid-extrinsic-weight/vulnerable/vulnerable-1).

## Remediation

To prevent this issue, assign a unique and dedicated weight calculation function to each extrinsic as in the following example.

```rust
#[pallet::call(weight(<T as Config>::WeightInfo))]
impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    pub fn dummy_call(_origin: OriginFor<T>) -> DispatchResult {
        Ok(())
    }

    #[pallet::call_index(1)]
    #[pallet::weight(<T as Config>::WeightInfo::dummy_call())]
    pub fn another_dummy_call(_origin: OriginFor<T>) -> DispatchResult {
        Ok(())
    }
}
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/substrate-pallets/invalid-extrinsic-weight/vulnerable/vulnerable-1).
