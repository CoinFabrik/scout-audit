import argparse
from test_utils import (
    run_tests,
    print_errors,
    run_subprocess,
)

if __name__ == "__main__":
    print(run_subprocess(["cargo", "scout-audit", "--version"], ".")[1])
    print(run_subprocess(["cargo", "scout-audit", "--src-hash"], ".")[1])
    parser = argparse.ArgumentParser(description="Run tests for a specific detector.")
    parser.add_argument(
        "--detector",
        type=str,
        required=True,
        help='The detector to run tests for, e.g., "unsafe-unwrap"',
    )
    args = parser.parse_args()

    errors = run_tests(args.detector)
    print_errors(errors)
    if errors:
        exit(1)
