# Unsafe map get

## Description

- Category: `Validations and error handling`
- Severity: `Medium`
- Detectors: [`unsafe-map-get`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-map-get)
- Test Cases: [`unsafe-map-get-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1)

The use of certain methods (`get`, `get_unchecked`, `try_get_unchecked`) on a `Map` object in the Soroban environment without appropriate error handling can lead to potential runtime panics. This issue stems from accessing the map's values with keys that may not exist, without using safer alternatives that check the existence of the key. 

## Why is it bad?

These methods can lead to panics if the key does not exist in the map. Using these methods without proper checks increases the risk of runtime errors that can disrupt the execution of the smart contract and potentially lead to unexpected behavior or denial of service.

## Issue example

Consider the following `Soroban` contract:

```rust
pub fn get_from_map(env: Env) -> Option<i32> {
    let map: Map<Val, Val> = map![&env, (1i32.into_val(&env), 2i64.into_val(&env))];
    let map: Val = map.into();
    let map: Map<i32, i32> = map.try_into_val(&env).unwrap();
    map.get(1)
}
```
The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1/vulnerable-example).

## Remediated example

```rust
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
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unsafe-map-get/unsafe-map-get-1/remediated-example).

## How is it detected?

Checks for array pushes without access control.
