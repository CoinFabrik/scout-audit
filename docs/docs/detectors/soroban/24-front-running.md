# Front running

## Description

- Category: `MEV`
- Severity: `Warning`
- Detector: [`front-running`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/front-running)
- Test Cases: [`front-running-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/front-running/front-running-1)

When transferring tokens, if the amount to be sent is calculated rather than passed as a parameter, it is advisable to include an additional parameter that sets a minimum threshold for the transfer. This helps mitigate the risk of front-running attacks.

## Why is this bad?

Front-running attacks can lead to financial losses for the victim and disrupt the normal functioning of the contract.

## Issue example

Consider the following function:

```rust

 pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        let transfer_amount = get_conversion_price(amount);
        TokenClient::new(&e, &get_token(&e)).transfer(&from, &to, &transfer_amount);
    }

```

In this example, the `transfer` function does not include a parameter indicating a minimum amount to be transferred.

## Remediated example

Consider the following function:

```rust

 pub fn transfer(e: Env, from: Address, to: Address, amount: i128, min_amount: i128) {
        let transfer_amount = get_conversion_price(amount);
        assert!(transfer_amount >= min_amount, "Insufficient amount");
        TokenClient::new(&e, &get_token(&e)).transfer(&from, &to, &transfer_amount);
    }

```

In this example, the `transfer` function includes a parameter indicating a minimum amount to be transferred.

The remediated example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/front-running/front-running-1/remediated-example).
