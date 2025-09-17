#!/usr/bin/python3
import glob
import os
import utils

BASE_DIRS = [
    ("apps/cargo-scout-audit",       "cargo clippy --all-targets --all-features -- -D warnings"  ),
    ("test-cases/ink",               "cargo clippy -- -D warnings -A clippy::new_without_default"),
    # Temporary fix for soroban issue with `used_linker`
    ("test-cases/soroban",           "cargo clippy --all-targets --all-features -- -D warnings"  ),
    ("test-cases/substrate-pallets", "cargo clippy --all-targets --all-features -- -D warnings"  ),
]



if __name__ == "__main__":
    commands = BASE_DIRS + utils.get_nightly_commands("cargo clippy --all-targets --all-features -- -D warnings")
    exit(
        utils.simple_runner(
            "cargo clippy",
            commands,
            {
                "fail_fast": True,
                "show_err": True,
            },
        )
    )
