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
                nightly_commands.append((detector_path, "cargo fmt --check"))

    return nightly_commands


def create_digest():
    digest_path = "apps/cargo-scout-audit/src/digest.rs"
    if not os.path.exists(digest_path):
        with open(digest_path, "w") as file:
            file.write("\n")
            return


if __name__ == "__main__":
    commands = BASE_DIRS + get_nightly_commands()
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
