import argparse
import os
import subprocess
import time

from utils import print_errors, print_results, is_rust_project

GREEN = "\033[92m"
ENDC = "\033[0m"


def run_udeps(directories):
    errors = []
    for directory in directories:
        if not os.path.isdir(directory):
            errors.append(
                f"Error: The specified path '{directory}' is not a directory or does not exist."
            )
            continue

        print(f"\n{GREEN}Checking unused dependencies in {directory}:{ENDC}")
        for root, _, _ in os.walk(directory):
            if is_rust_project(root):
                start_time = time.time()
                returncode, _, stderr = subprocess.run(
                    ["cargo", "udeps"], cwd=root, capture_output=True, text=True
                )
                print_results(
                    returncode, stderr, "udeps", root, time.time() - start_time
                )
                if returncode != 0:
                    errors.append(root)

    return errors


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Run cargo-udeps for specified directories"
    )
    parser.add_argument(
        "--dir",
        nargs="+",
        required=True,
        help="Specify the directories to run cargo-udeps on. Multiple directories can be specified.",
    )
    args = parser.parse_args()

    errors = run_udeps(args.dir)
    print_errors(errors)
    if errors:
        exit(1)
