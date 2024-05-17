#!/bin/bash
set -e -o pipefail
TARGET=$INPUT_TARGET
cd $INPUT_TARGET

if [[ $INPUT_MARKDOWN_OUTPUT == "true" ]]
then
INPUT_SCOUT_ARGS="$INPUT_SCOUT_ARGS --output-format md --output-path /github/workspace/report.md"
fi

cargo scout-audit $INPUT_SCOUT_ARGS