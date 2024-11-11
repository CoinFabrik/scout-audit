# Storage change events

## Description 

- Category: `Best practices`
- Severity: `Minor`
- Detectors: [`storage-change-events`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/storage-change-events)
- Test Cases: [`storage-change-events-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/storage-change-events/storage-change-events-1)


In Rust, it is very important to control storage, since it contains a large part of the information of a contract. For this reason, it is common to control storage movements through events, in order to record the changes that occur. If there is no control over these changes, it can lead to potential problems in the contract.

## Why is this bad? 

If there is no control over storage changes, it can lead to security and transparency issues within the contract.

## Issue example 

Consider the following `Soroban` contract:

```rust

  fn set_counter(env: Env, counter: CounterState) {
        env.storage().instance().set(&STATE, &counter);
    }

```

In this example, the `set_counter()` function does not emit an event to notify of a change in the storage.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/storage-change-events/storage-change-events-1/vulnerable-example).


## Remediated example

```rust
    fn set_counter(env: Env, counter: CounterState) {
        env.storage().instance().set(&STATE, &counter);
        env.events()
            .publish((COUNTER, symbol_short!("set")), counter.count);
    }
```
In this example, the `set_counter()` function emits an event to notify of a change in the storage.

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/storage-change-events/storage-change-events-1/remediated-example).

## How is it detected?

Checks if the function emits an event in case a change has occurred in the storage. 



    
