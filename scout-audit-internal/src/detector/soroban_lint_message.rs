pub const SOROBAN_AVOID_CORE_MEM_FORGET_LINT_MESSAGE: &str =
    "Use the `let _ = ...` pattern or `.drop()` method to forget the value";
pub const SOROBAN_INSUFFICIENTLY_RANDOM_VALUES_LINT_MESSAGE: &str =
    "Use env.prng() to generate random numbers, and remember that all random numbers are under the control of validators";

pub const SOROBAN_DIVIDE_BEFORE_MULTIPLY_LINT_MESSAGE: &str =
    "Division before multiplication might result in a loss of precision";
pub const SOROBAN_OVERFLOW_CHECK_LINT_MESSAGE: &str =
    "Use `overflow-checks = true` in Cargo.toml profile";
pub const SOROBAN_SET_CONTRACT_STORAGE_LINT_MESSAGE:&str = "Abitrary users should not have control over keys because it implies writing any value of left mapping, lazy variable, or the main struct of the contract located in position 0 of the storage";
pub const SOROBAN_UNPROTECTED_UPDATE_CURRENT_CONTRACT_LINT_MESSAGE: &str =
    "This update_current_contract_wasm is called without access control";
pub const SOROBAN_UNSAFE_EXPECT_LINT_MESSAGE: &str = "Unsafe usage of `expect`";
pub const SOROBAN_UNSAFE_UNWRAP_LINT_MESSAGE: &str = "Unsafe usage of `unwrap`";
