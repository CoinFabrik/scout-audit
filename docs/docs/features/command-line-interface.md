---
sidebar_position: 10
---

# Command Line Interface (CLI)

The command line interface is designed to allow you to run Scout on an entire project. It is especially useful for auditing or performing a final review of your code.

## Installation

Make sure that [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) is installed on your computer. Then, install Scout with the following command:

```bash
cargo install cargo-scout-audit
```

## Usage

To run Scout on your project execute the following command:

```bash
cargo scout-audit
```

:bulb: Scout supports [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). When run on a workspace, Scout will be executed on all packages specified as members of the workspace.

:warning: Make sure that your smart contracts compile properly. Scout won't run if any compilation errors exist.

In the table below, we specify all the option available for the CLI.

| Command/Option                                                             | Explanation                                                                                                                                        |
| -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cargo scout-audit`                                                        | Runs the static analyzer on the current directory                                                                                                  |
| `cargo scout-audit --help`                                                 | Provides a brief explanation of all the available commands and their usage.                                                                        |
| `cargo scout-audit --manifest-path <PATH_TO_CARGO_TOML>`                   | This option is used to specify the path to the Cargo.toml file that you want to analyze.                                                           |
| `cargo scout-audit --profile <PROFILE_NAME>`                               | This option allows you to analyze code using specific group of detectors, configured previously on `$HOME/.config/scout/(ink/soroban)-config.toml` |
| `cargo scout-audit --filter <DETECTOR_LIST_SEPARATED_BY_COMAS>`            | This option allows you to analyze code using specific detectors. Provide a comma-separated list of detectors for this purpose.                     |
| `cargo scout-audit --exclude <DETECTOR_LIST_SEPARATED_BY_COMAS>`           | With this command, you can exclude specific detectors from the analysis. You need to give a comma-separated list of the detectors to be excluded.  |
| `cargo scout-audit --list-detectors`                                       | Display a list of all available detectors.                                                                                                         |
| `cargo scout-audit --version`                                              | Displays the current version of the static analyzer.                                                                                               |
| `cargo scout-audit --verbose`                                              | Print additional information on run                                                                                                                |
| `cargo scout-audit --local-detectors <PATH_TO_FOLDER>`                     | Uses the detectors of a local folder. This considers the sub-folders as detectors.                                                                 |
| `cargo scout-audit --output-format [text,json,html,sarif,pdf,md,markdown]` | Sets the output format. Selecting `json`, `html`, `sarif`, `markdown`, or `pdf` will create a file with the output                                 |
| `cargo scout-audit --output-path <PATH_TO_OUTPUT_FILE>`                    | Sets the output path. If a format was selected, this will replace the default file with the given one                                              |
