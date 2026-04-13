# Delegated spending should not require owner auth

## Description

- Category: `Authorization`
- Severity: `Minor`
- Detector: [`delegated-spending-from-auth`](https://github.com/CoinFabrik/scout-audit/tree/main/nightly/2025-08-07/detectors/soroban/delegated-spending-from-auth)
- Test Cases: [`delegated-spending-from-auth`](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/delegated-spending-from-auth)

Delegated token operations such as `transfer_from` and `burn_from` are meant to let an approved spender act on behalf of the token owner. If those delegated flows still require `from.require_auth()`, the allowance mechanism stops being useful because both accounts must sign.

## Why is this bad?

Allowance-based spending exists so the owner can approve a spender once and let that spender execute later transfers or burns without another owner signature. Requiring both signatures breaks that contract behavior and can make integrations or protocol flows unusable.

## Issue example

```rust
pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
    spender.require_auth();
    Self::decrease_allowance(env, from, spender, amount);
    let _ = to;
}

pub fn decrease_allowance(_env: Env, from: Address, _spender: Address, _amount: i128) {
    from.require_auth();
}
```

The code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/delegated-spending-from-auth/vulnerable/vulnerable-1).

## Remediated example

```rust
pub fn transfer_from(_env: Env, spender: Address, from: Address, to: Address, amount: i128) {
    spender.require_auth();
    let _ = (from, to, amount);
}
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-audit/tree/main/test-cases/soroban/delegated-spending-from-auth/remediated/remediated-4).

## How is it detected?

The detector looks for canonical Soroban token-interface `transfer_from` and `burn_from` entrypoints. It warns when those delegated functions require `spender` authorization, but also require authorization from the `from` account either in the function itself or through a reachable local helper call. The current analysis is reachability-based and is not path-sensitive across branches.
