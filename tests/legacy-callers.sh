#!/bin/bash
# T6 of D42: callers start programs with `choractl run` and keep plugins
# with `supervise` and `restart`. This guard fails when a script, a test
# client or the user-facing docs call a core method for program start
# again: the old `choractl spawn`, `process.spawn`, `process.wait` or
# `plugins.restart`. The core methods still exist until T14 removes them;
# the core, its MCP pass-through in `mcp.rs`, the method docs, plans and
# battle history may name them. `tests/deprecated-start.sh` checks the
# deprecated path on purpose and is excluded; T14 removes it here too.
#
# Usage: tests/legacy-callers.sh
set -u
cd "$(dirname "$0")/.." || exit 1
files=(
    README.md
    CONTRIBUTING.md
    tests/*.sh
    tests/*.py
    tests/battle/lib.sh
    tests/battle/battle.sh
    tests/battle/w[0-9].sh
)
pattern='choractl (--name [a-z-]+ )?spawn|ctl (--name [a-z-]+ )?spawn|process\.(spawn|wait)|plugins\.restart'
if hits=$(grep -nE --exclude="$(basename "$0")" --exclude=deprecated-start.sh "$pattern" "${files[@]}"); then
    echo "FAIL: callers still start programs through the core:"
    echo "$hits"
    exit 1
fi
echo "ok: no caller uses the core methods for program start"
