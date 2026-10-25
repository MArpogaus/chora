#!/bin/bash
# T6 of D42: callers start programs with `choractl run` and keep plugins
# with `supervise` and `restart`. This guard fails when a script or the
# user-facing docs call the core command again. The core methods for
# program start still exist until T14 removes them; the core, its MCP
# pass-through, the method docs, plans and battle history may name them.
#
# Usage: tests/legacy-callers.sh
set -u
cd "$(dirname "$0")/.." || exit 1
files=(
    README.md
    CONTRIBUTING.md
    tests/*.sh
    tests/battle/lib.sh
    tests/battle/battle.sh
    tests/battle/w[0-9].sh
)
pattern='choractl (--name [a-z-]+ )?spawn|ctl (--name [a-z-]+ )?spawn'
if hits=$(grep -nE --exclude="$(basename "$0")" "$pattern" "${files[@]}"); then
    echo "FAIL: callers still spawn through the core:"
    echo "$hits"
    exit 1
fi
echo "ok: no caller uses choractl spawn"
