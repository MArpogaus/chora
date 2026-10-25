#!/bin/bash
# T6 of D42: callers start programs with `choractl run` and keep plugins
# with `supervise` and `restart`. This guard fails when a script, a test
# client or the user-facing docs call a core method for program start
# again: the old `choractl spawn`, `process.spawn`, `process.wait` or
# `plugins.restart`. T14 removed the core methods; plans and battle
# history may name them.
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
if hits=$(grep -nE --exclude="$(basename "$0")" "$pattern" "${files[@]}"); then
    echo "FAIL: callers still start programs through the core:"
    echo "$hits"
    exit 1
fi
echo "ok: no caller uses the core methods for program start"
