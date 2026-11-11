#!/bin/bash
# Captures the marble floor of chora-construct: three probe panes in a
# headless session, once standing and once while walking, with the
# render.stats of the still screen before and after a second of rest.
# The pictures come from `choractl record --pointer none`, so they show
# no cursor and no mark of injected input; the walk is three presses of
# the forward key at the stepped clock of the recording, and the picture
# is the twelfth frame.
#
# Usage: demo/marble-shots.sh THEME SPEC OUT_DIR TAG
#   THEME    light or dark
#   SPEC     the output, for example 1280x800 or 2560x1600@2
#   OUT_DIR  gets TAG-standing.png, TAG-walking.png,
#            TAG-stats-before.json and TAG-stats-still.json
#   TAG      the start of the file names, for example light-1x
#
# Environment:
#   BIN  the directory with the binaries (default: target/debug)
set -u
repo=$(cd "$(dirname "$0")/.." && pwd)
[ $# -eq 4 ] || {
    sed -n '2,/^set /p' "$0" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
}
theme=$1 spec=$2 out=$3 tag=$4
. "$repo/tests/gate-lib.sh"
need_bins chora choractl chora-construct chora-probe
temp_config
work=$XDG_CONFIG_HOME
mkdir -p "$work/chora/constructs" "$out"
export CHORA_TEST_ACCEPT_INJECTED_KEYS=1
printf 'test_accept_injected_keys: true\ntheme: %s\n' "$theme" >"$work/chora/chora-construct.yaml"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-2000000}
{
    echo "camera: {x: 0, z: 0, yaw: 0, pitch: 0}"
    echo "radius: 6000"
    echo "panes:"
    for pane in "-700,2000:0.5:640,400:40 90 200" "650,2300:-0.4:560,360:230 120 40" "0,3400:0:900,560:30 160 120"; do
        IFS=: read -r at yaw size rgb <<<"$pane"
        echo "  - kind: program"
        echo "    command: exec chora-probe window $rgb"
        echo "    at: [$at]"
        echo "    yaw: $yaw"
        echo "    size: [$size]"
    done
} >"$work/chora/constructs/marble.yaml"
session_start "$spec" -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space"
ctl call construct construct.load '{"name": "marble"}' >/dev/null || fail "construct.load"
sleep 6
frames=$(mktemp -d)
still() {
    rm -f "$frames"/*.png
    "$bin/choractl" record --name agent --frames "$1" --out "$frames" --pointer none ${2:+--script "$2"} >/dev/null ||
        fail "choractl record"
    cp "$(printf '%s/frame-%05d.png' "$frames" "$1")" "$3"
}
still 1 "" "$out/$tag-standing.png"
ctl render-stats >"$out/$tag-stats-before.json"
sleep 1
ctl render-stats >"$out/$tag-stats-still.json"
walk=$frames/walk.jsonl
for frame in 1 2 3; do
    echo "{\"frame\": $frame, \"method\": \"input.inject\", \"params\": {\"events\": [{\"shortcut\": {\"keys\": \"w\"}}]}}"
done >"$walk"
still 12 "$walk" "$out/$tag-walking.png"
rm -rf "$frames"
echo "marble-shots: $tag in $out"
