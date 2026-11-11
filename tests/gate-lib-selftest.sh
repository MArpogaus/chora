#!/bin/bash
# Self-test of tests/gate-lib.sh. Its EXIT trap: a gate that ends, also
# with a failure, ends at once and leaves none of its programs behind,
# also not one that it stopped with SIGSTOP to force a late answer. Its
# lock: a gate script waits while another gate holds the lock, and says
# so.
#
# Usage: tests/gate-lib-selftest.sh
# shellcheck disable=SC2016 # the inner gate expands its own arguments
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
export CHORA_GATE_LOCK=$work/gate.lock
unset CHORA_GATE_LOCK_HELD

timeout -k 1 10 bash -c '
. "$1/tests/gate-lib.sh"
trap gate_cleanup EXIT
runs() { until [ "$(ps -o comm= -p "$1")" = sleep ]; do sleep 0.05; done; }
sleep 60 &
pids+=($!)
echo $! >"$2/running"
sleep 60 &
pids+=($!)
echo $! >"$2/stopped"
runs "$!"
runs "$(cat "$2/running")"
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
(
    flock 9
    touch "$work/held"
    sleep 2
    date +%s.%N >"$work/released"
) 9>"$CHORA_GATE_LOCK" &
for _ in $(seq 50); do
    [ -e "$work/held" ] && break
    sleep 0.1
done
timeout 10 bash -c '. "$1/tests/gate-lib.sh"; date +%s.%N >"$2/started"' gate "$repo" "$work" >"$work/out" 2>&1
wait
if ! awk -v a="$(cat "$work/started")" -v b="$(cat "$work/released")" 'BEGIN { exit !(a >= b) }'; then
    echo "FAIL: a gate script runs while another gate holds the lock"
    failures=$((failures + 1))
fi
if ! grep -q '^gate: waits for' "$work/out"; then
    echo "FAIL: a gate script that waits for the lock does not say so: $(cat "$work/out")"
    failures=$((failures + 1))
fi
[ "$failures" = 0 ] || exit 1
echo "ok: gate-lib-selftest"
