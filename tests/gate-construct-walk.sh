#!/bin/bash
# Gate of step 14 of the construct look v2: the walk in pane steps (D8).
# The clock of the animations is stopped (clock.step), so that each cell
# reads the place of the camera, in construct.list, at an exact time
# after the press of the key, which the core sends as a real key.
#
# 1. A tap of w glides one pane step (800 px) in 320 ms, is part of the
#    way at 160 ms and stays there: nothing overshoots.
# 2. After the walk, the keyboard works: q turns the camera and Super+/
#    shows and hides the help.
# The hold of a key is not here: the core releases the keys that a
# request of input.inject leaves down at its end, so that no key can be
# held across the clock steps. The profile of the hold (third pane step
# at 1200 ms, stop over 160 ms, no overshoot) is in the tests of
# plugins/chora-construct/src/stride.rs.
# The gate ends with the window audit: the window of the probe pane is
# not lost.
#
# Usage: tests/gate-construct-walk.sh   BIN: directory with the binaries
# shellcheck disable=SC2016,SC2034 # cell evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct
need foot
temp_config
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
step=800

help_shown() { ctl query id:construct-help | jq -e 'length > 0 and (.[0].visible != false) and (.[0].opacity // 1) > 0.9' >/dev/null 2>&1; }
until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
press() { echo "[{\"key\": {\"key\": \"$1\", \"state\": \"pressed\"}}]" | ctl inject >/dev/null || fail "press $1"; }
release() { echo "[{\"key\": {\"key\": \"$1\", \"state\": \"released\"}}]" | ctl inject >/dev/null || fail "release $1"; }
clock() { "$bin/choractl" call construct clock.step "{\"ms\": $1}" >/dev/null || fail "clock.step $1"; }
camera() { "$bin/choractl" call construct construct.list | python3 -I -c 'import json, math, sys; c = json.load(sys.stdin)["camera"]; print(round(math.hypot(c["x"], c["z"]), 1), round(c["yaw"], 3))'; }
dist() { camera | cut -d' ' -f1; }
near() { awk -v a="$1" -v b="$2" -v t="${3:-2}" 'BEGIN { d = a - b; exit !(d <= t && d >= -t) }'; }
less() { awk -v a="$1" -v b="$2" 'BEGIN { exit !(a < b) }'; }

session_start 1280x720 -- "$bin/chora-construct"
grant screen.capture input.inject
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
foot -a walkprobe sleep 600 >/dev/null 2>&1 &
run_pid $!
until_ok '"$bin/choractl" call construct construct.list | grep -q walkprobe' || fail "the probe window became no pane"
failed=0
cell() {
    if eval "$2"; then
        ok "$1"
    else
        echo "FAIL: $1"
        failed=1
    fi
}

clock 0
at=0
begin() {
    at=$((at + 10000))
    clock "$at"
    start=$(dist)
}
begin
press w
release w
sleep 0.3
clock $((at + 160))
part=$(dist)
clock $((at + 320))
whole=$(dist)
clock $((at + 1500))
later=$(dist)
cell "a tap of w is part of the way at 160 ms ($part of $step px)" 'less "$start" "$part" && less "$part" "$((${start%.*} + step))" && ! near "$part" "$start" 20'
cell "a tap of w glides one pane step in 320 ms ($whole px)" 'near "$whole" "$((${start%.*} + step))"'
cell "a tap of w goes no further ($later px)" 'near "$later" "$whole" 1'

yaw=$(camera | cut -d' ' -f2)
press q
release q
sleep 0.3
clock $((at + 3500))
turned=$(camera | cut -d' ' -f2)
cell "after the walk, q turns the camera ($yaw to $turned)" '! near "$yaw" "$turned" 0.001'
clock null
echo '[{"shortcut": {"keys": "Super+slash"}}]' | ctl inject >/dev/null || fail "inject Super+slash"
cell "after the walk, Super+/ shows the help" 'until_ok help_shown'
echo '[{"shortcut": {"keys": "Super+slash"}}]' | ctl inject >/dev/null || fail "inject Super+slash"
cell "after the walk, Super+/ hides the help" 'until_ok "! help_shown"'

cell "no window is lost: the window of the probe pane stays" 'ctl query "app_id:walkprobe kind:surface" | python3 -I -c "import json, sys; sys.exit(len(json.load(sys.stdin)) != 1)"'
[ "$failed" = 0 ] || fail "the walk in pane steps has findings"
ok "the walk goes in pane steps"
