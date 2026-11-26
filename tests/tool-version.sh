#!/bin/sh
# Prints the first version number of `TOOL --version` (nothing when there is none). The one parse of
# tests/merge-gate.sh (tool_minimums) and demo/tour/launch.py (supported_fuzzel).
"$1" --version 2>/dev/null | sed -n 's/^[^0-9]*\([0-9][0-9.]*\).*/\1/p' | head -n 1
