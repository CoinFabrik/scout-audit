---
sidebar_position: 10
---

# Toggle detections on and off

In addition to enabling and disabling detectors, Scout allows users to toggle individual detections on or off. This feature is useful for disabling detections that are false positives or not relevant to the analyzed codebase.

## Usage

### 1) Import scout-utils package

To use the toggle detections on/off feature, you’ll need to import the `scout-utils` package into your project, adding the following line to your `Cargo.toml`.

```rust
scout-utils = "0.1.0"
```

### 2) Include scout-utils in your Rust file

Include the scout-utils package in the Rust file in which you want to disable detections, adding the following line:

```rust
use scout-audit::scout_allow;
```

### 3) Use scout_allow macro to disable a detection

To disable a detection, you’ll need to use the scout_allow macro, with the name of the detection to disable as an attribute. For example:

```rust
#[scout_allow(unsafe_unwrap)]
```

Place the macro before the block of code in which you want to disable a detection. For example:

```rust
#[scout_allow(unsafe_expect)]
pub fn my_func() {
let x: Option<&str> = None;
x.expect("Something went wrong!");
}
```

The macro supports including more than one attribute to disable multiple detections at once. For example:

```rust
#[scout_allow(unsafe_unwrap, integer_overflow_or_underflow)]
```

## Supported scope

`scout_allow` macro supports disabling detections for the following scopes:

- Functions (entire body)
- Modules
- Structs
- Enums
- Traits
- Impl blocks

## Unnecesary scout_allow macro detector

If Scout Audit detects a scout_allow macro for a block of code in which the disallowed detection is not triggered, it will raise a warning.
