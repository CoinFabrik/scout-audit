# Infinite recursion

## Description

- Category: `Denial of service`
- Severity: `Medium`
- Detector: [`infinite-recursion-over-storage`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/infinite-recursion-over-storage)
- Test Cases: [`infinite-recursion-over-storage-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/infinite-recursion-over-storage/infinite-recursion-over-storage-1)

This detector looks for recursive call cycles reachable from a Soroban entrypoint. In Soroban, recursive execution already risks exhausting gas or overflowing the call stack, even when the recursive flow does not touch storage.

## Why is this bad?

Recursive cycles without an exit condition can keep calling themselves until execution runs out of gas or stack budget. That creates a denial-of-service pattern and can make otherwise simple entrypoints unexpectedly revert.

This detector is intentionally conservative in one dimension: it only considers recursion reachable from Soroban entrypoints. But once a reachable recursive cycle exists, it warns regardless of whether storage is touched along the way.

## Issue example

```rust
pub fn recurse(env: Env, depth: u32) -> u32 {
    Self::recurse(env, depth + 1)
}
```

The function immediately calls itself again with no terminating condition. The flow can continue until the call stack or gas budget is exhausted.

The vulnerable examples can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/infinite-recursion-over-storage/infinite-recursion-over-storage-1/vulnerable-example).

## Remediated example

```rust
fn write_once(env: Env, depth: u32) -> u32 {
    depth
}
```

By removing the recursive cycle, the function returns normally. More generally, the fix is to break the recursive cycle so the entrypoint cannot keep consuming gas indefinitely.

The remediated examples can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/infinite-recursion-over-storage/infinite-recursion-over-storage-1/remediated-example).

## How is it detected?

The detector builds a local call graph for the contract, starts from Soroban entrypoints, and computes recursive strongly connected components over the reachable functions. A warning is emitted on recursive callsites inside a reachable recursive cycle, including direct self-recursion and longer mutual-recursion chains.
