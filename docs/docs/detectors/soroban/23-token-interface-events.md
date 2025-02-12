# Token interface events

## Description 

- Category: `Best practices`
- Severity: `Medium`
- Detectors: [`token-interface-events`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/token-interface-events)
- Test Cases: [`token-interface-events-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/token-interface-events/token-interface-events-1)


In Rust, the token contracts have a special interface with certain requirements. One of these requirements is related to events; this requirement states that token functions must emit the events in the specified format. If this does not happen, the contract will have potential errors.

## Why is this bad? 

If the token functions do not emit events, the following errors may occur:

* Token standard compliance

* Transparency: Events provide a transparent way to log and broadcast important actions like token transfers, approvals, and minting/burning. This transparency is crucial for users, developers, and external systems to monitor and react to contract activities.

* Interoperability: Many decentralized applications (dApps) rely on events to interact with tokens. Without events, these applications might not be able to function correctly, as they would have no way of knowing when a transfer or other important action has occurred. Also, off-chain systems, like wallets, exchanges, and block explorers, use events to track token activity. If events are not implemented, these systems may encounter errors in providing accurate and real-time information about the token.

* Debugging and Auditing: Events are very helpful for debugging and auditing smart contracts. They are useful because they provide detailed information about what happened in the contract during execution.

## Issue example 

Consider the following `Soroban` contract:

```rust

  fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_balance = Self::balance(env.clone(), from.clone());
        let to_balance = Self::balance(env.clone(), to.clone());
        assert!(from_balance >= amount);
        env.storage()
            .instance()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::Balance(to), &(to_balance + amount));
    }

```

In this example, the `transfer()` function does not emit an event.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/token-interface-events/token-interface-events-1/vulnerable-example).


## Remediated example

```rust
  fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_balance = Self::balance(env.clone(), from.clone());
        let to_balance = Self::balance(env.clone(), to.clone());
        assert!(from_balance >= amount);
        env.storage()
            .instance()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));

        TokenUtils::new(&env).events().transfer(from, to, amount);
    }
```
In this example, the `transfer()` function emits an event.

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/token-interface-events/token-interface-events-1/remediated-example).

## How is it detected?

If the token interface trait is being used, check if all of the token's functions emit events. 



    
