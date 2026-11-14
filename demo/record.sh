#!/bin/bash
# Records the demo media of chora-construct. In the scene of
# demo/scene-construct.sh: a walk through the construct, and a pane that
# is grabbed and swung so that it wobbles in 3D. In the scene of
# demo/scene-cube.sh: the cube stood back, turned once around and
# stepped into; a still of the cube in a turn; and a window that is
# pulled off a face and dropped as a pane. Each clip becomes a GIF (1280
# by 720, at most 10 MB, with its own palette) and an MP4 (H.264); the
# still becomes a PNG.
#
# The frames come from `choractl record`, which steps the scene clock by
# 1/30 s per frame, so the motion is the same on any renderer. The paths
# of the camera and of the pointer come from demo/paths.py.
#
# Usage: demo/record.sh OUT_DIR [CLIP...]
#   OUT_DIR  gets construct-walk and construct-swing, cube and pull, each
#            as .gif and .mp4, and cube-float.png
#   CLIP     walk, swing, cube, float or pull (default: all). The clips
#            of one scene run in this order, as pull changes its scene.
#
# Environment:
#   CHORA_BIN    the directory with the binaries (default: target/debug)
#   DEMO_SOCKET  the CHORA_SOCKET of a running scene, for clips of that
#                one scene; without it, the script starts each scene and
#                ends it
#   DEMO_THEME   light or dark (default light), for a scene that the script
#                starts
#   DEMO_FRAMES  a directory that keeps the PNG frames
#
# Needs ffmpeg with libx264, and what the scenes need.
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
[[ ${#clips[@]} -gt 0 ]] || clips=(walk swing cube float pull)
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

scene_of() {
    case $1 in
    walk | swing) echo construct ;;
    cube | float | pull) echo cube ;;
    *)
        echo "$0: no clip $1" >&2
        exit 2
        ;;
    esac
}
for clip in "${clips[@]}"; do scene_of "$clip" >/dev/null; done

start_scene() {
    CHORA_BIN=$bin "$repo/demo/scene-$1.sh" "${DEMO_THEME:-light}" >"$work/env" 2>&1 &
    scene=$!
    for _ in $(seq 360); do
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
}
end_scene() {
    kill "$scene" 2>/dev/null || true
    wait "$scene" 2>/dev/null || true
    scene=
}
ctl() { timeout 20 "$bin/choractl" "$@"; }

record() {
    local clip=$1 frames=$frames_root/$1 count name=$1 gif gif_fps filter
    local quiet=(-hide_banner -loglevel error -y)
    rm -rf "$frames"
    mkdir -p "$frames"
    python3 -I "$repo/demo/paths.py" "$clip" >"$work/$clip.cues"
    count=$(python3 -I "$repo/demo/paths.py" "$clip" frames)
    "$bin/choractl" record --fps "$fps" --frames "$count" --out "$frames" --script "$work/$clip.cues" \
        --pointer arrow >/dev/null
    if [[ $clip == float ]]; then
        cp "$(printf '%s/frame-%05d.png' "$frames" "$count")" "$out/cube-float.png"
        ctl call construct view.step_in >/dev/null
        sleep 4
        ctl call construct cube.turn '{"by": -1}' >/dev/null
        sleep 2
        echo "float: $out/cube-float.png"
        return
    fi
    [[ $(scene_of "$clip") != construct ]] || name=construct-$clip
    ffmpeg "${quiet[@]}" -framerate "$fps" -i "$frames/frame-%05d.png" \
        -c:v libx264 -preset slow -crf 17 -pix_fmt yuv420p -movflags +faststart "$out/$name.mp4"
    gif=$out/$name.gif
    for gif_fps in 30 25 20 15 12 10; do
        filter="fps=$gif_fps,split[a][b];[a]palettegen=max_colors=255:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=4:diff_mode=rectangle"
        ffmpeg "${quiet[@]}" -framerate "$fps" -i "$frames/frame-%05d.png" -filter_complex "$filter" -loop 0 "$gif"
        [[ $(stat -c %s "$gif") -gt $gif_limit ]] || break
    done
    [[ $(stat -c %s "$gif") -le $gif_limit ]] || {
        echo "$0: $gif has more than $gif_limit bytes at 10 frames a second" >&2
        exit 1
    }
    echo "$clip: $count frames, GIF at $gif_fps fps with $(stat -c %s "$gif") bytes, $out/$name.mp4"
}

mkdir -p "$out"
frames_root=${DEMO_FRAMES:-$work/frames}
for kind in construct cube; do
    wanted=()
    for clip in walk swing cube float pull; do
        [[ " ${clips[*]} " == *" $clip "* && $(scene_of "$clip") == "$kind" ]] && wanted+=("$clip")
    done
    [[ ${#wanted[@]} -gt 0 ]] || continue
    if [[ -n ${DEMO_SOCKET:-} ]]; then
        export CHORA_SOCKET=$DEMO_SOCKET
    else
        start_scene "$kind"
    fi
    for clip in "${wanted[@]}"; do record "$clip"; done
    [[ -z $scene ]] || end_scene
done
