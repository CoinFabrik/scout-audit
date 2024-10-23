import os
import argparse
import time

from utils import (
    parse_json_from_string,
    print_errors,
    print_results,
    run_subprocess,
    is_rust_project,
)

RED = "\033[91m"
GREEN = "\033[92m"
ENDC = "\033[0m"


def run_tests(detector):
    errors = []
    directory = os.path.join("test-cases", detector)
    print(f"\n{GREEN}Performing tests in {directory}:{ENDC}")
    if not os.path.exists(directory):
        print(f"{RED}The specified directory does not exist.{ENDC}")
        return errors

    for root, _, _ in os.walk(directory):
        if is_rust_project(root):
            if run_unit_tests(root):
                errors.append(root)
            if run_integration_tests(detector, root):
                errors.append(root)
    return errors


def run_unit_tests(root):
    start_time = time.time()
    returncode, stdout, _ = run_subprocess(["cargo", "test", "--all-features"], root)
    print_results(
        returncode,
        stdout,
        "unit-test",
        root,
        time.time() - start_time,
    )
    return returncode != 0


def run_integration_tests(detector, root):
    start_time = time.time()

    returncode, stdout, _ = run_subprocess(
        [
            "cargo",
            "scout-audit",
            "--filter",
            detector,
            "--metadata",
            "--local-detectors",
            os.path.join(os.getcwd(), "detectors"),
        ],
        root,
    )

    if stdout is None:
        print(
            f"{RED}Failed to run integration tests in {root} - Metadata returned empty.{ENDC}"
        )
        return True

    detector_metadata = parse_json_from_string(stdout)

    if not isinstance(detector_metadata, dict):
        print("Failed to extract JSON:", detector_metadata)
        return True

    detector_key = detector.replace("-", "_")
    short_message = detector_metadata.get(detector_key, {}).get("short_message")

    returncode, _, stderr = run_subprocess(
        [
            "cargo",
            "scout-audit",
            "--filter",
            detector,
            "--local-detectors",
            os.path.join(os.getcwd(), "detectors"),
        ],
        root,
    )

    should_lint = root.endswith("vulnerable-example")
    if should_lint and short_message and short_message not in stderr:
        returncode = 1

    print_results(
        returncode,
        stderr,
        "integration-test",
        root,
        time.time() - start_time,
    )
    return returncode != 0


if __name__ == "__main__":
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
