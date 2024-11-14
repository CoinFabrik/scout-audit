import os
import re
import subprocess
from typing import List, Tuple
from dataclasses import dataclass

from utils import BLUE, GREEN, RED, ENDC


@dataclass
class ValidationError:
    blockchain: str
    detector: str
    message: str


def should_skip_validation(dir_path: str) -> bool:
    """Check if validation should be skipped for this directory."""
    return os.path.isfile(os.path.join(dir_path, "Cargo.toml.skip"))


def is_rust_project(dir_path: str) -> List[str]:
    """Check if a directory contains a valid Rust project."""
    errors = []
    required_files = {
        "Cargo.toml": "Missing Cargo.toml",
        "src/lib.rs": "Missing src/lib.rs",
    }

    for file_path, error_msg in required_files.items():
        if not os.path.isfile(os.path.join(dir_path, file_path)):
            errors.append(f"{error_msg} in {dir_path}")

    # Check for unexpected files/directories
    allowed_items = {"Cargo.toml", "Cargo.lock", "src", "target", ".cargo"}
    for item in os.listdir(dir_path):
        if item not in allowed_items:
            errors.append(f"Unexpected item found in {dir_path}: {item}")

    return errors


def run_cargo_check(dir_path: str) -> List[str]:
    """Run cargo check in the specified directory."""
    try:
        result = subprocess.run(
            ["cargo", "check"], cwd=dir_path, capture_output=True, text=True
        )
        if result.returncode != 0:
            return [f"Cargo check failed in {dir_path}: {result.stderr}"]
    except Exception as e:
        return [f"Failed to run cargo check in {dir_path}: {str(e)}"]
    return []


def validate_example_naming(dir_path: str, prefix: str) -> List[str]:
    """Validate example naming convention and numbering."""
    errors = []
    pattern = re.compile(f"^{prefix}-\\d+$")

    # Get all numbered examples
    examples = [d for d in os.listdir(dir_path) if pattern.match(d)]
    if not examples:
        return [f"No {prefix} examples found in {dir_path}"]

    # Check numbering starts at 1 and is sequential
    numbers = sorted([int(e.split("-")[-1]) for e in examples])
    expected_numbers = list(range(1, len(numbers) + 1))
    if numbers != expected_numbers:
        errors.append(
            f"Non-sequential or missing numbers in {prefix} examples. "
            f"Found: {numbers}, Expected: {expected_numbers}"
        )

    # Validate each example is a Rust project
    for example in examples:
        example_path = os.path.join(dir_path, example)
        errors.extend(is_rust_project(example_path))
        errors.extend(run_cargo_check(example_path))

    return errors


def validate_test_case(test_case_path: str, detector_name: str) -> List[str]:
    """Validate a single test case structure and contents."""
    errors = []

    # Check main directories exist
    vulnerable_path = os.path.join(test_case_path, "vulnerable")
    remediated_path = os.path.join(test_case_path, "remediated")

    if not os.path.isdir(vulnerable_path):
        errors.append(f"Missing 'vulnerable' directory in {test_case_path}")
    if not os.path.isdir(remediated_path):
        errors.append(f"Missing 'remediated' directory in {test_case_path}")

    # If directories exist, validate their contents
    if os.path.isdir(vulnerable_path):
        errors.extend(validate_example_naming(vulnerable_path, "vulnerable"))
    if os.path.isdir(remediated_path):
        errors.extend(validate_example_naming(remediated_path, "remediated"))

    # Check for unexpected items in test case root
    allowed_items = {"vulnerable", "remediated"}
    for item in os.listdir(test_case_path):
        if item not in allowed_items:
            errors.append(
                f"Unexpected item in test case directory {test_case_path}: {item}"
            )

    return errors


def validate_blockchain(blockchain: str, base_path: str) -> List[ValidationError]:
    """Validate detectors and test cases for a specific blockchain."""
    print(f"{BLUE}[*] Validating {blockchain}...{ENDC}")
    errors: List[ValidationError] = []

    detectors_path = os.path.join(base_path, "detectors", blockchain)
    test_cases_path = os.path.join(base_path, "test-cases", blockchain)

    # Skip if either path doesn't exist
    if not os.path.isdir(detectors_path) or not os.path.isdir(test_cases_path):
        errors.append(
            ValidationError(
                blockchain=blockchain,
                detector="",
                message=f"Missing detector or test-case directory for blockchain {blockchain}",
            )
        )
        return errors

    # Get all detectors and test cases for this blockchain
    detectors = set(os.listdir(detectors_path))
    test_cases = set(os.listdir(test_cases_path))

    # Remove common ignored directories
    ignore_dirs = {"target", ".cargo"}
    detectors = {
        d
        for d in detectors
        if d not in ignore_dirs and os.path.isdir(os.path.join(detectors_path, d))
    }
    test_cases = {
        t
        for t in test_cases
        if t not in ignore_dirs and os.path.isdir(os.path.join(test_cases_path, t))
    }

    # Check for mismatches between detectors and test cases
    missing_test_cases = detectors - test_cases
    extra_test_cases = test_cases - detectors

    for detector in missing_test_cases:
        errors.append(
            ValidationError(
                blockchain=blockchain,
                detector=detector,
                message=f"Detector '{detector}' has no corresponding test case",
            )
        )

    for test_case in extra_test_cases:
        errors.append(
            ValidationError(
                blockchain=blockchain,
                detector=test_case,
                message=f"Test case '{test_case}' has no corresponding detector",
            )
        )

    # Validate matching pairs
    for detector in detectors & test_cases:
        detector_path = os.path.join(detectors_path, detector)
        test_case_path = os.path.join(test_cases_path, detector)

        # Validate detector
        detector_errors = is_rust_project(detector_path)
        detector_errors.extend(run_cargo_check(detector_path))
        for error in detector_errors:
            errors.append(
                ValidationError(blockchain=blockchain, detector=detector, message=error)
            )

        # Validate test case
        test_case_errors = validate_test_case(test_case_path, detector)
        for error in test_case_errors if not should_skip_validation(test_case_path) else []:
            errors.append(
                ValidationError(blockchain=blockchain, detector=detector, message=error)
            )

    return errors


def validate_detectors_and_test_cases(base_path: str) -> List[ValidationError]:
    """Validate all blockchains, their detectors and test-cases."""
    all_errors: List[ValidationError] = []

    # Get all blockchain directories
    blockchains = set(os.listdir(os.path.join(base_path, "detectors")))
    blockchains &= set(os.listdir(os.path.join(base_path, "test-cases")))

    for blockchain in blockchains:
        all_errors.extend(validate_blockchain(blockchain, base_path))

    return all_errors


def main():
    errors = validate_detectors_and_test_cases(".")

    if errors:
        print(f"{RED}Validation errors found:{ENDC}")
        current_blockchain = None
        for error in errors:
            if current_blockchain != error.blockchain:
                current_blockchain = error.blockchain
                print(f"\n{BLUE}Blockchain: {current_blockchain}{ENDC}")
            if error.detector:
                print(f"{RED}[{error.detector}] {error.message}{ENDC}")
            else:
                print(f"{RED}{error.message}{ENDC}")
        exit(1)
    else:
        print(f"{GREEN}All detectors and test cases are valid!{ENDC}")


if __name__ == "__main__":
    main()
