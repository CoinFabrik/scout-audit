#!/usr/bin/python3
import utils

commands = [
    ("apps/cargo-scout-audit",       "cargo fmt --all --check"),
    ("detectors/ink",                "cargo fmt --check"      ),
    ("detectors/soroban",            "cargo fmt --check"      ),
    ("detectors/substrate-pallets",  "cargo fmt --check"      ),
    ("test-cases/ink",               "cargo fmt --check"      ),
    ("test-cases/soroban",           "cargo fmt --check"      ),
    ("test-cases/substrate-pallets", "cargo fmt --check"      ),
]

if __name__ == "__main__":
    exit(utils.simple_runner("cargo fmt", commands, {
        "fail_fast": False,
        "show_err": True,
    }))
