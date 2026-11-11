#!/bin/bash
# Records the demo clips of chora-construct: a walk through the
# construct, and a pane that is grabbed and swung so that it wobbles in
# 3D. Each clip becomes a GIF (1280 by 720, at most 10 MB, with its own
# palette) and an MP4 (H.264).
#
# The frames come from `choractl record`, which steps the scene clock by
# 1/30 s per frame, so the motion is the same on any renderer. The paths
# of the camera and of the pointer come from demo/paths.py.
#
# Usage: demo/record.sh OUT_DIR [CLIP...]
#   OUT_DIR  gets construct-CLIP.gif and construct-CLIP.mp4
#   CLIP     walk or swing (default: both)
#
# Environment:
#   CHORA_BIN    the directory with the binaries (default: target/debug)
#   DEMO_SOCKET  the CHORA_SOCKET of a running demo/scene-construct.sh;
#                without it, the script starts the scene and ends it
#   DEMO_THEME   light or dark (default light), for a scene that the script
#                starts
#   DEMO_FRAMES  a directory that keeps the PNG frames
#
# Needs ffmpeg with libx264, and what demo/scene-construct.sh needs.
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${CHORA_BIN:-$repo/target/debug}
[[ $# -ge 1 ]] || {
    sed -n '2,/^set /p' "$0" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
}
out=$1
shift
clips=("$@")
[[ ${#clips[@]} -gt 0 ]] || clips=(walk swing)
command -v ffmpeg >/dev/null || {
    echo "$0: ffmpeg is not installed" >&2
    exit 1
}
gif_limit=10000000
fps=30

work=$(mktemp -d /tmp/chora-record.XXXXXX)
scene=
cleanup() {
    trap - EXIT INT TERM HUP
    [[ -z $scene ]] || kill "$scene" 2>/dev/null || true
    wait 2>/dev/null || true
    rm -rf "$work"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

if [[ -n ${DEMO_SOCKET:-} ]]; then
    export CHORA_SOCKET=$DEMO_SOCKET
else
    CHORA_BIN=$bin "$repo/demo/scene-construct.sh" "${DEMO_THEME:-light}" >"$work/env" 2>&1 &
    scene=$!
    for _ in $(seq 240); do
        grep -q '^DEMO_READY=1' "$work/env" && break
        kill -0 "$scene" 2>/dev/null || break
        sleep 0.5
    done
    grep -q '^DEMO_READY=1' "$work/env" || {
        cat "$work/env" >&2
        exit 1
    }
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$work/env")
    export CHORA_SOCKET
fi

mkdir -p "$out"
frames_root=${DEMO_FRAMES:-$work/frames}
for clip in "${clips[@]}"; do
    frames=$frames_root/$clip
    rm -rf "$frames"
    mkdir -p "$frames"
    python3 -I "$repo/demo/paths.py" "$clip" >"$work/$clip.cues"
    count=$(python3 -I "$repo/demo/paths.py" "$clip" frames)
    "$bin/choractl" record --fps "$fps" --frames "$count" --out "$frames" --script "$work/$clip.cues" \
        --pointer arrow >/dev/null
    quiet=(-hide_banner -loglevel error -y)
    ffmpeg "${quiet[@]}" -framerate "$fps" -i "$frames/frame-%05d.png" \
        -c:v libx264 -preset slow -crf 17 -pix_fmt yuv420p -movflags +faststart "$out/construct-$clip.mp4"
    gif=$out/construct-$clip.gif
    for gif_fps in 30 25 20 15 12 10; do
        filter="fps=$gif_fps,split[a][b];[a]palettegen=max_colors=255:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=4:diff_mode=rectangle"
        ffmpeg "${quiet[@]}" -framerate "$fps" -i "$frames/frame-%05d.png" -filter_complex "$filter" -loop 0 "$gif"
        [[ $(stat -c %s "$gif") -gt $gif_limit ]] || break
    done
    [[ $(stat -c %s "$gif") -le $gif_limit ]] || {
        echo "$0: $gif has more than $gif_limit bytes at 10 frames a second" >&2
        exit 1
    }
    echo "$clip: $count frames, GIF at $gif_fps fps with $(stat -c %s "$gif") bytes, $out/construct-$clip.mp4"
done
