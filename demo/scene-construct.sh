#!/bin/bash
# Starts a headless chora session with chora-construct and a scene for
# demo media: five panes with a generated picture, a Rust file with
# syntax colours, a page with text and pictures, and two more views.
# Every pane is a foot terminal that runs demo/content.py; nothing comes
# from outside the repository.
#
# Usage: demo/scene-construct.sh [THEME]
#   THEME  light or dark (default light)
#
# Before that, the camera looks to both sides once, so that every pane
# has drawn its content. When the scene stands, it prints the lines
# KEY=VALUE of tests/headless-session.sh (XDG_RUNTIME_DIR, WAYLAND_DISPLAY,
# CHORA_SOCKET) and DEMO_READY=1, then waits until a signal. choractl of
# the build talks to the session with CHORA_SOCKET. The client name
# choractl holds the grants that demo/record.sh needs.
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

mkdir -p "$config/chora/constructs"
cat >"$config/chora/chora-construct.yaml" <<YAML
theme: $theme
effects: [wobbly]
voice_hud: false
test_accept_injected_keys: true
YAML

title() {
    case $1 in
    sunset) echo Sunset ;;
    chora) echo 'chora · a compositor that gives forms a place' ;;
    load) echo 'Load' ;;
    *.rs) echo "$1 · rust" ;;
    *) echo "$1" ;;
    esac
}

pane() {
    local name=$1 at=$2 yaw=$3 width=$4 height=$5 pad=14x12
    shift 5
    [[ $1 != picture ]] || pad=0x0
    cat <<YAML
  - kind: program
    command: >-
      exec foot --app-id=$name --title='$(title "$name")' -o pad=$pad -o colors.background=1a1b26 -o colors.foreground=c0caf5
      -o 'font=DejaVu Sans Mono:size=9' -o cursor.blink=no
      -e sh -c 'sleep 1.5; python3 -I "\$0" "\$@"; exec sleep infinity'
      $(printf '%q ' "$repo/demo/content.py" "$@")
    at: [$at]
    yaw: $yaw
    size: [$width, $height]
YAML
}
{
    echo "camera: {x: 0, z: -100, yaw: 0, pitch: -0.07}"
    echo "radius: 6000"
    echo "panes:"
    pane sunset "0, 1500" 0 760 480 picture
    pane wobbly.rs "-770, 1440" -0.42 560 470 code "$repo/crates/chora-kit/src/wobbly.rs"
    pane chora "770, 1440" 0.42 560 470 page
    pane load "-1550, 700" -1.1 560 400 palette
    pane wobble.rs "1550, 700" 1.1 600 420 code "$repo/plugins/chora-construct/src/wobble.rs"
} >"$config/chora/constructs/demo.yaml"

env_file=$config/env
XDG_CONFIG_HOME=$config CHORA_BIN=$bin PATH=$bin:$PATH CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000} \
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
export CHORA_SOCKET
ctl() { timeout 20 "$bin/choractl" "$@"; }
ctl grant choractl input.inject methods.call methods.privileged screen.capture >/dev/null
for _ in $(seq 40); do
    ctl call construct construct.load '{"name": "demo"}' >/dev/null 2>&1 && break
    sleep 0.25
done
panes() { ctl call construct window.list | grep -c '"pane": "\(shown\|aside\|behind\|far\)"'; }
for _ in $(seq 80); do
    [[ $(panes) -ge 5 ]] && break
    sleep 0.25
done
[[ $(panes) -ge 5 ]] || {
    echo "$0: the panes did not start" >&2
    exit 1
}
look() { ctl call construct camera.glide "{\"x\": 0, \"z\": -100, \"yaw\": $1, \"pitch\": -0.07}" >/dev/null; }
for yaw in -1.2 1.2 0; do
    look "$yaw"
    sleep 1.5
done
cat "$env_file"
echo "DEMO_READY=1"
wait "$session"
