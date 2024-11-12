#!/usr/bin/python3
import utils

commands = [
    ("apps/cargo-scout-audit",                 "cargo fmt --all --check"),
    ("detectors/ink/detectors",                "cargo fmt --check -v"),
    ("detectors/soroban/detectors",            "cargo fmt --check"),
    ("detectors/substrate-pallets/detectors",  "cargo fmt --check"),
    ("detectors/ink/test-cases",               "cargo fmt --check -v"),
    ("detectors/soroban/test-cases",           "cargo fmt --check"),
    ("detectors/substrate-pallets/test-cases", "cargo fmt --check"),
]

if __name__ == "__main__":
    exit(utils.simple_runner("cargo fmt", commands, {
        "fail_fast": False,
        "show_err": False,
    }))
