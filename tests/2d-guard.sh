#!/bin/bash
# Proves that 2D rendering did not change between a base commit and the
# working tree: both builds render the same scenes on the headless backend
# and the captures must match pixel for pixel.
#
# Usage: tests/2d-guard.sh BASE_SHA
#   BIN  directory with chora, choractl and chora-probe of the current
#        build (default: target/debug of the checkout)
#
# The base is built once in the worktree /home/user/wt/base2d, with the
# target directory /home/user/target-base; its captures stay in
# /home/user/guard-cache/BASE_SHA, so later runs skip the base.
# Prints one line per scene and PASS or FAIL; exits 1 on FAIL.
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
[[ $# -eq 1 ]] || {
    echo "usage: $0 BASE_SHA" >&2
    exit 2
}
base=$(git -C "$repo" rev-parse --verify "$1^{commit}")
cur_bin=${BIN:-$repo/target/debug}
wt=/home/user/wt/base2d
base_target=/home/user/target-base
cache=/home/user/guard-cache/$base
scenes=(rect text rounded shader layer window stacked)
size=640x400

# software EGL for the headless renderer, where that copy exists
if [[ -d $HOME/.local/opt/mesa-egl ]]; then
    export __EGL_VENDOR_LIBRARY_DIRS=${__EGL_VENDOR_LIBRARY_DIRS:-$HOME/.local/opt/mesa-egl/usr/share/glvnd/egl_vendor.d}
    export LD_LIBRARY_PATH=$HOME/.local/opt/mesa-egl/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
fi

work=$(mktemp -d /tmp/2d-guard.XXXXXX)
session=
clients=()
# shellcheck disable=SC2329 # runs as the EXIT trap
cleanup() {
    trap - EXIT INT TERM HUP
    stop_clients
    # the TERM trap of headless-session.sh ends the core and its group
    [[ -z $session ]] || kill "$session" 2>/dev/null || true
    wait 2>/dev/null || true
    rm -rf "$work"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

die() {
    echo "2d-guard: $*" >&2
    exit 2
}

# The fixed shader of the shader scene: a gradient.
cat >"$work/grad.frag" <<'GLSL'
precision mediump float;
varying vec2 v_uv;
void main() {
    gl_FragColor = vec4(v_uv.x, v_uv.y, 0.5, 1.0);
}
GLSL

stop_clients() {
    local p
    for p in "${clients[@]}"; do
        kill "$p" 2>/dev/null || true
    done
    clients=()
}

# apply JSON: one transaction of the agent
apply() { echo "$1" | timeout 10 "$bin/choractl" --name agent apply - >/dev/null; }

# A scene is a function that fills the output; "reset" removes it.
rect_nodes() {
    echo '{"create": {"id": "g-rect", "kind": "rect", "parent": "root", "props": {"size": [200, 120], "color": [0.9, 0.3, 0.1, 1], "transform": {"translate": [40, 40]}}}}'
}
text_nodes() {
    echo '{"create": {"id": "g-text", "kind": "text", "parent": "root", "props": {"text": "chora 2d guard", "font_size": 32, "color": [1, 1, 1, 1], "transform": {"translate": [60, 200]}}}}'
}
rounded_nodes() {
    echo '{"create": {"id": "g-round", "kind": "rect", "parent": "root", "props": {"size": [220, 140], "color": [0.2, 0.5, 0.9, 0.8], "radius": 28, "border": {"width": 4, "color": [1, 1, 1, 1]}, "transform": {"translate": [300, 30]}}}}'
}
shader_nodes() {
    echo '{"create": {"id": "g-shader", "kind": "shader", "parent": "root", "props": {"size": [180, 120], "shader": "'"$work"'/grad.frag", "transform": {"translate": [380, 240]}}}}'
}

# nodes NAME...: create the nodes of the named scenes in one transaction
nodes() {
    local list=() n
    for n in "$@"; do
        list+=("$("${n}_nodes")")
    done
    local IFS=,
    apply "[${list[*]}]"
}

# start_layer: a layer surface, the solid background of swaybg
start_layer() {
    command -v swaybg >/dev/null || die "swaybg is not installed"
    swaybg -c '#336699' >/dev/null 2>&1 &
    clients+=($!)
    sleep 1.5
}

# start_window: a probe window, placed at 100,100
start_window() {
    "$bin/chora-probe" window 200 40 160 >/dev/null 2>&1 &
    clients+=($!)
    local id
    id=$(timeout 10 "$bin/choractl" --name agent wait 'app_id:chora-probe kind:surface' |
        sed -n 's/.*"id": *"\(s[0-9]*\)".*/\1/p' | head -n 1)
    [[ -n $id ]] || die "the probe window did not appear"
    apply '[{"attach": {"surface": "'"$id"'", "parent": "root"}}, {"update": {"id": "'"$id"'", "props": {"transform": {"translate": [100, 100]}}}}]'
    sleep 1
}

# reset: remove all nodes and clients of the last scene
reset() {
    stop_clients
    local id
    for id in g-rect g-text g-round g-shader; do
        apply '[{"remove": {"id": "'"$id"'"}}]' 2>/dev/null || true
    done
    # wait for the windows and layer surfaces to go
    timeout 10 "$bin/choractl" --name agent wait 'kind:surface' --gone >/dev/null 2>&1 || true
    sleep 0.5
}

# shot NAME: capture the output
shot() {
    timeout 20 "$bin/choractl" --name agent capture "$out/$1.png" >/dev/null ||
        die "no capture of $1"
}

# render BIN OUTDIR: all scenes with one build, in one headless session
render() {
    bin=$1
    out=$2
    mkdir -p "$out"
    [[ -x $bin/chora && -x $bin/choractl && -x $bin/chora-probe ]] ||
        die "chora, choractl or chora-probe is missing in $bin"
    local env_file=$work/env
    CHORA_BIN=$bin "$repo/tests/headless-session.sh" "$size" >"$env_file" 2>&1 &
    session=$!
    local _
    for _ in $(seq 60); do
        grep -q CHORA_SOCKET "$env_file" && break
        sleep 0.5
    done
    grep -q CHORA_SOCKET "$env_file" || die "no core: $(tail -n 3 "$env_file")"
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$env_file")
    WAYLAND_DISPLAY=$(sed -n 's/^WAYLAND_DISPLAY=//p' "$env_file")
    XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$env_file")
    export CHORA_SOCKET WAYLAND_DISPLAY XDG_RUNTIME_DIR
    "$bin/choractl" grant agent screen.capture >/dev/null || die "the grant"

    nodes rect;    shot rect;    reset
    nodes text;    shot text;    reset
    nodes rounded; shot rounded; reset
    nodes shader;  shot shader;  reset
    start_layer;   shot layer;   reset
    start_window;  shot window;  reset
    start_layer
    start_window
    nodes rect text rounded shader
    shot stacked
    reset

    kill "$session" 2>/dev/null || true
    wait "$session" 2>/dev/null || true
    session=
}

# the base: build and capture once
have_base=1
for s in "${scenes[@]}"; do
    [[ -s $cache/$s.png ]] || have_base=0
done
if ((have_base == 0)); then
    avail=$(df -BG --output=avail /home/user | tail -n 1 | tr -dc 0-9)
    ((avail >= 8)) || die "only ${avail} GB free in /home/user; need 8"
    if [[ -d $wt ]]; then
        git -C "$wt" checkout -q --detach "$base"
    else
        git -C "$repo" worktree add -q --detach "$wt" "$base"
    fi
    (
        cd "$wt"
        CARGO_TARGET_DIR=$base_target CARGO_PROFILE_DEV_DEBUG=0 \
            cargo build -p chora -p choractl -p chora-probe
    ) >&2
    rm -rf "$cache.tmp"
    render "$base_target/debug" "$cache.tmp"
    rm -rf "$cache"
    mv "$cache.tmp" "$cache"
fi

# the current build
render "$cur_bin" "$work/cur"

verdict=PASS
for s in "${scenes[@]}"; do
    res=
    rc=0
    if [[ -s $cache/$s.png && -s $work/cur/$s.png ]]; then
        res=$(python3 -I "$repo/tests/png-same.py" "$cache/$s.png" "$work/cur/$s.png" 2>&1) || rc=$?
    else
        res="capture missing"
        rc=1
    fi
    if ((rc == 0)) && [[ $res == "0 0" ]]; then
        echo "ok   $s: identical"
    else
        echo "FAIL $s: ${res//$'\n'/ }"
        verdict=FAIL
    fi
done
echo "$verdict"
[[ $verdict == PASS ]] || exit 1
