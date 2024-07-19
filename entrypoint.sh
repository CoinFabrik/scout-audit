#!/bin/bash
set -e -o pipefail

# Change to the target directory
cd "$INPUT_TARGET"

# Run cargo scout-audit with the provided arguments
cargo scout-audit $INPUT_SCOUT_ARGS
