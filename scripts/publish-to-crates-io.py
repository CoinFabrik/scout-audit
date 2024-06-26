import argparse
from pathlib import Path
import subprocess
import requests
import toml
from contextlib import contextmanager
import time

RED = "\033[91m"
GREEN = "\033[92m"
BLUE = "\033[94m"
ENDC = "\033[0m"


@contextmanager
def timed_operation(msg, color):
    start_time = time.time()
    try:
        yield
    finally:
        duration = time.time() - start_time
        print(f"{color}[> {duration:.2f} sec]{ENDC} - {msg}.")


def run_subprocess(command, cwd, dry_run=False):
    if dry_run:
        command.append("--dry-run")
    result = subprocess.run(
        command, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    stdout = result.stdout.strip() if result.stdout else None
    stderr = result.stderr.strip() if result.stderr else None
    return result.returncode == 0, stdout, stderr


def get_crate_name(dir_path):
    try:
        data = toml.load(dir_path / "Cargo.toml")
        return data["package"]["name"]
    except (FileNotFoundError, KeyError) as e:
        print(f"{RED}Error loading name from TOML: {str(e)}{ENDC}")


def get_crate_version(file_path):
    try:
        data = toml.load(file_path)
        return data["package"]["version"]
    except (FileNotFoundError, KeyError) as e:
        print(f"{RED}Error loading version from TOML: {str(e)}{ENDC}")


def get_latest_crate_version(crate_name):
    url = f"https://crates.io/api/v1/crates/{crate_name}"
    headers = {"User-Agent": "Scout-CI/1.0"}
    try:
        response = requests.get(url, headers=headers)
        response.raise_for_status()
        data = response.json()
        return data.get("crate", {}).get("max_stable_version")
    except requests.RequestException as e:
        print(f"{RED}Failed to fetch data from crates.io: {str(e)}{ENDC}")


def is_crate_published(name, version):
    latest_version = get_latest_crate_version(name)
    return latest_version == version if latest_version else False


def publish_crate(name, version, path, dry_run):
    if not is_crate_published(name, version):
        print(f"Publishing {name} version {version} from {path} to crates.io.")
        with timed_operation(f"Attempted to publish crate in {path}", BLUE):
            success, _, stderr = run_subprocess(
                ["cargo", "publish"], cwd=path, dry_run=dry_run
            )
            if not success:
                print(f"{RED}Error: {stderr}{ENDC}")
            else:
                print(f"{GREEN}Successfully published crate to crates.io.{ENDC}")
                return True
    else:
        print(
            f"{RED}Error: {name} version {version} is already published and will not be republished.{ENDC}"
        )
    return False


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Publish a specific Rust crate.")
    parser.add_argument(
        "--crate-path", type=Path, required=True, help="Path to the crate directory."
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="Perform a dry run without publishing."
    )

    args = parser.parse_args()
    crate_path = args.crate_path
    crate_name = get_crate_name(crate_path)
    crate_version = get_crate_version(crate_path / "Cargo.toml")
    if crate_version and crate_name:
        if not publish_crate(crate_name, crate_version, crate_path, args.dry_run):
            exit(1)
    else:
        exit(1)
