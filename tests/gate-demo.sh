#!/bin/bash
# The demo scene of demo/scene-construct.sh stands and shows colourful
# content: the construct has five panes, and the left, the middle and
# the right third of the output each show strong colours, so that no
# pane reads as flat grey. Then demo/record.sh makes its clips, when
# ffmpeg is installed: each clip is a GIF of at most 10 MB and an MP4
# with H.264, 1280 by 720.
#
# The cube scene of demo/scene-cube.sh stands too: each of the four
# faces has two windows of chora-xp. A step into the front face takes
# the clock of the animations: with a stepped clock far ahead of the
# monotonic clock, the face is not at 1:1 one frame after view.step_in.
# Then demo/record.sh makes cube and pull as clips and cube-float.png,
# a still of 1280 by 720.
#
# Usage: tests/gate-demo.sh
#   BIN  the directory with the binaries (default: target/debug)
#   DEMO_CLIPS=0  skip the clips
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
python3 -I -c 'import PIL' 2>/dev/null || skip "python3 has no Pillow"
test -x "$repo/demo/scene-construct.sh" || fail "no demo/scene-construct.sh"

work=$(mktemp -d /tmp/gate-demo.XXXXXX)
tmp_dirs+=("$work")
CHORA_BIN=$bin "$repo/demo/scene-construct.sh" >"$work/env" 2>&1 &
pids+=($!)
trap gate_cleanup EXIT
for _ in $(seq 120); do
    grep -q '^DEMO_READY=1' "$work/env" && break
    sleep 0.5
done
grep -q '^DEMO_READY=1' "$work/env" || fail "the scene did not stand: $(tail -n 3 "$work/env")"
CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$work/env")
export CHORA_SOCKET

panes=$(timeout 20 "$bin/choractl" call construct construct.list |
    python3 -I -c 'import json, sys; print(len(json.load(sys.stdin)["panes"]))')
[ "$panes" = 5 ] || fail "the scene has $panes panes, not 5"
ok "the scene has five panes"

timeout 20 "$bin/choractl" capture "$work/scene.png" >/dev/null || fail "no capture"
colourful=$(python3 -I - "$work/scene.png" <<'PY'
import sys
from PIL import Image

image = Image.open(sys.argv[1]).convert("RGB")
width, height = image.size
shares = []
for third in range(3):
    part = image.crop((third * width // 3, 0, (third + 1) * width // 3, height))
    data = part.tobytes()
    pixels = [data[i : i + 3] for i in range(0, len(data), 3)]
    strong = sum(1 for pixel in pixels if max(pixel) - min(pixel) > 60)
    shares.append(strong / len(pixels))
print(" ".join(f"{share:.4f}" for share in shares))
sys.exit(0 if min(shares) > 0.002 else 1)
PY
) || fail "a third of the scene shows no strong colour: $colourful"
ok "each third of the scene shows strong colours: $colourful"

clips=1
if [ "${DEMO_CLIPS:-1}" = 0 ] || ! command -v ffmpeg >/dev/null; then
    clips=0
fi
check_clip() {
    local gif=$work/clips/$1.gif mp4=$work/clips/$1.mp4 probe
    test -s "$gif" && test -s "$mp4" || fail "no clip $1"
    [ "$(stat -c %s "$gif")" -le 10000000 ] || fail "$gif has more than 10 MB"
    probe=$(ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,width,height -of csv=p=0 "$mp4")
    [ "$probe" = "h264,1280,720" ] || fail "$mp4 is $probe, not h264,1280,720"
    ok "the clip $1: a GIF of $(stat -c %s "$gif") bytes and an MP4 ($probe)"
}
construct_clips() {
    test -x "$repo/demo/record.sh" || fail "no demo/record.sh"
    CHORA_BIN=$bin DEMO_SOCKET=$CHORA_SOCKET "$repo/demo/record.sh" "$work/clips" walk swing >"$work/record.log" 2>&1 ||
        fail "demo/record.sh: $(tail -n 5 "$work/record.log")"
    check_clip construct-walk
    check_clip construct-swing
}
[ "$clips" = 0 ] || construct_clips
kill "${pids[@]}" 2>/dev/null
wait 2>/dev/null
pids=()

test -x "$repo/demo/scene-cube.sh" || fail "no demo/scene-cube.sh"
CHORA_BIN=$bin "$repo/demo/scene-cube.sh" >"$work/cube.env" 2>&1 &
pids+=($!)
for _ in $(seq 240); do
    grep -q '^DEMO_READY=1' "$work/cube.env" && break
    sleep 0.5
done
grep -q '^DEMO_READY=1' "$work/cube.env" || fail "the cube scene did not stand: $(tail -n 3 "$work/cube.env")"
CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$work/cube.env")
export CHORA_SOCKET
ctl() { timeout 20 "$bin/choractl" "$@"; }
faces=$(ctl query kind:surface | python3 -I -c '
import collections, json, sys
on = collections.Counter(
    output for n in json.load(sys.stdin) if n.get("owner") == "xp" for output in n["rects"] if output.startswith("VIRTUAL")
)
print(" ".join(f"{output}:{count}" for output, count in sorted(on.items())))')
[ "$faces" = "VIRTUAL-1:2 VIRTUAL-2:2 VIRTUAL-3:2 VIRTUAL-4:2" ] || fail "the faces have the windows $faces, not two each"
ok "the cube scene has two windows on each of its four faces"

width() { ctl query id:construct-cube-face-0 | python3 -I -c 'import json, sys; print(round(json.load(sys.stdin)[0]["rects"]["HEADLESS-1"][2]))'; }
step() { ctl call construct clock.step "{\"ms\": $1}" >/dev/null || fail "clock.step $1"; }
ctl call construct view.step_back >/dev/null || fail "view.step_back"
for _ in $(seq 40); do
    [ "$(width)" -lt 1000 ] && break
    sleep 0.25
done
[ "$(width)" -lt 1000 ] || fail "no step back: the face is $(width) wide"
step 0
step 100000
ctl call construct view.step_in >/dev/null || fail "view.step_in"
step 100033
between=$(width)
step 101000
ended=$(width)
step null
[ "$between" -lt 1280 ] || fail "one frame after view.step_in the face is $between wide: the step does not take the stepped clock"
[ "$ended" = 1280 ] || fail "a second after view.step_in the face is $ended wide, not 1280"
ok "a step in takes the clock of the animations: the face is $between wide one frame after it, and $ended a second after"

if [ "$clips" = 0 ]; then
    ok "the scenes stand; the clips are skipped"
    exit 0
fi
CHORA_BIN=$bin DEMO_SOCKET=$CHORA_SOCKET "$repo/demo/record.sh" "$work/clips" cube float pull >"$work/record-cube.log" 2>&1 ||
    fail "demo/record.sh: $(tail -n 5 "$work/record-cube.log")"
check_clip cube
check_clip pull
still=$(python3 -I -c 'import sys; from PIL import Image; print("%dx%d" % Image.open(sys.argv[1]).size)' "$work/clips/cube-float.png") ||
    fail "no still cube-float.png"
[ "$still" = 1280x720 ] || fail "cube-float.png is $still, not 1280x720"
ok "the still cube-float.png ($still)"
ok "the demo scenes and their clips"
