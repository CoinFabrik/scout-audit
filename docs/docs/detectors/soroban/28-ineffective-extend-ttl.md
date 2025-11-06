# Ineffective extend_ttl

## Description

- Category: `Best practices`
- Severity: `Low`
- Detector: [`ineffective-extend-ttl`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/ineffective-extend-ttl)
- Test Cases: [`ineffective-extend-ttl-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/ineffective-extend-ttl/ineffective-extend-ttl-1)

Soroban exposes `extend_ttl` to prolong the lifetime of storage entries. The call receives a threshold value that gates when the entry should be refreshed and the target TTL to extend the entry to. If the target TTL is the same as or lower than the threshold, the extension becomes wasteful because it triggers on every contract invocation instead of only when needed.

## Why is this bad?

When the target TTL does not exceed the refresh threshold, `extend_ttl` fires on **every** contract invocation, wasting resources. Each call pays to extend the TTL to a value that is at or below the threshold, meaning the next invocation will find the entry below the threshold again and repeat the extension unnecessarily. This defeats the purpose of the threshold parameter, which is designed to limit how frequently extensions occur.

Instead of extending only when the entry is about to expire, the contract pays for TTL extensions on every single call, consuming resources for no practical benefit.

## Issue example

Consider the following excerpt:
```rust
env.storage().temporary().set(&CACHE_KEY, &entry);
let ttl = 100_000;
env.storage().temporary().extend_ttl(&CACHE_KEY, ttl, ttl);
```

Because both parameters use the same value, each invocation extends the TTL to exactly the threshold value. Since the TTL will always be at or below the threshold after any ledger passes, every subsequent invocation triggers another extension, wasting resources on every call.

The vulnerable example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/ineffective-extend-ttl/ineffective-extend-ttl-1/vulnerable-example).

## Remediated example
```rust
env.storage()
    .temporary()
    .extend_ttl(&CACHE_KEY, 50_000, 100_000);
```

By setting the target TTL higher than the refresh threshold, the entry is only refreshed when its remaining lifetime falls below the threshold. This allows the entry to age naturally between refreshes, reducing unnecessary extension calls and saving resources. Alternatively, the contract can avoid calling `extend_ttl` altogether and manage expiration through business logic.

The remediated example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/ineffective-extend-ttl/ineffective-extend-ttl-1/remediated-example).

## How is it detected?

The detector looks for `extend_ttl` calls on any Soroban storage interface. It warns whenever the threshold and target refer to the same binding, or when both resolve to constant integer values and the target does not strictly exceed the threshold (including values imported through `const` items). This ensures the lint fires even when contracts centralize TTL thresholds in shared constants.