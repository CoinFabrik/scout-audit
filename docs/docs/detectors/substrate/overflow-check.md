# Overflow-check

## Description

- Category: `Arithmetic`
- Severity: `Critical`
- Detectors: [`overflow-check`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/rust/overflow-check/src/lib.rs)

Checks that `overflow-checks` is enabled in the `[profile.release]` section of the `Cargo.toml`.

### Why is this bad?

Integer overflow will trigger a panic in debug builds or will wrap in
release mode. Division by zero will cause a panic in either mode. In some applications one
wants explicitly checked, wrapping or saturating arithmetic.

### Example

```toml

[package]
edition = "2021"
name = "overflow-check-vulnerable-1"
version = "0.1.0"

[package.metadata.docs.rs]
targets = ["x86_64-unknown-linux-gnu"]

[dependencies]
codec = { workspace = true }
frame-support = { workspace = true }
frame-system = { workspace = true }
log = { workspace = true }
pallet-balances = { workspace = true }
scale-info = { features = ["derive"], workspace = true }
sp-io = { workspace = true }
sp-runtime = { workspace = true }

[dev-dependencies]
sp-core = { workspace = true }

[features]
default = ["std"]
std = [
	"codec/std",
	"frame-support/std",
	"frame-system/std",
	"log/std",
	"pallet-balances/std",
	"scale-info/std",
	"sp-core/std",
	"sp-io/std",
	"sp-runtime/std",
]

```

Use instead:

```toml

[package]
edition = "2021"
name = "overflow-check-remediated-1"
version = "0.1.0"

[package.metadata.docs.rs]
targets = ["x86_64-unknown-linux-gnu"]

[dependencies]
codec = { workspace = true }
frame-support = { workspace = true }
frame-system = { workspace = true }
log = { workspace = true }
pallet-balances = { workspace = true }
scale-info = { features = ["derive"], workspace = true }
sp-io = { workspace = true }
sp-runtime = { workspace = true }

[dev-dependencies]
sp-core = { workspace = true }

[features]
default = ["std"]
std = [
	"codec/std",
	"frame-support/std",
	"frame-system/std",
	"log/std",
	"pallet-balances/std",
	"scale-info/std",
	"sp-core/std",
	"sp-io/std",
	"sp-runtime/std",
]
[profile.release]
codegen-units = 1
debug = 0
debug-assertions = false
lto = true
opt-level = "z"
overflow-checks = false
panic = "abort"
strip = "symbols"
```

### Implementation

The detector's implementation can be found at [this link](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/overflow-check).
