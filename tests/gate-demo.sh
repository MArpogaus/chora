#!/bin/bash
# The demo scene of demo/scene-construct.sh stands and shows colourful
# content: the construct has five panes, and the left, the middle and
# the right third of the output each show strong colours, so that no
# pane reads as flat grey. Then demo/record.sh makes its clips, when
# ffmpeg is installed: each clip is a GIF of at most 10 MB and an MP4
# with H.264, 1280 by 720.
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

if [ "${DEMO_CLIPS:-1}" = 0 ] || ! command -v ffmpeg >/dev/null; then
    ok "the scene stands; the clips are skipped"
    exit 0
fi
test -x "$repo/demo/record.sh" || fail "no demo/record.sh"
CHORA_BIN=$bin DEMO_SOCKET=$CHORA_SOCKET "$repo/demo/record.sh" "$work/clips" >"$work/record.log" 2>&1 ||
    fail "demo/record.sh: $(tail -n 5 "$work/record.log")"
for clip in walk swing; do
    gif=$work/clips/construct-$clip.gif
    mp4=$work/clips/construct-$clip.mp4
    test -s "$gif" && test -s "$mp4" || fail "no clip $clip"
    [ "$(stat -c %s "$gif")" -le 10000000 ] || fail "$gif has more than 10 MB"
    probe=$(ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,width,height -of csv=p=0 "$mp4")
    [ "$probe" = "h264,1280,720" ] || fail "$mp4 is $probe, not h264,1280,720"
    ok "the clip $clip: a GIF of $(stat -c %s "$gif") bytes and an MP4 ($probe)"
done
ok "the demo scene and its clips"
