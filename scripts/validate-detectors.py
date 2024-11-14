import os
import re
from fuzzywuzzy import process
from utils import BLUE, GREEN, RED, ENDC


def is_rust_project(dir_path):
    """Check if a directory contains a Rust project with a Cargo.toml and src/lib.rs."""
    errors = []
    has_cargo_toml = os.path.isfile(os.path.join(dir_path, "Cargo.toml"))
    has_cargo_toml_skip = os.path.isfile(os.path.join(dir_path, "Cargo.toml.skip"))
    has_lib_rs = os.path.isfile(os.path.join(dir_path, "src", "lib.rs"))

    if not (has_cargo_toml or has_cargo_toml_skip):
        errors.append(f"Missing Cargo.toml in {dir_path}.")
    if not has_lib_rs:
        errors.append(f"Missing src/lib.rs in {dir_path}.")

    return errors


def check_for_extra_files(directory):
    """Ensure there are no unexpected files in a given directory."""
    errors = []
    ignore_files = {"Cargo.lock", "Cargo.toml", "Cargo.toml.skip"}
    for item in os.listdir(directory):
        item_path = os.path.join(directory, item)
        if os.path.isfile(item_path) and item not in ignore_files:
            errors.append(f"Unexpected file found: {item_path}")
    return errors


def validate_naming_convention(example, detector_name):
    """Validate the naming convention of the example."""
    if not re.match(f"{re.escape(detector_name)}-\\d+$", example):
        return [
            f"Naming issue for '{example}' in {detector_name}: Expected format is {detector_name}-[number]."
        ]
    return []


def validate_example_structure(example_path, example_name):
    """Ensure each example has the required subdirectories with detailed errors."""
    errors = []
    expected_subs = ["vulnerable-example", "remediated-example"]
    actual_subs = [
        d
        for d in os.listdir(example_path)
        if os.path.isdir(os.path.join(example_path, d))
    ]

    for expected_sub in expected_subs:
        if expected_sub not in actual_subs:
            error_msg = f"Directory '{expected_sub}' not found in {example_path}."
            closest_match = process.extractOne(
                expected_sub, actual_subs, score_cutoff=80
            )
            if closest_match:
                error_msg += f" A similar directory exists: '{closest_match[0]}', please rename it to '{expected_sub}'."
            errors.append(error_msg)
        else:
            sub_errors = is_rust_project(os.path.join(example_path, expected_sub))
            for error in sub_errors:
                errors.append(error)

    return errors


def validate_examples(detector_path, test_cases):
    """Validate the structure and naming convention of test cases."""
    errors = []
    ignore_dirs = {"target", ".cargo"}
    detector_name = os.path.basename(detector_path)
    test_case_suffixes = set()

    for test_case in test_cases:
        test_case_path = os.path.join(detector_path, test_case)
        if os.path.basename(test_case_path) not in ignore_dirs:
            errors.extend(check_for_extra_files(test_case_path))
            errors.extend(validate_naming_convention(test_case, detector_name))
            suffix = test_case.split("-")[-1]
            if suffix in test_case_suffixes:
                errors.append(
                    f"Duplicate example number found in {detector_name}: {test_case}"
                )
            else:
                test_case_suffixes.add(suffix)
            errors.extend(validate_example_structure(test_case_path, test_case))
    return errors


def validate(test_cases_path, detectors_path):
    """Validate the structure of the test-cases directory."""
    errors = []

    # Directories to ignore while validating
    ignore_dirs = {"target", ".cargo"}

    detectors = [
        tc
        for tc in os.listdir(detectors_path)
        if os.path.isdir(os.path.join(detectors_path, tc)) and tc not in ignore_dirs
    ]

    for detector in detectors:
        detector_path = os.path.join(detectors_path, detector)

        # Validate that the detector exists
        if not os.path.exists(os.path.join(detectors_path, detector)):
            errors.append(f"Detector folder missing for {detector} in {detectors_path}")
        else:
            errors.extend(is_rust_project(os.path.join(detectors_path, detector)))

        # Check for unwanted files in the detector directory
        errors.extend(check_for_extra_files(detector_path))

        # Validate the detector test-cases
        test_cases = [
            e
            for e in os.listdir(test_cases_path)
            if os.path.isdir(os.path.join(test_cases_path, e)) and e not in ignore_dirs
        ]
        if not test_cases:
            errors.append(f"No test cases found in {detector}.")
        # else:
        # Validate each vulnerable and remediated example
        # errors.extend(validate_examples(detector_path, test_cases))

        ## Verify that there are an exact match of test cases and detectors and names are unique ## TODOOO
        # if len(test_cases) != len(set(test_cases)):
        # errors.append(f"Duplicate test cases found in {detector}.")
    return errors


if __name__ == "__main__":
    errors = []
    for blockchain in os.listdir("detectors"):
        detectors_path = f"detectors/{blockchain}/"
        test_cases_path = f"test-cases/{blockchain}/"
        if not os.path.isdir(detectors_path) or not os.path.isdir(test_cases_path):
            print(f"{RED}Skipping {blockchain} as it is not a directory.{ENDC}")
            continue
        errors.extend(validate(test_cases_path, detectors_path))
        if errors:
            print(f"{RED}\nValidation errors found for {blockchain}:{ENDC}")
            # for error in errors:
            # print(f"* {error}")
        else:
            print(
                f"{GREEN}[+] All detectors and test cases are valid for {blockchain}.{ENDC}"
            )

    if errors:
        exit(1)

