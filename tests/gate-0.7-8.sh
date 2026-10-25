#!/bin/bash
# Gate check 8 of 0.7: `choractl run foot` starts foot in its own systemd
# scope when the user manager answers, and directly without one. foot
# reads XDG_ACTIVATION_TOKEN of the run and asks for activation before
# its first map, so the `surface.created` of its window carries the
# token. foot is not installed here, so the answer is SKIP.
#
# Usage: tests/gate-0.7-8.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need foot
out=$(mktemp -d /tmp/gate8.XXXXXX)
session_start 1280x800 # its EXIT trap ends the session, the events and the run
"$bin/choractl" --name agent events surface.created >"$out/events" 2>/dev/null &
pids+=($!)
sleep 1
pid=$(ctl run foot | sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p')
test -n "$pid" || fail "the run of foot printed no pid"
run_pid "$pid"
"$bin/choractl" --name agent wait "app_id:foot kind:surface" --timeout 20 >/dev/null ||
    fail "no foot window appeared"
if systemctl --user is-system-running >/dev/null 2>&1; then
    grep -q '\.scope' "/proc/$pid/cgroup" ||
        fail "foot runs in no systemd scope of its own"
    ok_kind=its own systemd scope
else
    ok_kind=the direct fallback without a user systemd
fi
for _ in $(seq 20); do
    grep -q '"token":"[^"]*"' "$out/events" && break
    sleep 0.5
done
grep -q '"token":"[^"]*"' "$out/events" ||
    fail "the surface.created of foot carries no activation token"
ok "choractl run started foot in $ok_kind with the token in surface.created"
