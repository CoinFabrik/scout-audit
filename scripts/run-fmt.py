#!/usr/bin/python3
import utils
import os

commands = [
    ("apps/cargo-scout-audit",       "cargo fmt --all --check"),
    ("detectors/ink",                "cargo fmt --check"      ),
    ("detectors/soroban",            "cargo fmt --check"      ),
    ("detectors/substrate-pallets",  "cargo fmt --check"      ),
    ("test-cases/ink",               "cargo fmt --check"      ),
    ("test-cases/soroban",           "cargo fmt --check"      ),
    ("test-cases/substrate-pallets", "cargo fmt --check"      ),
]

def create_digest():
    digest_path = "apps/cargo-scout-audit/src/digest.rs"
    if not os.path.exists(digest_path):
        with open(digest_path, 'w') as file:
            file.write('\n')
            return

if __name__ == "__main__":
    create_digest()
    exit(utils.simple_runner("cargo fmt", commands, {
        "fail_fast": False,
        "show_err": True,
    }))
