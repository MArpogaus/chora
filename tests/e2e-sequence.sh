#!/bin/bash
# Runs the scripts of the e2e job of .github/workflows/ci.yaml in the
# order of the job, so that a failure that CI shows first shows locally.
# The commands are read from the job: each `run: tests/...` line, with
# `target/debug` replaced by BIN_DIR and the output directories `e2e`
# replaced by OUT_DIR. A script whose tool is missing on this machine
# skips itself; the runner lists the tools that this machine lacks.
#
# Usage: tests/e2e-sequence.sh [BIN_DIR [OUT_DIR]]
#   BIN_DIR  holds the built binaries (default target/debug)
#   OUT_DIR  receives the logs of each script (default: a temp dir)
#   E2E_KEEP_GOING=1  run every script and list the failed ones at the end
# Exit status: 0 every script passed or was skipped, 1 a script failed.
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo" || exit 2
bin=${1:-target/debug}
out=$(realpath "${2:-$(mktemp -d /tmp/e2e-sequence.XXXXXX)}")
mkdir -p "$out"
workflow=.github/workflows/ci.yaml
tools="sway foot grim jq waybar fuzzel wlrctl swaylock wlr-randr dbus-run-session mako swaybg swayidle kanshi xwayland-satellite Xwayland xeyes thunar"

missing=""
for tool in $tools; do
    command -v "$tool" >/dev/null 2>&1 || missing="$missing $tool"
done

commands=$(sed -n '/^  e2e:/,/^  [a-z-]*:$/p' "$workflow" |
    sed -n 's/^ *\(- \)\{0,1\}run: \(tests\/.*\)$/\2/p')
failed=""
number=0
while read -r command; do
    [ -n "$command" ] || continue
    number=$((number + 1))
    command=$(sed "s|\\be2e\\b|$out|g; s|target/debug|$bin|g" <<<"$command")
    name=$(basename "${command%% *}" .sh)
    echo "== $number. $command"
    if ! sh -c "$command" >"$out/$number-$name.log" 2>&1; then
        tail -n 5 "$out/$number-$name.log"
        failed="$failed $name"
        [ -n "${E2E_KEEP_GOING:-}" ] || break
    fi
done <<<"$commands"

[ -z "$missing" ] || echo "tools missing on this machine:$missing"
if [ -n "$failed" ]; then
    echo "FAIL: scripts failed:$failed (logs in $out)"
    exit 1
fi
echo "ok: the e2e sequence of $workflow passed (logs in $out)"
