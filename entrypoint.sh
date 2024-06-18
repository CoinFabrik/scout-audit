#!/bin/bash
set -e -o pipefail

# Change to the target directory
cd "$INPUT_TARGET"

# Check if markdown output is required
if [[ "$INPUT_MARKDOWN_OUTPUT" == "true" ]]; then
  INPUT_SCOUT_ARGS="$INPUT_SCOUT_ARGS --output-format md --output-path /github/workspace/report.md"
fi

# Run cargo scout-audit with the provided arguments
cargo scout-audit $INPUT_SCOUT_ARGS
