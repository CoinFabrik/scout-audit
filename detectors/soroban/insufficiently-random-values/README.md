# Insuficciently random values

### What it does
Checks the usage of `ledger().timestamp()` or `ledger().sequence()` for generation of random numbers.

### Why is this bad?
Using `ledger().timestamp()` is not recommended because it could be potentially manipulated by validator. On the other hand, `ledger().sequence()` is publicly available, an attacker could predict the random number to be generated.

### Example

```rust
#[contractimpl]
impl Contract {
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
}
```

Use instead:

```rust
#[contractimpl]
impl Contract {
    pub fn generate_random_value(env: Env, max_val: u64) -> Result<u64, Error> {
        if max_val == 0 {
            Err(Error::MaxValZero)
        } else {
            let val = env.prng().u64_in_range(0..max_val);
            Ok(val)
        }
    }
}
```

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/insufficiently-random-values).
