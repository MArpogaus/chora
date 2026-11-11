#!/bin/bash
# Starts a headless chora session with chora-construct and its cube for
# demo media: four faces of chora-xp, each with two windows that show a
# generated picture, a Rust file with syntax colours, a page or a chart.
# The pictures are text in a small font: a terminal on a face must not
# depend on sixel here.
# Every window is a foot terminal that runs demo/content.py; nothing
# comes from outside the repository.
#
# Usage: demo/scene-cube.sh [THEME]
#   THEME  light or dark (default light)
#
# The windows draw their content when all of them are placed, and the
# cube turns once around before the scene stands, so that every face
# has shown its windows. Then the script prints the lines
# KEY=VALUE of tests/headless-session.sh (XDG_RUNTIME_DIR,
# WAYLAND_DISPLAY, CHORA_SOCKET) and DEMO_READY=1, and waits until a
# signal. The client name choractl holds the grants that demo/record.sh
# needs. The core runs with --test-injected-press-is-real, so that an
# injected drag of a title bar pulls a window out of its face.
#
# Needs foot with sixel, python3 with Pillow and the font DejaVu Sans
# Mono.
#
# Environment:
#   CHORA_BIN  the directory with the binaries (default: target/debug)
#   DEMO_SIZE  the output (default 1280x720)
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${CHORA_BIN:-$repo/target/debug}
theme=${1:-light}
size=${DEMO_SIZE:-1280x720}
for tool in foot python3; do
    command -v "$tool" >/dev/null || {
        echo "$0: $tool is not installed" >&2
        exit 1
    }
done
python3 -I -c 'import PIL' 2>/dev/null || {
    echo "$0: python3 has no Pillow" >&2
    exit 1
}

config=$(mktemp -d /tmp/chora-demo.XXXXXX)
session=
cleanup() {
    trap - EXIT INT TERM HUP
    [[ -z $session ]] || kill "$session" 2>/dev/null || true
    wait 2>/dev/null || true
    rm -rf "$config"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

mkdir -p "$config/chora"
cat >"$config/chora/chora-construct.yaml" <<YAML
theme: $theme
effects: [wobbly]
voice_hud: false
test_accept_injected_keys: true
YAML

env_file=$config/env
XDG_CONFIG_HOME=$config CHORA_BIN=$bin PATH=$bin:$PATH CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000} \
    CHORA_TEST_FLAGS=--test-injected-press-is-real \
    "$repo/tests/headless-session.sh" "$size" -- "$bin/chora-construct" >"$env_file" 2>&1 &
session=$!
for _ in $(seq 120); do
    grep -q '^CHORA_SOCKET=' "$env_file" && break
    kill -0 "$session" 2>/dev/null || break
    sleep 0.25
done
grep -q '^CHORA_SOCKET=' "$env_file" || {
    cat "$env_file" >&2
    exit 1
}
CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$env_file")
WAYLAND_DISPLAY=$(sed -n 's/^WAYLAND_DISPLAY=//p' "$env_file")
XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$env_file")
export CHORA_SOCKET WAYLAND_DISPLAY XDG_RUNTIME_DIR
ctl() { timeout 20 "$bin/choractl" "$@"; }
until_ok() {
    for _ in $(seq "${2:-80}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
die() {
    echo "$0: $1" >&2
    exit 1
}
ctl grant choractl input.inject methods.call methods.privileged screen.capture >/dev/null
ctl wait id:construct-look >/dev/null || die "the construct did not start"
ctl call construct cube.open '{}' >/dev/null || die "cube.open"
until_ok 'ctl query id:xp-bar-VIRTUAL-4 | grep -q xp-bar' || die "chora-xp shows no taskbar on the last face"
echo '[{"pointer_move": {"position": [640, 360]}}]' | ctl inject >/dev/null

surface_of() {
    ctl query kind:surface | python3 -I -c '
import json, sys
print(next((n["id"] for n in json.load(sys.stdin) if n["surface"].get("app_id") == sys.argv[1] and n.get("owner") == "xp"), ""))' "$1"
}
window() {
    local name=$1 face=$2 x=$3 y=$4 width=$5 height=$6 pad=14x12 font=9 surface
    shift 6
    if [[ $1 == mosaic || $1 == bars ]]; then
        pad=0x0
        font=3
    fi
    # shellcheck disable=SC2016 # the shell of the terminal expands it
    LANG=C.UTF-8 DEMO_PLACED=$config/placed foot --app-id="$name" --title="$name" --window-size-pixels="${width}x${height}" \
        -o pad=$pad -o colors.background=1a1b26 -o colors.foreground=c0caf5 \
        -o "font=DejaVu Sans Mono:size=$font" -o cursor.blink=no \
        -e sh -c 'until [ -e "$DEMO_PLACED" ]; do sleep 0.2; done; python3 -I "$0" "$@"; exec sleep infinity' \
        "$repo/demo/content.py" "$@" \
        >/dev/null 2>&1 &
    until_ok "test -n \"\$(surface_of $name)\"" || die "the window $name did not open"
    surface=$(surface_of "$name")
    ctl call xp window.move \
        "{\"surface\": \"$surface\", \"output\": \"VIRTUAL-$face\", \"at\": [$((x + width / 2)), $((y - 15))]}" \
        >/dev/null || die "window.move $name"
}
window sunset 1 70 64 560 375 mosaic
window chora 1 690 262 520 400 notes
window wobbly.rs 2 110 90 560 420 code "$repo/crates/chora-kit/src/wobbly.rs"
window load 2 700 330 470 290 palette
window cube.rs 3 600 80 560 420 code "$repo/plugins/chora-construct/src/cube.rs"
window chart 3 120 300 518 330 bars
window wobble.rs 4 100 70 540 380 code "$repo/plugins/chora-construct/src/wobble.rs"
window dusk 4 560 240 602 405 mosaic 3

front_is() { ctl query "id:construct-cube-face-$1" | grep -q '"forward": true'; }
sleep 1
touch "$config/placed"
sleep 4
for face in 1 2 3 0; do
    ctl call construct cube.turn '{"by": 1}' >/dev/null
    until_ok "front_is $face" 40 || die "the cube did not turn to face $face"
    sleep 1.5
done
cat "$env_file"
echo "DEMO_READY=1"
wait "$session"
