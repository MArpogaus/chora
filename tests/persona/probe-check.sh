#!/bin/bash
# The T5 probe of a round: `round.sh probe NAME` starts chora-probe in the
# session of NAME with the socket name (WAYLAND_DISPLAY), a window of it is
# in the scene, and a probe that died turns `check.py end --probe T5` red
# (round 4: the probe got the socket path, died with "Broken pipe", and T5
# passed on an empty report).
#
# Usage: tests/persona/probe-check.sh
# Needs: target/debug binaries, jq, setsid and dbus-run-session.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
export ROUND_DIR
ROUND_DIR=$(mktemp -d /tmp/persona-probe-check.XXXXXX)
round=$here/round.sh
started=
# shellcheck disable=SC2329 # runs as the EXIT trap
cleanup() {
    trap - EXIT
    [[ -z $started ]] || "$round" stop probe-check >/dev/null 2>&1 || true
    rm -rf "$ROUND_DIR"
}
trap cleanup EXIT
fail() {
    echo "FAIL: $*" >&2
    exit 1
}

"$round" snapshot >/dev/null
start_out=$("$round" start probe-check construct 2>&1) || fail "round.sh start failed: $start_out"
env_file=$(sed -n 's/^PERSONA_ENV=//p' <<<"$start_out")
[[ -n $env_file ]] || fail "no session"
started=1
export PERSONA_ENV=$env_file
ctl() {
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$env_file") XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$env_file") \
        "$ROUND_DIR/bin/choractl" "$@"
}

probe_out=$("$round" probe probe-check 2>&1) || fail "round.sh probe failed: $probe_out"
report=$(sed -n 's/^PROBE_REPORT=//p' <<<"$probe_out")
[[ $probe_out == "PROBE_REPORT=$ROUND_DIR/run/probe-check.probe" ]] || fail "round.sh probe printed more than the report path: $probe_out"
[[ -n $report ]] || fail "round.sh probe printed no PROBE_REPORT"
ctl wait 'kind:surface' --timeout 10 >/dev/null 2>&1 || fail "no window in the scene after round.sh probe"
ctl query 'kind:surface' | jq -e 'any(.[]; .surface.app_id == "chora-probe")' >/dev/null || fail "no chora-probe window in the scene"
python3 -I "$repo/tests/persona/check.py" begin
out=$(python3 -I "$repo/tests/persona/check.py" end --probe "$report" T5) || fail "T5 is red with a live probe: $out"
echo "$out" | grep -q 'task T5 pass' || fail "T5 did not pass with a live probe: $out"

pid=$(cat "$ROUND_DIR/run/probe-check.probe.pid")
kill "$pid"
ctl wait 'kind:surface' --gone --timeout 10 >/dev/null 2>&1 || fail "the window of the probe stayed after the probe died"
if out=$(python3 -I "$repo/tests/persona/check.py" end --probe "$report" T5); then
    fail "T5 passed with a dead probe: $out"
fi
echo "$out" | grep -q 'task T5 FAIL' || fail "T5 did not name the dead probe: $out"
echo "PASS: round.sh probe starts a probe that T5 reads, and a dead one is red"
