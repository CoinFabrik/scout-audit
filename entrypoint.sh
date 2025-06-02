#!/bin/bash
set -e -o pipefail

# Change to the target directory
cd "$INPUT_TARGET"

# Run cargo scout-audit with the provided arguments
cargo scout-audit --local-detectors /scout-audit/nightly/2024-07-11/detectors $INPUT_SCOUT_ARGS
