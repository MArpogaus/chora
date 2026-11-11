#!/bin/bash
# Self-test of the EXIT trap of tests/gate-lib.sh: a gate that ends,
# also with a failure, ends at once and leaves none of its programs
# behind, also not one that it stopped with SIGSTOP to force a late
# answer.
#
# Usage: tests/gate-lib-selftest.sh
# shellcheck disable=SC2016 # the inner gate expands its own arguments
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

timeout -k 1 10 bash -c '
. "$1/tests/gate-lib.sh"
trap gate_cleanup EXIT
sleep 60 &
pids+=($!)
echo $! >"$2/running"
sleep 60 &
pids+=($!)
echo $! >"$2/stopped"
kill -STOP "$!"
fail "the gate fails while a program stands still"
' gate "$repo" "$work" >/dev/null 2>&1
status=$?
failures=0
if [ "$status" = 124 ]; then
    echo "FAIL: the EXIT trap of the gate hangs"
    failures=1
fi
for program in running stopped; do
    pid=$(cat "$work/$program")
    for _ in $(seq 20); do
        kill -0 "$pid" 2>/dev/null || break
        sleep 0.1
    done
    if kill -0 "$pid" 2>/dev/null; then
        echo "FAIL: the $program program $pid outlives the gate"
        kill -KILL "$pid"
        failures=$((failures + 1))
    fi
done
[ "$failures" = 0 ] || exit 1
echo "ok: gate-lib-selftest"
