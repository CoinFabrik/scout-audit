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
    parser.add_argument(
        "--detectors",
        type=str,
        required=True,
        help="The detectors to run tests for, e.g., 'unsafe-unwrap,unsafe-unwrap-2'",
    )
    args = parser.parse_args()

    if args.detectors:
        detectors = args.detectors.split(",")
    else:
        detectors = [args.detector]
    errors = run_tests(detectors)
    print_errors(errors)
    if errors:
        exit(1)
