#!/bin/bash
# Gaps and corners of tiling (tiling-v2, item 4): the outer gap is 12 px,
# the inner gap 10 px, the corners have the radius 10; the max layout and a
# fullscreen window have no gap and no radius. A headless capture of red
# windows is measured pixel by pixel.
#
# Usage: tests/gate-tiling-gaps.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling chora-probe
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture
shot=$XDG_CONFIG_HOME/gaps.png

window() {
    pid=$(ctl run "$bin/chora-probe" window 255 0 0 |
        sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p')
    [ -n "$pid" ] || fail "the run of the window"
    run_pid "$pid"
}
red() { [ "$(pixel "$shot" "$1" "$2" | cut -d' ' -f1-3)" = "255 0 0" ]; }
snap() {
    sleep 2
    ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
}
window
window
for _ in $(seq 60); do
    [ "$(ctl query 'app_id:chora-probe kind:surface' | grep -c '"owner": *"tiling"')" = 2 ] && break
    sleep 0.5
done
snap
red 12 400 || fail "the window does not start at x 12"
red 11 400 && fail "the window starts before x 12"
red 300 12 || fail "the windows do not start at y 12"
red 300 11 && fail "the windows start before y 12"
red 1267 400 || fail "the stack does not end at x 1268"
red 1268 400 && fail "the stack ends after x 1268"
red 300 787 || fail "the windows do not end at y 788"
red 300 788 && fail "the windows end after y 788"
ok "the outer gap is 12"
master_right=$(for x in $(seq 560 720); do red "$x" 400 || { echo "$x"; break; }; done)
stack_left=$(for x in $(seq "$master_right" 720); do red "$x" 400 && { echo "$x"; break; }; done)
[ -n "$master_right" ] && [ -n "$stack_left" ] || fail "no gap between the tiles"
[ $((stack_left - master_right)) = 10 ] || fail "the inner gap is $((stack_left - master_right))"
ok "the inner gap is 10"
red 12 12 && fail "the corner is square"
red 13 13 && fail "the corner at 13,13 is inside the radius 10"
red 22 12 || fail "the top edge is missing at x 22"
red 12 22 || fail "the left edge is missing at y 22"
red 16 16 || fail "the corner is cut too deep at 16,16"
ok "the corners have the radius 10"

ctl call tiling set_layout '{"layout": "max"}' >/dev/null || fail "set_layout max"
snap
red 0 0 || fail "max has a gap or a corner at 0,0"
red 1279 799 || fail "max has a gap or a corner at 1279,799"
ok "max has no gap and no corner"
