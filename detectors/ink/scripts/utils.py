import json
import os
import subprocess
from typing import Tuple, Optional

RED = "\033[91m"
GREEN = "\033[92m"
BLUE = "\033[94m"
ENDC = "\033[0m"


def is_rust_project(dir_path):
    has_cargo_toml = os.path.isfile(os.path.join(dir_path, "Cargo.toml"))
    has_lib_rs = os.path.isfile(os.path.join(dir_path, "src", "lib.rs"))
    return has_cargo_toml and has_lib_rs


def parse_json_from_string(console_output):
    json_start, json_end = None, None
    brace_count = 0

    for i, char in enumerate(console_output):
        if char == "{":
            brace_count += 1
            if brace_count == 1:
                json_start = i
        elif char == "}":
            brace_count -= 1
            if brace_count == 0 and json_start is not None:
                json_end = i + 1
                break

    if json_start is not None and json_end is not None:
        json_str = console_output[json_start:json_end]
        try:
            return json.loads(json_str)
        except json.JSONDecodeError:
            return "Extracted string is not valid JSON"
    else:
        return console_output


def run_subprocess(command: list, cwd: str):
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    stdout = result.stdout.strip() if result.stdout else None
    stderr = result.stderr.strip() if result.stderr else None
    return (result.returncode, stdout, stderr)


def print_errors(errors):
    if errors:
        print(f"{RED}\nErrors detected in the following directories:{ENDC}")
        for error_dir in errors:
            print(f"• {error_dir}")
    else:
        print(f"{GREEN}\nNo errors found in the specified directory.{ENDC}")


def print_results(returncode, error_message, check_type, root, elapsed_time):
    allowed_check_types = ["clippy", "format", "udeps", "unit-test", "integration-test"]
    if check_type not in allowed_check_types:
        raise ValueError(
            f"Invalid check_type '{check_type}'. Allowed values are: {', '.join(allowed_check_types)}"
        )

    if check_type in ["clippy", "format", "udeps"]:
        issue_type = "issues"
        action_type = "check"
    elif check_type in ["unit-test", "integration-test"]:
        issue_type = "errors"
        action_type = "run"
    else:
        raise ValueError(f"Invalid check_type '{check_type}'.")

    message_color = RED if returncode != 0 else BLUE
    print(
        f"{message_color}[> {elapsed_time:.2f} sec]{ENDC} - Completed {check_type} {action_type} in: {root}."
    )
    if returncode != 0:
        print(f"\n{RED}{check_type.capitalize()} {issue_type} found in: {root}{ENDC}\n")
        if error_message is not None:
            for line in error_message.strip().split("\n"):
                print(f"| {line}")
            print("\n")
