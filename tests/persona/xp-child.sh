#!/bin/bash
# The XP that the cube of a construct session starts must connect, also when
# the binaries are a snapshot outside target/ (as tests/persona/round.sh runs
# them): chora-xp finds its theme by the source tree of its executable
# otherwise, and a copy of the binaries has none (round 4: "XP did not start
# within 10 s" in all three construct sessions).
#
# Usage: tests/persona/xp-child.sh
# Needs: target/debug binaries, jq, setsid and dbus-run-session.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
dir=$(mktemp -d /tmp/persona-xp-child.XXXXXX)
spid=
env_file=
# shellcheck disable=SC2329 # runs as the EXIT trap
cleanup() {
    trap - EXIT
    if [[ -n $env_file ]]; then
        "$here/session.sh" --stop "$env_file" 2>/dev/null || true
    fi
    if [[ -n $spid ]]; then
        wait "$spid" 2>/dev/null || true
    fi
    rm -rf "$dir"
}
trap cleanup EXIT
fail() {
    echo "FAIL: $*" >&2
    exit 1
}

mkdir "$dir/bin"
for b in "$repo"/target/debug/chora*; do
    [[ -f $b && -x $b && $b != *.d ]] && cp "$b" "$dir/bin/"
done
mkdir "$dir/s"
CHORA_BIN=$dir/bin PERSONA_DIR=$dir/s "$here/session.sh" construct >"$dir/out" 2>"$dir/err" &
spid=$!
for _ in $(seq 150); do
    [[ -s $dir/out ]] && break
    kill -0 "$spid" 2>/dev/null || fail "session ended: $(cat "$dir/err")"
    sleep 0.2
done
[[ -s $dir/out ]] || fail "no session"
env_file=$(sed -n 's/^PERSONA_ENV=//p' "$dir/out")
ctl() {
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$env_file") XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$env_file") \
        PATH=$(sed -n 's/^PATH=//p' "$env_file") "$dir/bin/choractl" "$@"
}
runtime=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$env_file")

ctl wait owner:construct --timeout 20 >/dev/null 2>&1 || ctl call construct construct.list '{}' >/dev/null || fail "the construct does not answer"
ctl call construct cube.open '{}' >/dev/null || fail "cube.open failed"
for _ in $(seq 100); do
    ctl state xp >/dev/null 2>&1 && break
    sleep 0.1
done
ctl state xp >/dev/null 2>&1 || {
    log=$(cat "$runtime"/chora-construct-cube-*.log 2>/dev/null | tail -n 1)
    fail "XP did not connect within 10 s of cube.open; its last line: ${log:-none}"
}
echo "PASS: the XP of the cube connects from a snapshot of the binaries"
