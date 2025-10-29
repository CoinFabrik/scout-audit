#!/usr/bin/python3
import utils
import os
import glob

# Base directories that always need formatting
BASE_DIRS = [
    ("apps/cargo-scout-audit", "cargo fmt --all --check"),
    ("test-cases/ink", "cargo fmt --check"),
    ("test-cases/soroban", "cargo fmt --check"),
    ("test-cases/substrate-pallets", "cargo fmt --check"),
]


def create_digest():
    digest_path = "apps/cargo-scout-audit/crates/cargo-scout-audit/src/digest.rs"
    if not os.path.exists(digest_path):
        with open(digest_path, "w") as file:
            file.write("\n")
            return


if __name__ == "__main__":
    commands = BASE_DIRS + utils.get_nightly_commands("cargo fmt --all --check")
    create_digest()
    exit(
        utils.simple_runner(
            "cargo fmt",
            commands,
            {
                "fail_fast": False,
                "show_err": True,
            },
        )
    )
