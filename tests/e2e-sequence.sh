#!/bin/bash
# Runs the scripts of the e2e job of .github/workflows/ci.yaml in the
# order of the job, so that a failure that CI shows first shows locally.
# The commands are read from the job: each `run: tests/...` line, with
# `target/debug` replaced by BIN_DIR and the output directory `e2e`
# replaced by OUT_DIR.
#
# A script that skips (a line `SKIP: ...` in its log) did not check what
# it names, so a skip fails the run unless tests/e2e-skips.txt lists the
# script with a reason. The file lists what a dev container cannot have;
# with CI set it is ignored, so that no skip passes in CI. The summary
# names every skip.
#
# Usage: tests/e2e-sequence.sh [BIN_DIR [OUT_DIR]]
#   BIN_DIR  holds the built binaries (default target/debug)
#   OUT_DIR  receives the logs of each script (default: a temp dir that
#            this script removes when the run passed)
#   E2E_KEEP_GOING=1  run every script and list the failed ones at the end
# Exit status: 0 every script passed and every skip is listed, 1 else.
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo" || exit 2
bin=${1:-target/debug}
temp_out=""
if [ -n "${2:-}" ]; then
    out=$(realpath "$2")
    mkdir -p "$out"
else
    out=$(mktemp -d /tmp/e2e-sequence.XXXXXX)
    temp_out=$out
fi
workflow=.github/workflows/ci.yaml
allowlist=tests/e2e-skips.txt

listed() {
    awk -F'|' -v name="$1" '$1 == name && length($2) > 0 { found = 1 } END { exit !found }' "$allowlist"
}

commands=$(sed -n '/^  e2e:/,/^  [a-z-]*:$/p' "$workflow" |
    sed -n 's/^ *\(- \)\{0,1\}run: \(tests\/.*\)$/\2/p')
if sed -n '/^  e2e:/,/^  [a-z-]*:$/p' "$workflow" | grep -q 'run: |'; then
    echo "FAIL: the e2e job of $workflow has a multi-line run: the sequence reads single-line run commands"
    exit 1
fi
if [ -z "$commands" ]; then
    echo "FAIL: no tests/ command in the e2e job of $workflow"
    exit 1
fi
failed=""
skipped=""
unlisted=""
number=0
while read -r command; do
    [ -n "$command" ] || continue
    number=$((number + 1))
    command=$(sed -E "s#(^| )e2e( |/|\$)#\\1$out\\2#g; s#target/debug#$bin#g" <<<"$command")
    name=$(basename "${command%% *}" .sh)
    log="$out/$number-$name.log"
    echo "== $number. $command"
    if ! sh -c "$command" >"$log" 2>&1; then
        tail -n 5 "$log"
        failed="$failed $name"
        [ -n "${E2E_KEEP_GOING:-}" ] || break
        continue
    fi
    reason=$(sed -n 's/^SKIP: //p' "$log" | head -n 1)
    [ -n "$reason" ] || continue
    skipped="$skipped $name"
    echo "SKIP $name: $reason"
    if [ -n "${CI:-}" ] || ! listed "$name"; then
        unlisted="$unlisted $name"
    fi
done <<<"$commands"

status=0
if [ -n "$failed" ]; then
    echo "FAIL: scripts failed:$failed (logs in $out)"
    status=1
fi
if [ -n "$unlisted" ]; then
    echo "FAIL: scripts skipped without an entry in $allowlist:$unlisted"
    status=1
fi
if [ "$status" = 0 ]; then
    echo "ok: the e2e sequence of $workflow passed; skipped, all listed:${skipped:- none}"
    [ -z "$temp_out" ] || rm -rf "$temp_out"
fi
exit "$status"
