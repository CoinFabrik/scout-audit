pub const SOROBAN_DETECTORS: [&str; 13] = [
    "avoid_core_mem_forget",
    "avoid_panic_error",
    "avoid_unsafe_block",
    "divide_before_multiply",
    "dos_unbounded_operation",
    "insufficiently_random_values",
    "overflow_check",
    "set_contract_storage",
    "soroban_version",
    "unprotected_update_current_contract_wasm",
    "unsafe_expect",
    "unsafe_unwrap",
    "unused_return_enum",
];

pub const INK_DETECTORS: [&str; 24] = [
    "assert_violation",
    "avoid_std_core_mem_forget",
    "avoid_format_string",
    "delegate_call",
    "divide_before_multiply",
    "dos_unbounded_operation",
    "unexpected_revert_warn",
    "check_ink_version",
    "insufficiently_random_values",
    "integer_overflow_underflow",
    "iterator_over_indexing",
    "lazy_delegate",
    "panic_error",
    "reentrancy_1",
    "reentrancy_2",
    "unprotected_set_code_hash",
    "set_storage_warn",
    "unprotected_mapping_operation",
    "unprotected_self_destruct",
    "unrestricted_transfer_from",
    "unsafe_expect",
    "unsafe_unwrap",
    "unused_return_enum",
    "zero_or_test_address",
];
use crate::output::report::RawVulnerability;
use scout_audit_internal::{ink_lint_message::*, soroban_lint_message::*};

pub const SOROBAN_AVOID_CORE_MEM_FORGET: RawVulnerability = RawVulnerability {
    id: "avoid_core_mem_forget",
    name: "Avoid core::mem::forget usage",
    short_message: SOROBAN_AVOID_CORE_MEM_FORGET_LINT_MESSAGE,
    long_message: "The core::mem::forget function is used to forget about a value without running its destructor. This could lead to memory leaks and logic errors.",
    severity: "Enhancement",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-core-mem-forget",
    vulnerability_class: "Best practices",
};

pub const SOROBAN_AVOID_PANIC_ERROR: RawVulnerability = RawVulnerability {
    id: "avoid_panic_error",
    name: "Avoid panic! macro",
    short_message: SOROBAN_AVOID_PANIC_ERROR_LINT_MESSAGE,
    long_message: "The use of the panic! macro to stop execution when a condition is not met is useful for testing and prototyping but should be avoided in production code. Using Result as the return type for functions that can fail is the idiomatic way to handle errors in Rust.    ",
    severity: "Enhancement",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-panic-error",
    vulnerability_class: "Validations and error handling",
};

pub const SOROBAN_AVOID_UNSAFE_BLOCK: RawVulnerability = RawVulnerability {
    id: "avoid_unsafe_block",
    name: "Avoid unsafe block",
    short_message: SOROBAN_AVOID_UNSAFE_BLOCK_LINT_MESSAGE,
    long_message: "The unsafe block is used to bypass Rust's safety checks. It is recommended to avoid using unsafe blocks as much as possible, and to use them only when necessary.    ",
    severity: "Enhancement",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/avoid-unsafe-block",
    vulnerability_class: "Best practices",
};

pub const SOROBAN_DIVIDE_BEFORE_MULTIPLY: RawVulnerability = RawVulnerability {
    id: "divide_before_multiply",
    name: "Divide Before Multiply",
    short_message: SOROBAN_DIVIDE_BEFORE_MULTIPLY_LINT_MESSAGE,
    long_message: "Performing a division operation before a multiplication can lead to a loss of precision. This issue becomes significant in programs like smart contracts where numerical precision is crucial.",
    severity: "Medium",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/divide-before-multiply",
    vulnerability_class: "Arithmetic",
};

pub const SOROBAN_DOS_UNBOUNDED_OPERATION: RawVulnerability = RawVulnerability {
    id: "dos_unbounded_operation",
    name: "Denial of Service: Unbounded Operation",
    short_message: SOROBAN_DOS_UNBOUNDED_OPERATION_LINT_MESSAGE,
    long_message: "In order to prevent a single transaction from consuming all the gas in a block, unbounded operations must be avoided. This includes loops that do not have a bounded number of iterations, and recursive calls.    ",
    severity: "Medium",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/dos-unbounded-operation",
    vulnerability_class: "Denial of Service",
};

pub const SOROBAN_INSUFFICIENTLY_RANDOM_VALUES: RawVulnerability = RawVulnerability {
    id: "insufficiently_random_values",
    name: "Insufficiently Random Values",
    short_message: SOROBAN_INSUFFICIENTLY_RANDOM_VALUES_LINT_MESSAGE,
    long_message: "Use env.prng() to generate random numbers, and remember that all random numbers are under the control of validators.",
    severity: "Critical",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/insufficiently-random-values",
    vulnerability_class: "Block attributes",
};

pub const SOROBAN_OVERFLOW_CHECK: RawVulnerability = RawVulnerability {
    id: "overflow_check",
    name: "Overflow Check",
    short_message: SOROBAN_OVERFLOW_CHECK_LINT_MESSAGE,
    long_message: "An overflow/underflow is typically caught and generates an error. When it is not caught, the operation will result in an inexact result which could lead to serious problems.",
    severity: "Critical",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/overflow-check",
    vulnerability_class: "Arithmetic",
};

pub const SOROBAN_SET_CONTRACT_STORAGE: RawVulnerability = RawVulnerability {
    id: "set_contract_storage",
    name: "Set Contract Storage",
    short_message: SOROBAN_SET_CONTRACT_STORAGE_LINT_MESSAGE,
    long_message: "In soroban, the storage of a contract can be modified by an arbitrary caller. When a smart contract uses this function, the contract needs to check if the caller should be able to alter this storage. If this does not happen, an arbitary caller may modify balances and other relevant contract storage.",
    severity: "Critical",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/set-contract-storage",
    vulnerability_class: "Authorization",
};

pub const SOROBAN_SOROBAN_VERSION: RawVulnerability = RawVulnerability {
    id: "soroban_version",
    name: "Check Soroban version",
    short_message: SOROBAN_VERSION_LINT_MESSAGE,
    long_message: "Using a older version of Soroban can be dangerous, as it may have bugs or security issues. Use the latest version available.",
    severity: "Enhancement",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/soroban-version",
    vulnerability_class: "Best practices",
};

pub const SOROBAN_UNPROTECTED_UPDATE_CURRENT_CONTRACT_WASM: RawVulnerability = RawVulnerability {
    id: "unprotected_update_current_contract_wasm",
    name: "Unprotected Update Current Contract Wasm",
    short_message: SOROBAN_UNPROTECTED_UPDATE_CURRENT_CONTRACT_LINT_MESSAGE,
    long_message: "If users are allowed to call update_current_contract_wasm, they can intentionally modify the contract behaviour, leading to the loss of all associated data/tokens and functionalities given by this contract or by others that depend on it. To prevent this, the function should be restricted to administrators or authorized users only.    ",
    severity: "Critical",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unprotected-update-current-contract-wasm",
    vulnerability_class: "Authorization",
};

pub const SOROBAN_UNSAFE_EXPECT: RawVulnerability = RawVulnerability {
    id: "unsafe_expect",
    name: "Unsafe Expect",
    short_message: SOROBAN_UNSAFE_EXPECT_LINT_MESSAGE,
    long_message: "In Rust, the expect method is commonly used for error handling. It retrieves the value from a Result or Option and panics with a specified error message if an error occurs. However, using expect can lead to unexpected program crashes.    ",
    severity: "Medium",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-expect",
    vulnerability_class: "Validations and error handling",
};

pub const SOROBAN_UNSAFE_UNWRAP: RawVulnerability = RawVulnerability {
    id: "unsafe_unwrap",
    name: "Unsafe Unwrap",
    short_message: SOROBAN_UNSAFE_UNWRAP_LINT_MESSAGE,
    long_message: "This vulnerability class pertains to the inappropriate usage of the unwrap method in Rust, which is commonly employed for error handling. The unwrap method retrieves the inner value of an Option or Result, but if an error or None occurs, it triggers a panic and crashes the program.    ",
    severity: "Medium",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unsafe-unwrap",
    vulnerability_class: "Validations and error handling",
};

pub const SOROBAN_UNUSED_RETURN_ENUM: RawVulnerability = RawVulnerability {
    id: "unused_return_enum",
    name: "Unused Return Enum",
    short_message: SOROBAN_UNUSED_RETURN_ENUM_LINT_MESSAGE,
    long_message: "Soroban functions can return a Result enum with a custom error type. This is useful for the caller to know what went wrong when the message fails. The definition of the Result type enum consists of two variants: Ok and Err. If any of the variants is not used, the code could be simplified or it could imply a bug.    ",
    severity: "Minor",
    help: "https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/unused-return-enum",
    vulnerability_class: "Validations and error handling",
};

pub const INK_ASSERT_VIOLATION: RawVulnerability = RawVulnerability {
    id: "assert_violation",
    name: "Assert Violation",
    short_message: INK_ASSERT_VIOLATION_LINT_MESSAGE,
    long_message: "The assert! macro is used in Rust to ensure that a certain condition holds true at a certain point in your code. If the condition does not hold, then the assert! macro will cause the program to panic. This is a problem, as seen in panic-error",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/assert-violation",
    vulnerability_class: "Validations and error handling",
};

pub const INK_AVOID_STD_CORE_MEM_FORGET: RawVulnerability = RawVulnerability {
    id: "avoid_std_core_mem_forget",
    name: "Avoid std::mem::forget usage",
    short_message: INK_AVOID_CORE_MEM_FORGET_LINT_MESSAGE,
    long_message: "The core::mem::forget function is used to forget about a value without running its destructor. This could lead to memory leaks and logic errors.",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/avoid-core-mem-forget",
    vulnerability_class: "Best practices",
};

pub const INK_AVOID_FORMAT_STRING: RawVulnerability = RawVulnerability {
    id: "avoid_format_string",
    name: "Avoid format! macro",
    short_message: INK_AVOID_FORMAT_STRING_LINT_MESSAGE,
    long_message: "The format! macro is used to create a String from a given set of arguments. This macro is not recommended, it is better to use a custom error type enum.    ",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/avoid-format-string",
    vulnerability_class: " Validations and error handling",
};

pub const INK_DELEGATE_CALL: RawVulnerability = RawVulnerability {
    id: "delegate_call",
    name: "Unsafe Delegate Call",
    short_message: INK_DELEGATE_CALL_LINT_MESSAGE,
    long_message: "It is important to validate and restrict delegate calls to trusted contracts, implement proper access control mechanisms, and carefully review external contracts to prevent unauthorized modifications, unexpected behavior, and potential exploits.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/delegate-call",
    vulnerability_class: "Authorization ",
};

pub const INK_DIVIDE_BEFORE_MULTIPLY: RawVulnerability = RawVulnerability {
    id: "divide_before_multiply",
    name: "Divide Before Multiply",
    short_message: INK_DIVIDE_BEFORE_MULTIPLY_LINT_MESSAGE,
    long_message: "Performing a division operation before a multiplication can lead to a loss of precision. This issue becomes significant in programs like smart contracts where numerical precision is crucial.",
    severity: "Medium",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/divide-before-multiply",
    vulnerability_class: "Arithmetic",
};

pub const INK_DOS_UNBOUNDED_OPERATION: RawVulnerability = RawVulnerability {
    id: "dos_unbounded_operation",
    name: "Denial of Service: Unbounded Operation",
    short_message: INK_DOS_UNBOUNDED_OPERATION_LINT_MESSAGE,
    long_message: "In order to prevent a single transaction from consuming all the gas in a block, unbounded operations must be avoided. This includes loops that do not have a bounded number of iterations, and recursive calls.    ",
    severity: "Medium",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/dos-unbounded-operation",
    vulnerability_class: "Denial of Service",
};

pub const INK_UNEXPECTED_REVERT_WARN: RawVulnerability = RawVulnerability {
    id: "unexpected_revert_warn",
    name: "Unexpected Revert Inserting to Storage",
    short_message: INK_DOS_UNEXPECTED_REVERT_WITH_VECTOR_LINT_MESSAGE,
    long_message: " It occurs by preventing transactions by other users from being successfully executed forcing the blockchain state to revert to its original state.",
    severity: "Medium",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/dos-unexpected-revert-with-vector",
    vulnerability_class: "Denial of Service",
};

pub const INK_CHECK_INK_VERSION: RawVulnerability = RawVulnerability {
    id: "check_ink_version",
    name: "Check Ink! version",
    short_message: INK_INK_VERSION_LINT_MESSAGE,
    long_message: "Using a older version of ink! can be dangerous, as it may have bugs or security issues. Use the latest version available.",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/ink-version",
    vulnerability_class: "Best practices",
};

pub const INK_INSUFFICIENTLY_RANDOM_VALUES: RawVulnerability = RawVulnerability {
    id: "insufficiently_random_values",
    name: "Insufficiently Random Values",
    short_message: INK_INSUFFICIENTLY_RANDOM_VALUES_LINT_MESSAGE,
    long_message: "Using block attributes like block_timestamp or block_number for random number generation in ink! Substrate smart contracts is not recommended due to the predictability of these values. Block attributes are publicly visible and deterministic, making it easy for malicious actors to anticipate their values and manipulate outcomes to their advantage.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/insufficiently-random-values",
    vulnerability_class: "Block attributes",
};

pub const INK_INTEGER_OVERFLOW_UNDERFLOW: RawVulnerability = RawVulnerability {
    id: "integer_overflow_underflow",
    name: "Integer Overflow/Underflow",
    short_message: INK_INTEGER_OVERFLOW_OR_UNDERFLOW_LINT_MESSAGE,
    long_message: "An overflow/underflow is typically caught and generates an error. When it is not caught, the operation will result in an inexact result which could lead to serious problems.\n In Ink! 5.0.0, using raw math operations will result in `cargo contract build` failing with an error message.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/integer-overflow-or-underflow",
    vulnerability_class: "Arithmetic",
};

pub const INK_ITERATOR_OVER_INDEXING: RawVulnerability = RawVulnerability {
    id: "iterator_over_indexing",
    name: "Iterator Over Indexing",
    short_message: INK_ITERATORS_OVER_INDEXING_LINT_MESSAGE,
    long_message: "The use of iterators over indexing is a best practice that should be followed in Rust. This is because accessing a vector by index is slower than using an iterator. Also, if the index is out of bounds, it will panic.    ",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/iterators-over-indexing",
    vulnerability_class: "Best practices",
};

pub const INK_LAZY_DELEGATE: RawVulnerability = RawVulnerability {
    id: "lazy_delegate",
    name: "Lazy Delegate",
    short_message: INK_LAZY_DELEGATE_LINT_MESSAGE,
    long_message: "A bug in ink! causes delegated calls to not modify the caller's storage unless Lazy with ManualKey or Mapping is used.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/lazy-delegate",
    vulnerability_class: "Known Bugs",
};

pub const INK_PANIC_ERROR: RawVulnerability = RawVulnerability {
    id: "panic_error",
    name: "Panic Error",
    short_message: INK_PANIC_ERROR_LINT_MESSAGE,
    long_message: "The use of the panic! macro to stop execution when a condition is not met is useful for testing and prototyping but should be avoided in production code. Using Result as the return type for functions that can fail is the idiomatic way to handle errors in Rust.    ",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/panic-error",
    vulnerability_class: "Validations and error handling",
};

pub const INK_REENTRANCY: RawVulnerability = RawVulnerability {
    id: "reentrancy",
    name: "Reentrancy",
    short_message: INK_REENTRANCY_LINT_MESSAGE,
    long_message: "An ink! smart contract can interact with other smart contracts. These operations imply (external) calls where control flow is passed to the called contract until the execution of the called code is over, then the control is delivered back to the caller. A reentrancy vulnerability may happen when a user calls a function, this function calls a malicious contract which again calls this same function, and this 'reentrancy' has unexpected reprecussions to the contract.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/reentrancy",
    vulnerability_class: "Reentrancy",
};

pub const INK_UNPROTECTED_SET_CODE_HASH: RawVulnerability = RawVulnerability {
    id: "unprotected_set_code_hash",
    name: "Unprotected Set Code Hash",
    short_message: INK_SET_CODE_HASH_LINT_MESSAGE,
    long_message: "If users are allowed to call set_code_hash, they can intentionally modify the contract behaviour, leading to the loss of all associated data/tokens and functionalities given by this contract or by others that depend on it. To prevent this, the function should be restricted to administrators or authorized users only.    ",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/unprotected-set-code-hash",
    vulnerability_class: "Authorization",
};

pub const INK_SET_STORAGE_WARN: RawVulnerability = RawVulnerability {
    id: "set_storage_warn",
    name: "Set Contract Storage",
    short_message: INK_SET_CONTRACT_STORAGE_LINT_MESSAGE,
    long_message: "In ink! the function set_contract_storage(key: &K, value: &V) can be used to modify the contract storage under a given key. When a smart contract uses this function, the contract needs to check if the caller should be able to alter this storage. If this does not happen, an arbitary caller may modify balances and other relevant contract storage.    ",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/set-contract-storage",
    vulnerability_class: "Authorization",
};

pub const INK_UNPROTECTED_MAPPING_OPERATION: RawVulnerability = RawVulnerability {
    id: "unprotected_mapping_operation",
    name: "Unprotected Mapping Operation",
    short_message: INK_UNPROTECTED_MAPPING_OPERATION_LINT_MESSAGE,
    long_message: "Modifying mappings with an arbitrary key given by the user could lead to unintented modifications of critical data, modifying data belonging to other users, causing denial of service, unathorized access, and other potential issues.    ",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/unprotected-mapping-operation",
    vulnerability_class: "Validations and error handling",
};

pub const INK_UNPROTECTED_SELF_DESTRUCT: RawVulnerability = RawVulnerability {
    id: "unprotected_self_destruct",
    name: "Unprotected Self Destruct",
    short_message: INK_UNPROTECTED_SELF_DESTRUCT_LINT_MESSAGE,
    long_message: "If users are allowed to call terminate_contract, they can intentionally or accidentally destroy the contract, leading to the loss of all associated data and functionalities given by this contract or by others that depend on it. To prevent this, the function should be restricted to administrators or authorized users only.    ",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/unprotected-self-destruct",
    vulnerability_class: "Authorization",
};

pub const INK_UNRESTRICTED_TRANSFER_FROM: RawVulnerability = RawVulnerability {
    id: "unrestricted_transfer_from",
    name: "Unrestricted Transfer From",
    short_message: INK_UNRESTRICTED_TRANSFER_FROM_LINT_MESSAGE,
    long_message: "In an ink! Substrate smart contract, allowing unrestricted transfer_from operations poses a significant vulnerability. When from arguments for that function is provided directly by the user, this might enable the withdrawal of funds from any actor with token approval on the contract. This could result in unauthorized transfers and loss of funds. To mitigate this vulnerability, instead of allowing an arbitrary from address, the from address should be restricted, ideally to the address of the caller (self.env().caller()), ensuring that the sender can initiate a transfer only with their own tokens.    ",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/unrestricted-transfer-from",
    vulnerability_class: "Validations and error handling",
};

pub const INK_UNSAFE_EXPECT: RawVulnerability = RawVulnerability {
    id: "unsafe_expect",
    name: "Unsafe Expect",
    short_message: INK_UNSAFE_EXPECT_LINT_MESSAGE,
    long_message: "In Rust, the expect method is commonly used for error handling. It retrieves the value from a Result or Option and panics with a specified error message if an error occurs. However, using expect can lead to unexpected program crashes.    ",
    severity: "Medium",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/unsafe-expect",
    vulnerability_class: "Validations and error handling",
};

pub const INK_UNSAFE_UNWRAP: RawVulnerability = RawVulnerability {
    id: "unsafe_unwrap",
    name: "Unsafe Unwrap",
    short_message: INK_UNSAFE_UNWRAP_LINT_MESSAGE,
    long_message: "This vulnerability class pertains to the inappropriate usage of the unwrap method in Rust, which is commonly employed for error handling. The unwrap method retrieves the inner value of an Option or Result, but if an error or None occurs, it triggers a panic and crashes the program.    ",
    severity: "Medium",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/unsafe-unwrap",
    vulnerability_class: "Validations and error handling",
};

pub const INK_UNUSED_RETURN_ENUM: RawVulnerability = RawVulnerability {
    id: "unused_return_enum",
    name: "Unused Return Enum",
    short_message: INK_UNUSED_RETURN_ENUM_LINT_MESSAGE,
    long_message: "Ink! messages can return a Result enum with a custom error type. This is useful for the caller to know what went wrong when the message fails. The definition of the Result type enum consists of two variants: Ok and Err. If any of the variants is not used, the code could be simplified or it could imply a bug.    ",
    severity: "Minor",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/unused-return-enum",
    vulnerability_class: "Validations and error handling",
};

pub const INK_ZERO_OR_TEST_ADDRESS: RawVulnerability = RawVulnerability {
    id: "zero_or_test_address",
    name: "Zero or Test Address",
    short_message: INK_ZERO_OR_TEST_ADDRESS_LINT_MESSAGE,
    long_message: "The assignment of the zero address to a variable in a smart contract represents a critical vulnerability because it can lead to loss of control over the contract. This stems from the fact that the zero address does not have an associated private key, which means it's impossible to claim ownership, rendering any contract assets or functions permanently inaccessible.    ",
    severity: "Medium",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/zero-or-test-address",
    vulnerability_class: "Validations and error handling",
};
