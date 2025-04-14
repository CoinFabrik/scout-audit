#!/usr/bin/python3
import glob
import os
import utils

BASE_DIRS = [
    ("apps/cargo-scout-audit",       "cargo clippy --all-targets --all-features -- -D warnings"  ),
    ("test-cases/ink",               "cargo clippy -- -D warnings -A clippy::new_without_default"),
    # Temporary fix for soroban issue with `used_linker`
    ("test-cases/soroban",           "cargo +nightly-2024-07-11 clippy --all-targets --all-features -- -D warnings"  ),
    ("test-cases/substrate-pallets", "cargo clippy --all-targets --all-features -- -D warnings"  ),
]

# Subdirectories to check in each nightly version
NIGHTLY_SUBDIRS = ["ink", "soroban", "substrate-pallets"]


def get_nightly_commands():
    """Dynamically generate commands for all nightly directories."""
    nightly_commands = []
    # Find all nightly directories
    nightly_dirs = glob.glob("nightly/20[0-9][0-9]-*-*")

    for nightly_dir in nightly_dirs:
        for subdir in NIGHTLY_SUBDIRS:
            detector_path = f"{nightly_dir}/detectors/{subdir}"
            if os.path.exists(detector_path):
                nightly_commands.append((detector_path, "cargo clippy --all-targets --all-features -- -D warnings"))

    return nightly_commands

if __name__ == "__main__":
    commands = BASE_DIRS + get_nightly_commands()
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
