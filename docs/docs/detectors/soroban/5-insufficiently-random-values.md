# Insufficiently random values

## Description 

- Category: `Block attributes`
- Severity: `Critical`
- Detector: [`insufficiently-random-values`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/insufficiently-random-values)
- Test Cases: [`insufficiently-random-values-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/insufficiently-random-values/insufficiently-random-values-1) 

Using block attributes like `timestamp` or `sequence` for random number generation in Soroban smart contracts is not recommended due to the predictability of these values. Block attributes are publicly visible and deterministic, making it easy for malicious actors to anticipate their values and manipulate outcomes to their advantage. It's important to use a source that is both unpredictable and external to the blockchain environment, reducing the potential for malicious exploitation.

## Why is this bad? 

Using `ledger().timestamp()` is not recommended because it could be potentially manipulated by validator, which might lead to potential problems. On the other hand, `ledger().sequence()` is publicly available. An attacker could predict the random number to be generated to manipulate the code and perform an attack on the contract.


## Issue example 

Consider the following `Soroban` contract:

```rust
pub fn generate_random_value_timestamp(env: Env, max_val: u64) -> Result<u64, Error> {
        if max_val == 0 {
            Err(Error::MaxValZero)
        } else {
            let val = env.ledger().timestamp() % max_val;
            Ok(val)
        }
    }
    
    pub fn generate_random_value_sequence(env: Env, max_val: u32) -> Result<u32, Error> {
        if max_val == 0 {
            Err(Error::MaxValZero)
        } else {
            let val = env.ledger().sequence() % max_val;
            Ok(val)
        }
    }
    
```

The issue lies in these functions use of blockchain-provided data like block timestamp and sequence number for pseudo-random number generation. This reliance on predictable blockchain data makes the generated values susceptible to manipulation by attackers.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/insufficiently-random-values/insufficiently-random-values-1/vulnerable-example).


## Remediated example

Avoid using block attributes like `timestamp` or `sequence` for randomness generation, and consider using PRNG instead.

```rust

  pub fn generate_random_value(env: Env, max_val: u64) -> Result<u64, Error> {
        if max_val == 0 {
            Err(Error::MaxValZero)
        } else {
            let val = env.prng().gen_range(0..max_val);
            Ok(val)
        }
    }
        
```

The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/insufficiently-random-values/insufficiently-random-values-1/remediated-example).

## How is it detected?

Checks the usage of `ledger().timestamp()` or `ledger().sequence()` for generation of random numbers.


## References

- https://dasp.co/#item-6
- https://blog.sigmaprime.io/solidity-security.html#SP-6
- [SWC-120](https://swcregistry.io/docs/SWC-120)
- [SWC-116](https://swcregistry.io/docs/SWC-116)
- [Ethernaut: Coinflip](https://ethernaut.openzeppelin.com/level/0x4dF32584890A0026e56f7535d0f2C6486753624f)
- [Slither: Weak PRNG](https://github.com/crytic/slither/wiki/Detector-Documentation#weak-PRNG)
- [Slither: Dangerous usage of block.timestamp](https://github.com/crytic/slither/wiki/Detector-Documentation#block-timestamp)

    
