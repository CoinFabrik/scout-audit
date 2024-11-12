#!/usr/bin/python3
import utils

commands = [
    ("apps/cargo-scout-audit",                 "cargo clippy --all-targets --all-features -- -D warnings"),
    ("detectors/ink/detectors",                "cargo clippy --all-features -- -D warnings"),
    ("detectors/soroban/detectors",            "cargo clippy --all-targets --all-features -- -D warnings"),
    ("detectors/substrate-pallets/detectors",  "cargo clippy --all-targets --all-features -- -D warnings"),
    ("detectors/ink/test-cases",               "cargo clippy -- -D warnings -A clippy::new_without_default"),
    ("detectors/soroban/test-cases",           "cargo clippy --all-targets --all-features -- -D warnings"),
    ("detectors/substrate-pallets/test-cases", "cargo clippy --all-targets --all-features -- -D warnings"),
]

if __name__ == "__main__":
    exit(utils.simple_runner("cargo clippy", commands, {
        "fail_fast": True,
        "show_err": True,
    }))
