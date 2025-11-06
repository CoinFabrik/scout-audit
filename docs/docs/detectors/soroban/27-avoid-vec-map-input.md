# Avoid Vec or Map inputs

## Description

- Category: `Best practices`
- Severity: `Medium`
- Detector: [`avoid-vec-map-input`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-vec-map-input)
- Test Cases: [`avoid-vec-map-input-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-vec-map-input/avoid-vec-map-input-1)

The Soroban host serializes `soroban_sdk::Vec` and `soroban_sdk::Map` collections as untyped `Val` values when they cross the contract boundary. Without validating each element, a contract might persist malformed data or values that fail when converted back into the expected types, halting execution or corrupting state.

## Why is it bad?

Accepting unvalidated collections allows an attacker to craft inputs that cannot be converted to the contract's domain-specific types. Persisting those values can later cause panics, failed deserializations, or logic errors when the contract tries to use the stored data, effectively becoming a denial-of-service vector.

## Issue example

Consider the following contract:

```rust
pub fn store_addresses(env: Env, addresses: Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::Addresses, &addresses);
}
```

The contract stores every address that arrives in the `Vec` without validating it. Any invalid entry that was coerced into a raw `Val` will be stored and may later break execution when the list is read and decoded.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-vec-map-input/avoid-vec-map-input-1/vulnerable-example).

## Remediated example

```rust
pub fn add_address(env: Env, address: Address) -> Result<(), ValidationError> {
    let mut stored = load_addresses(&env);
    let len = stored.len();
    let mut i = 0;
    while i < len {
        if stored.get(i).unwrap() == address {
            return Err(ValidationError::DuplicateAddress);
        }
        i += 1;
    }

    stored.push_back(address);
    save_addresses(&env, &stored);
    Ok(())
}
```

Instead of accepting an entire collection, the contract receives one value at a time and validates it before extending the stored list. Other strategies include iterating over the received collection, converting each `Val` into the contract's native type, and rejecting any element that fails validation.

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/avoid-vec-map-input/avoid-vec-map-input-1/remediated-example).

## How is it detected?

The detector looks for contract functions whose parameters include `soroban_sdk::Vec` or `soroban_sdk::Map`, indicating that the function accepts raw host collections without enforcing validation.
