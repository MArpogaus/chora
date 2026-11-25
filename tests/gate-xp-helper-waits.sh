#!/usr/bin/env bash
# Self-check of tests/gate-xp-helper.sh (rxm2-5, rxm2-6): the cells wait on
# the condition with `ctl wait`, not on a fixed wait or a polling loop.
#
# Usage: tests/gate-xp-helper-waits.sh
set -eu
cd "$(dirname "$0")/.."
file=tests/gate-xp-helper.sh
fail() { echo "FAIL: $*" >&2; exit 1; }

grep -Eq '^[[:space:]]*pause[[:space:]]+[0-9]' "$file" &&
    fail "$file: a cell still calls pause (a fixed wait): $(grep -nE '^[[:space:]]*pause[[:space:]]+[0-9]' "$file" | tr '\n' ' ')"
grep -Eq '^pause\(\)' "$file" && fail "$file: pause is still defined"
click=$(awk '/^xp30-click\)/{on=1} on{print} /^    ;;/{if(on) exit}' "$file")
echo "$click" | grep -q 'wait --gone --timeout 4' ||
    fail "xp30-click does not wait with ctl wait --gone --timeout 4"
echo "$click" | grep -q 'seq 40' && fail "xp30-click polls in a loop"
echo "ok: the xp helper cells wait on conditions"
