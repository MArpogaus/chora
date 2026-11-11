#!/bin/bash
# Gaps and corners of tiling (tiling-v2, item 4): the outer gap is 12 px,
# the inner gap 10 px, both measured from the outer edge of the 2 px border,
# which is what the eye sees; the window itself starts 2 px further in. The
# corners have the radius 10; the max layout and a fullscreen window have no
# gap, no border and no radius. A headless capture of red windows is
# measured pixel by pixel; the wallpaper of a headless session is black.
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
lit() { read -r r g b _ < <(pixel "$shot" "$1" "$2"); [ $((r + g + b)) -gt 60 ]; }
lit 12 400 || fail "the border does not start at x 12"
lit 11 400 && fail "the border starts before x 12"
lit 300 12 || fail "the border does not start at y 12"
lit 300 11 && fail "the border starts before y 12"
lit 1267 400 || fail "the border does not end at x 1268"
lit 1268 400 && fail "the border ends after x 1268"
lit 300 787 || fail "the border does not end at y 788"
lit 300 788 && fail "the border ends after y 788"
red 14 400 || fail "the window does not start 2 px inside the border"
ok "the outer gap is 12, from the border"
master_right=$(for x in $(seq 560 720); do lit "$x" 400 || { echo "$x"; break; }; done)
stack_left=$(for x in $(seq "$master_right" 720); do lit "$x" 400 && { echo "$x"; break; }; done)
[ -n "$master_right" ] && [ -n "$stack_left" ] || fail "no gap between the tiles"
[ $((stack_left - master_right)) = 10 ] || fail "the inner gap is $((stack_left - master_right))"
ok "the inner gap is 10, from border to border"
red 14 14 && fail "the corner is square"
red 15 15 && fail "the corner at 15,15 is inside the radius 10"
red 24 14 || fail "the top edge is missing at x 24"
red 14 24 || fail "the left edge is missing at y 24"
red 18 18 || fail "the corner is cut too deep at 18,18"
ok "the corners have the radius 10"

ctl call tiling set_layout '{"layout": "max"}' >/dev/null || fail "set_layout max"
snap
red 0 0 || fail "max has a gap or a corner at 0,0"
red 1279 799 || fail "max has a gap or a corner at 1279,799"
ok "max has no gap and no corner"
