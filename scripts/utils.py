import glob
import subprocess
import os
import json

RED = "\033[91m"
GREEN = "\033[92m"
BLUE = "\033[94m"
ENDC = "\033[0m"

NIGHTLY_SUBDIRS = ["ink", "soroban", "substrate-pallets"]

def get_nightly_commands(command: str):
    """Dynamically generate commands for all nightly directories."""
    nightly_commands = []
    # Find all nightly directories
    nightly_dirs = glob.glob("nightly/20[0-9][0-9]-*-*")

    for nightly_dir in nightly_dirs:
        for subdir in NIGHTLY_SUBDIRS:
            detector_path = f"{nightly_dir}/detectors/{subdir}"
            if os.path.exists(detector_path):
                nightly_commands.append((detector_path, command))

    return nightly_commands

def run_subprocess(command: list, cwd: str):
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    stdout = result.stdout.strip() if result.stdout else None
    stderr = result.stderr.strip() if result.stderr else None
    return (result.returncode, stdout, stderr)


def get_or_default(map, k, default):
    return map[k] if k in map else default


def simple_runner(name, commands, opts):
    show_err = get_or_default(opts, "show_err", True)
    fail_fast = get_or_default(opts, "fail_fast", True)
    ret = 0
    for wd, cmd in commands:
        print(f"{BLUE}[>] Running '{name}' in {wd}{ENDC}")
        retcode, out, err = run_subprocess(cmd.split(" "), wd)
        if retcode != 0:
            if show_err:
                print(f"{RED}Error running {name} on {wd}:{ENDC}")
                print(f"STDOUT: {out}{ENDC}")
                print(f"STDERR: {err}{ENDC}")
            else:
                print(f"{RED}Error running {name} on {wd}{ENDC}")
            if fail_fast:
                return retcode
            ret = retcode
    return ret


def list_test_cases():
    ret = []
    for blockchain in os.listdir("test-cases"):
        path = f"test-cases/{blockchain}"
        if not os.path.isdir(path):
            continue
        for test_case in os.listdir(path):
            if not os.path.isdir(f"{path}/{test_case}"):
                continue
            if test_case == "target":
                continue
            if test_case[0:1] == ".":
                continue
            ret.append(f"{blockchain}/{test_case}")
    return ret


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


def is_rust_project(dir_path):
    has_cargo_toml = os.path.isfile(os.path.join(dir_path, "Cargo.toml"))
    has_lib_rs = os.path.isfile(os.path.join(dir_path, "src", "lib.rs"))
    return has_cargo_toml and has_lib_rs
