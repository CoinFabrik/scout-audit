# Unsafe map get

## Description
- Vulnerability Category: `Validations and error handling`
- Severity: `Medium`
- Detectors: [`unsafe-map-get`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-map-get)
- Test Cases: [`unsafe-map-get-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1)

The use of certain methods (`get`, `get_unchecked`, `try_get_unchecked`) on a `Map` object in the Soroban environment without appropriate error handling can lead to potential runtime panics. This vulnerability stems from accessing the map's values with keys that may not exist, without using safer alternatives that check the existence of the key. Such practices can compromise the robustness of the smart contract by causing it to terminate unexpectedly, which may lead to denial of service or inconsistent state within the contract.

## Exploit Scenario

Consider the following `Soroban` contract:

```rust
    #[contractimpl]
    impl UnsafeMapGet {
        pub fn get_from_map(env: Env) -> Option<i32> {
            let map: Map<Val, Val> = map![&env, (1i32.into_val(&env), 2i64.into_val(&env))];
            let map: Val = map.into();
            let map: Map<i32, i32> = map.try_into_val(&env).unwrap();
            map.get(1)
        }
    }
```
This function retrieves values from a map using `map.get()` without checking if the key actually exists in the map. If the key doesn't exist after the conversion, `get` will panic, causing the entire contract to fail.

The vulnerable code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1/vulnerable-example).

## Remediation

Both remediated functions presented below ensure the contract doesn't panic due to missing keys. The remediated contract avoid the `unsafe map get` vulnerability by using `try_get` for safer access and ensuring the map keys and values have compatible types throughout the process.

```rust
    #[contractimpl]
    impl UnsafeMapGet {
        pub fn get_map_with_different_values(env: Env, key: i32) -> Result<Option<i32>, Error> {
            let map: Map<Val, Val> = map![
                &env,
                (1i32.into_val(&env), 2i32.into_val(&env)),
                (3i32.into_val(&env), 4i64.into_val(&env)),
            ];
            let map: Val = map.into();
            let map: Map<i32, i32> = map.try_into_val(&env).unwrap();
            map.try_get(key).map_err(Error::from)
        }

        pub fn get_map_with_different_keys(env: Env, key: i32) -> Result<Option<i32>, Error> {
            let map: Map<Val, Val> = map![
                &env,
                (1i32.into_val(&env), 2i32.into_val(&env)),
                (3i64.into_val(&env), 4i32.into_val(&env)),
            ];
            let map: Val = map.into();
            let map: Map<i32, i32> = map.try_into_val(&env).unwrap();
            map.try_get(key).map_err(Error::from)
        }
    }
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1/remediated-example).
