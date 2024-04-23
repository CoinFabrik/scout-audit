import argparse
from pathlib import Path


def main(crate, dry_run):
    root_path = Path(__file__).parent.parent
    crates_paths = {
        "scout-audit-clippy-utils": root_path / "apps" / "scout-audit-clippy-utils",
        "scout-audit-internal": root_path / "scout-audit-internal",
        "cargo-scout-audit": root_path / "cargo-scout-audit",
        "scout-audit-dylint-linting": root_path / "scout-audit-dylint-linting",
    }

    selected_crates = (
        [crates_paths[crate]] if crate in crates_paths else crates_paths.values()
    )

    # for path in selected_crates:
    #     name, version = get_crate_details(path / "Cargo.toml")
    #     if is_crate_published(name, version, path):
    #         print(f"{name} {version} is already published.")
    #     else:
    #         publish_crate(path, dry_run)
    #         if not dry_run:
    #             while not is_crate_published(name, version, path):
    #                 print(f"Waiting for {name} {version} to be published...")
    #                 time.sleep(10)
    #             print(f"{name} {version} is published.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Publish Rust crates.")
    parser.add_argument(
        "--crate",
        choices=[
            "scout-audit-clippy-utils",
            "scout-audit-internal",
            "cargo-scout-audit",
            "scout-audit-dylint-linting",
            "all",
        ],
        default="all",
        help="Specify which crate to publish",
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="Perform a dry run without publishing."
    )
    args = parser.parse_args()
    main(args.crate, args.dry_run)
