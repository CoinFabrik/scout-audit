#!/usr/bin/python3
import utils

commands = [
    ("apps/cargo-scout-audit",       "cargo clippy --all-targets --all-features -- -D warnings"  ),
    ("detectors/ink",                "cargo clippy --all-features -- -D warnings"                ),
    ("detectors/soroban",            "cargo clippy --all-targets --all-features -- -D warnings"  ),
    ("detectors/substrate-pallets",  "cargo clippy --all-targets --all-features -- -D warnings"  ),
    ("test-cases/ink",               "cargo clippy -- -D warnings -A clippy::new_without_default"),
    ("test-cases/soroban",           "cargo clippy --all-targets --all-features -- -D warnings"  ),
    ("test-cases/substrate-pallets", "cargo clippy --all-targets --all-features -- -D warnings"  ),
]

if __name__ == "__main__":
    exit(utils.simple_runner("cargo clippy", commands, {
        "fail_fast": True,
        "show_err": True,
    }))
