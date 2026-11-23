#!/bin/bash
# Gate of the stray keys on the path of a person (co-25, cell keys, part
# persona-path): the steps 0 to 27 of the persona Max of round p2-r2
# (media/2026-11-20/persona-r2-max/actions.jsonl on claude/session-log),
# replayed by tests/persona/act.py in a persona session of the construct
# with a chora-probe window that reports its keys (as round.sh probe).
#
# The path: a click on the probe pane, Programs and the Foot row of the top
# bar (the terminal comes as a pane the camera flies into), Alt+Escape,
# three drags on the Foot pane, Tab, a click on the Foot pane, Return, a
# click on the probe pane, then Super+d, Super+q and Super+Tab. The click
# on a pane in walk mode gives its window the keyboard where it stands; the
# camera is in no pane. No Super chord that the construct does not bind may
# reach the probe then, nor the Super key itself. Foot is the one program
# of the Programs list (the row of the persona). A look comes before each
# action, as the persona did, so act.py scales the pixels of an 800x500
# capture.
#
# Usage: tests/gate-construct-keys-persona.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need python3
need foot
need_fuzzel 1.11
need jq
need setsid
need_bins chora choractl chora-construct chora-assistant chora-probe

act() { PERSONA_ENV=$dir/session.env python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null || fail "act.py $*"; }
env_of() { sed -n "s/^$1=//p" "$dir/session.env"; }
admin() { CHORA_SOCKET=$(env_of CHORA_SOCKET) XDG_RUNTIME_DIR=$(env_of XDG_RUNTIME_DIR) "$bin/choractl" "$@"; }
shown() { admin query "app_id:$1 kind:surface" | jq length; }
until_ok() {
    local _
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}

spid=
probe_pid=
dir=$(mktemp -d /tmp/keys-persona.XXXXXX)
tmp_dirs+=("$dir")
stop_session() {
    [ -z "$probe_pid" ] || kill "$probe_pid" 2>/dev/null
    [ -z "$spid" ] || {
        "$repo/tests/persona/session.sh" --stop "$dir/session.env" >/dev/null 2>&1
        wait "$spid" 2>/dev/null
        spid=
    }
}
trap 'status=$?; stop_session; gate_cleanup $status' EXIT

mkdir -p "$dir/data/applications"
printf '[Desktop Entry]\nType=Application\nName=Foot\nExec=foot -a foot sh -c "sleep 600"\n' \
    >"$dir/data/applications/foot.desktop"
(
    export XDG_DATA_DIRS=$dir/none
    export XDG_CACHE_HOME=$dir/cache
    PERSONA_DIR=$dir CHORA_BIN=$bin CHORA_TEST_FLAGS=--test-injected-press-is-real \
        exec "$repo/tests/persona/session.sh" construct 1280x800 >"$dir.out" 2>"$dir.err"
) &
spid=$!
until_ok "grep -q '^PERSONA_ENV=' '$dir.out'" 200 || fail "the construct session did not start: $(cat "$dir.err")"
admin grant choractl screen.capture >/dev/null || fail "grant of the capture"

socket=$(basename "$(sed -n 's/^WAYLAND_DISPLAY=//p' "$dir/headless.out")")
report=$dir/probe.report
WAYLAND_DISPLAY=$socket XDG_RUNTIME_DIR=$(env_of XDG_RUNTIME_DIR) \
    "$bin/chora-probe" window 200 200 200 --report >"$report" 2>"$report.err" </dev/null &
probe_pid=$!
until_ok '[ "$(shown chora-probe)" -gt 0 ]' 40 || fail "the probe shows no window: $(cat "$report.err")"
sleep 2
cap() { [ -z "${SHOTS:-}" ] || admin capture "$SHOTS/construct-keys-persona-$1.png" >/dev/null || fail "capture $1"; }

act look
act click 400 300
act look
act click 93 32
until_ok 'XDG_RUNTIME_DIR=$(env_of XDG_RUNTIME_DIR) session_pids -x fuzzel >/dev/null' 80 || fail "Programs started no launcher"
sleep 2
act look
cap programs
act click 344 184
until_ok '[ "$(shown foot)" -gt 0 ]' 120 || fail "the Foot row of Programs started no foot"
sleep 2
act look
act key Alt+Escape
sleep 1
act look
act drag 650 250 450 240
act look
act drag 640 250 420 250
act look
act drag 610 225 470 225
act look
act key Tab
act look
act click 653 250
act look
act key Return
act look
act click 400 250
act look
cap probe-clicked
: >"$report.before"
grep '^key' "$report" >"$report.before"
act key Super+d
act look
act key Super+q
act look
act key Super+Tab
act look
sleep 1
cap after
[ -s "$report.before" ] && fail "persona-path: keys reached the probe before the Super chords: $(tr '\n' ' ' <"$report.before")"
stray=$(grep '^key' "$report" | tr '\n' ' ')
[ -z "$stray" ] || fail "persona-path: a Super chord of the persona reached the probe, which a click in walk mode focused: $stray(probe report $(tr '\n' ' ' <"$report"))"
[ "$(shown chora-probe)" -gt 0 ] || fail "persona-path: the probe window is gone"
ok "persona-path: after the steps of persona Max (a click on the probe pane in walk mode), Super+d, q and Tab reach no window, nor Super itself"
