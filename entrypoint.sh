#!/bin/bash
set -e -o pipefail

# Change to the target directory
cd "$INPUT_TARGET"

# Run cargo scout-audit with the provided arguments
cargo scout-audit --local-detectors /scout-audit/detectors $INPUT_SCOUT_ARGS
