import glob
import os
import re
from typing import List
from dataclasses import dataclass

from utils import BLUE, GREEN, RED, ENDC


@dataclass
class ValidationError:
    path: str
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


def validate_example_naming(dir_path: str, prefix: str) -> List[str]:
    """Validate example naming convention, numbering, and structure."""
    errors = []
    pattern = re.compile(f"^{prefix}-\\d+$")

    # Check all items in directory - everything must match the pattern
    for item in os.listdir(dir_path):
        if not pattern.match(item):
            errors.append(
                f"Invalid item found in {dir_path}: {item}. Must match pattern '{prefix}-n'"
            )

    # Get all valid numbered examples
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


def validate(nightly: str, base_path: str) -> List[ValidationError]:
    """Validate detectors and test cases for a specific blockchain."""
    print(f"{BLUE}[*] Validating everything for {nightly}...{ENDC}")

    errors: List[ValidationError] = []

    detectors_path = f"{base_path}/{nightly}/detectors"
    if not os.path.isdir(detectors_path):
        errors.append(
            ValidationError(
                path=detectors_path,
                detector="",
                message=f"Missing detectors directory for {detectors_path}",
            )
        )
        return errors

    test_cases_path = f"{base_path}/test-cases"
    if not os.path.isdir(test_cases_path):
        errors.append(
            ValidationError(
                path=test_cases_path,
                detector="",
                message=f"Missing test-cases directory for {test_cases_path}",
            )
        )
        return errors

    # Difference now is we validate each target directory
    targets = [t for t in os.listdir(detectors_path) if t != "rust"]
    for target in targets:
        # We now validate each target directory against the test-cases
        print(f"{BLUE}[*] Validating {target}...{ENDC}")

        targets_path = os.path.join(detectors_path, target)
        detectors = os.listdir(targets_path)
        test_cases = os.listdir(os.path.join(test_cases_path, target))

        # Remove common ignored directories
        ignore_dirs = {"target", ".cargo"}
        detectors = {
            d
            for d in detectors
            if d not in ignore_dirs and os.path.isdir(os.path.join(targets_path, d))
        }

        test_cases = {
            t
            for t in test_cases
            if t not in ignore_dirs
            and os.path.isdir(os.path.join(test_cases_path, target, t))
        }

        # Check for mismatches between detectors and test cases
        missing_test_cases = detectors - test_cases
        for detector in missing_test_cases:
            errors.append(
                ValidationError(
                    path=os.path.join(detectors_path, target, detector),
                    detector=detector,
                    message=f"Detector '{detector}' has no corresponding test case",
                )
            )

        extra_test_cases = test_cases - detectors
        rust_detectors = set(
            d
            for d in os.listdir(os.path.join(detectors_path, "rust"))
            if d not in ignore_dirs
            and os.path.isdir(os.path.join(detectors_path, "rust", d))
        )

        for test_case in extra_test_cases:
            if test_case not in rust_detectors:
                errors.append(
                    ValidationError(
                        path=os.path.join(test_cases_path, target, test_case),
                        detector=test_case,
                        message=f"Test case '{test_case}' has no corresponding detector",
                    )
                )

        # Validate matching pairs

        for detector in detectors & test_cases:
            detector_path = os.path.join(detectors_path, target, detector)
            test_case_path = os.path.join(test_cases_path, target, detector)

            # Validate detector
            detector_errors = is_rust_project(detector_path)
            for error in detector_errors:
                errors.append(
                    ValidationError(
                        path=detector_path,
                        detector=detector,
                        message=error,
                    )
                )

            # Validate test case
            test_case_errors = (
                validate_test_case(test_case_path, detector)
                if not should_skip_validation(test_case_path)
                else []
            )
            for error in test_case_errors:
                errors.append(
                    ValidationError(
                        path=test_case_path,
                        detector=detector,
                        message=error,
                    )
                )

    return errors


def validate_detectors_and_test_cases(base_path: str) -> List[ValidationError]:
    """Validate all blockchains, their detectors and test-cases."""
    all_errors: List[ValidationError] = []

    # Get all nightlies
    nightly_dirs = glob.glob("nightly/20[0-9][0-9]-*-*")

    # Validate all nightlies
    for nightly in nightly_dirs:
        all_errors.extend(validate(nightly, base_path))

    return all_errors


def main():
    errors = validate_detectors_and_test_cases(".")

    if errors:
        print(f"{RED}Validation errors found:{ENDC}")
        for error in errors:
            print(f"{RED}[{error.path}] {error.detector} {error.message}{ENDC}")
        exit(1)
    else:
        print(f"{GREEN}All detectors and test cases are valid!{ENDC}")


if __name__ == "__main__":
    main()
