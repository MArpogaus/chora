#!/bin/bash
# round.sh stop keeps the capture folder of a session (persona round 5: the
# folder of a tiling+xp session was missing after the stop): stop waits for
# the copy and fails, with the folder named, when nothing was kept; a real
# tiling+xp session leaves shots/ and calls.log in the folder of its name.
#
# Usage: tests/gate-persona-keep.sh   BIN: the binaries (default target/debug)
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need_bins chora choractl chora-xp chora-tiling chora-notify chora-assistant chora-probe

dir=$(mktemp -d /tmp/persona-keep.XXXXXX)
tmp_dirs+=("$dir")
started=
stop_started() {
    [ -z "$started" ] || "$repo/tests/persona/round.sh" stop "$started" >/dev/null 2>&1
    started=
}
trap 'status=$?; stop_started; gate_cleanup $status' EXIT
export ROUND_DIR=$dir/round ROUND_STOP_WAIT=2
mkdir -p "$ROUND_DIR/run"

sleep 300 &
gone=$!
pids+=("$gone")
echo "PERSONA_SESSION_PID=$gone" >"$dir/gone.env"
echo "$dir/gone.env" >"$ROUND_DIR/run/gone.env"
out=$("$repo/tests/persona/round.sh" stop gone 2>&1)
status=$?
[ "$status" != 0 ] || fail "stop of a session that kept nothing said ok: $out"
grep -q "$ROUND_DIR/keep/gone" <<<"$out" || fail "the failure does not name the folder: $out"
ok "stop fails and names the capture folder when the session kept nothing"

rm -rf "$ROUND_DIR"
mkdir -p "$ROUND_DIR/run"
export ROUND_STOP_WAIT=30 CHORA_BIN=$bin
started=both
"$repo/tests/persona/round.sh" start both tiling+xp >"$dir/start.out" 2>&1 || fail "start: $(cat "$dir/start.out")"
env_file=$(cat "$ROUND_DIR/run/both.env")
core_env=$(tr '\0' '\n' <"/proc/$(sed -n 's/^CHORA_CORE_PID=//p' "$env_file")/environ")
grep -qx "CHORA_FLAVOR_DIR=$repo/flavors/tiling" <<<"$core_env" ||
    fail "the core of a tiling session has no CHORA_FLAVOR_DIR of the checkout, so Super+s finds no settings.sh: $(grep CHORA_FLAVOR <<<"$core_env")"
ok "the core of a tiling session knows the flavor folder of the checkout"
out=$("$repo/tests/persona/round.sh" stop both 2>&1)
status=$?
started=
[ "$status" = 0 ] || fail "stop of a tiling+xp session failed: $out"
keep=$ROUND_DIR/keep/both
[ -d "$keep/shots" ] || fail "no shots/ in $keep: $(ls -A "$keep" 2>&1)"
[ -f "$keep/calls.log" ] || fail "no calls.log in $keep: $(ls -A "$keep")"
grep -q calls.log <<<"$out" || fail "stop did not list the folder: $out"
[ ! -e "$env_file" ] || [ -d "$keep" ] || fail "the session directory went and the folder did not stay"
ok "a tiling+xp session keeps its capture folder: $(find "$keep" -mindepth 1 -maxdepth 1 -printf "%f ")"
