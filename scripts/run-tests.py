import os
import argparse
import time
import tempfile
import json
import utils

from utils import (
    parse_json_from_string,
    print_errors,
    print_results,
    run_subprocess,
    is_rust_project,
)

def run_tests(detector):
    errors = []
    [blockchain, detector] = detector.split('/')
    directory = os.path.join("detectors", blockchain, 'test-cases', detector)
    print(f"\n{utils.GREEN}Performing tests in {directory}:{utils.ENDC}")
    if not os.path.exists(directory):
        print(f"{utils.RED}The specified directory does not exist.{utils.ENDC}")
        return errors

    for root, _, _ in os.walk(directory):
        if is_rust_project(root):
            if run_unit_tests(root):
                errors.append(root)
            if not run_integration_tests(blockchain, detector, root):
                errors.append(root)
    return errors

def convert_code(s):
    return s.replace('_', '-')

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



def run_integration_tests(blockchain, detector, root):
    start_time = time.time()

    local_detectors = os.path.join(os.getcwd(), "detectors", blockchain, "detectors")

    returncode, stdout, _ = run_subprocess(
        [
            "cargo",
            "scout-audit",
            "--filter",
            detector,
            "--metadata",
            "--local-detectors",
            local_detectors,
        ],
        root,
    )

    if stdout is None:
        print(
            f"{utils.RED}Failed to run integration tests in {root} - Metadata returned empty.{utils.ENDC}"
        )
        return True

    detector_metadata = parse_json_from_string(stdout)

    if not isinstance(detector_metadata, dict):
        print("Failed to extract JSON:", detector_metadata)
        return True

    detector_key = detector.replace("-", "_")
    short_message = detector_metadata.get(detector_key, {}).get("short_message")

    _, tempPath = tempfile.mkstemp(None, f'scout_{os.getpid()}_')

    returncode = None
    stderr = None

    returncode, _, stderr = run_subprocess(
        [
            "cargo",
            "scout-audit",
            "--local-detectors",
            local_detectors,
            "--output-format",
            "raw-json",
            "--output-path",
            tempPath,
        ],
        root,
    )

    if returncode != 0:
        print(f"{utils.RED}Scout failed to run.{utils.ENDC}")
        return False
    
    should_fail = "vulnerable" in root
    did_fail = False

    with open(tempPath) as file:
        detectors_triggered = {convert_code(json.loads(line.rstrip())['code']['code']) for line in file}
        did_fail = detector in detectors_triggered
        if should_fail != did_fail:
            explanation = "it failed when it shouldn't have" if did_fail else "it didn't fail when it should have"
            print(f"{utils.RED}Test case {root} didn't pass because {explanation}.{utils.ENDC}")
            return False

    print_results(
        returncode,
        stderr,
        "integration-test",
        root,
        time.time() - start_time,
    )
    return True


if __name__ == "__main__":
    print(run_subprocess(["cargo", "scout-audit", "--version"], ".")[1])
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
