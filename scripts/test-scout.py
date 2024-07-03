from contextlib import contextmanager
import os
import subprocess
import time
import toml
from pathlib import Path

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


def get_crate_version(file_path):
    try:
        data = toml.load(file_path)
        return data["package"]["version"]
    except (FileNotFoundError, KeyError) as e:
        print(f"{RED}Error loading version from TOML: {str(e)}{ENDC}")


def run_scout():
    with timed_operation("Running cargo-scout-audit", BLUE):
        result = subprocess.run(
            ["cargo", "scout-audit"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0


def run_scout_with_markdown_output():
    with timed_operation("Running cargo-scout-audit", BLUE):
        result = subprocess.run(
            [
                "cargo",
                "scout-audit",
                "-o=md",
            ],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        assert Path("report.md").exists()
        assert Path("report.md").stat().st_size > 0


def test_scout_version():
    with timed_operation("Scout version should be the latest", GREEN):
        crate_version = get_crate_version(
            Path("apps") / "cargo-scout-audit" / "Cargo.toml"
        )
        result = subprocess.run(
            ["cargo", "scout-audit", "--version"], capture_output=True, text=True
        )
        assert result.returncode == 0
        assert result.stdout.strip() == f"cargo-scout-audit {crate_version}"


def main():
    test_scout_version()
    # Navigate to the contract directory and run the scout
    os.chdir(Path("apps") / "cargo-scout-audit" / "tests" / "contract")

    run_scout()
    run_scout_with_markdown_output()


if __name__ == "__main__":
    main()
