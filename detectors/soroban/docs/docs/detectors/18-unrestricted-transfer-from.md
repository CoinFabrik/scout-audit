# Unrestricted transfer from

## Description

- Category: `Validations and error handling`
- Severity: `High`
- Detectors: [`unrestricted-transfer-from`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unrestricted-transfer-from)
- Test Cases: [`unrestricted-transfer-from-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unrestricted-transfer-from/unrestricted-transfer-from-1)

Allowing unrestricted `transfer_from` operations poses a significant issue. When `from` arguments for that function is provided directly by the user, this might enable the withdrawal of funds from any actor with token approval on the contract. 

## Why is this bad?

The absence of proper authorization checks for sensitive operations, like `transfer_from`, can lead to the loss of funds or other undesired consequences. For example, if a user, Alice, approves a contract to spend her tokens, and the contract lacks proper authorization checks, another user, Bob, could invoke the contract and potentially transfer Alice's tokens to himself without her explicit consent.

## Issue example

Consider the following `Soroban` function:

```rust
     pub fn deposit(env: Env, from: Address) -> Result<(), UTFError> {
        let mut state: State = Self::get_state(env.clone())?;
        state.buyer.require_auth();
        if state.status != Status::Created {
            return Err(UTFError::StatusMustBeCreated);
        }
        let token_client = token::Client::new(&env, &state.token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &from,
            &env.current_contract_address(),
            &state.amount,
        );
        state.status = Status::Locked;
        env.storage().instance().set(&STATE, &state);
        Ok(())
    }
```

The issue in this `deposit` function arises from the use of `from`, an user-defined parameter as an argument in the `from` field of the `transfer_from` function. Alice can approve a contract to spend their tokens, then Bob can call that contract, use that allowance to send as themselves Alice's tokens.

The code example can be found [`here`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unrestricted-transfer-from/unrestricted-transfer-from-1/vulnerable-example).

## Remediated example

Avoid using user-defined arguments as `from` parameter in `transfer_from`. Instead, use `state.buyer` as shown in the following example.

```rust
     pub fn deposit(env: Env) -> Result<(), UTFError> {
        let mut state: State = Self::get_state(env.clone())?;
        state.buyer.require_auth();
        if state.status != Status::Created {
            return Err(UTFError::StatusMustBeCreated);
        }
        let token_client = token::Client::new(&env, &state.token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &state.buyer,
            &env.current_contract_address(),
            &state.amount,
        );
        state.status = Status::Locked;
        env.storage().instance().set(&STATE, &state);
        Ok(())
    }
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/unrestricted-transfer-from/unrestricted-transfer-from-1/remediated-example).

## How is it detected?

It warns you if a `transfer_from` function is called with a user-defined parameter in the `from` field.

## References

- [Slither: Arbitrary from in transferFrom](https://github.com/crytic/slither/wiki/Detector-Documentation#arbitrary-from-in-transferfrom)

