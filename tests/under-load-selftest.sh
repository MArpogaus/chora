#!/bin/bash
# Self-test of tests/under-load.sh. A run that hangs ends at its time
# limit and counts as failed, so a hung gate cannot hang the tool (FF1
# rw-4); a run that passes in time still passes; the busy loops end with
# the tool.
#
# Usage: tests/under-load-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

start=$SECONDS
timeout -k 1 20 "$repo/tests/under-load.sh" --runs 1 --cpus 1 --timeout 2 --keep "$work/hung" \
    -- sleep 60 >"$work/hung.out" 2>&1
status=$?
took=$((SECONDS - start))
if [ "$status" = 124 ] || [ "$status" = 137 ]; then
    echo "FAIL: a hung run hangs under-load.sh (ended by the outer limit after $took s)"
    failures=$((failures + 1))
elif [ "$status" = 0 ]; then
    echo "FAIL: a hung run counts as passed"
    failures=$((failures + 1))
elif [ "$took" -gt 10 ]; then
    echo "FAIL: a run with a limit of 2 s ended after $took s"
    failures=$((failures + 1))
elif ! grep -q '^run 1: FAIL .*timed out after 2 s' "$work/hung.out"; then
    echo "FAIL: a hung run does not say it timed out:"
    sed 's/^/  /' "$work/hung.out"
    failures=$((failures + 1))
fi

setsid timeout -k 1 20 "$repo/tests/under-load.sh" --runs 2 --cpus 2 --timeout 5 --keep "$work/ok" \
    -- true >"$work/ok.out" 2>&1 &
session=$!
if ! wait "$session"; then
    echo "FAIL: runs that pass in time do not pass:"
    sed 's/^/  /' "$work/ok.out"
    failures=$((failures + 1))
fi
if left=$(pgrep -s "$session"); then
    echo "FAIL: a busy loop outlives under-load.sh: $(echo "$left" | tr '\n' ' ')"
    pkill -s "$session"
    failures=$((failures + 1))
fi

[ "$failures" = 0 ] && echo "under-load selftest: ok"
[ "$failures" = 0 ]
