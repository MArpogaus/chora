#!/bin/bash
# Gate of window managers in construct panes: a nested chora that
# `choractl run` starts from inside the construct flavor opens its own
# winit window, and that window becomes a pane.
#
# For chora-tiling and for chora-xp:
# 1. `choractl run -- chora -- WM` with the CHORA_SOCKET and the
#    WAYLAND_DISPLAY of the outer session in its environment: the nested
#    chora opens its own Wayland and IPC socket, and the outer one still
#    answers.
# 2. Its window becomes a pane; a fly-in gives it the keyboard.
# 3. A foot in the nested session gets the typed text.
#
# Usage: tests/gate-construct-nested.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the captures as construct-nested-*.png
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-tiling chora-xp
need foot
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work}
mkdir -p "$shots"
# chora-construct starts its programs with `choractl run`, and the nested
# chora finds its window manager in PATH.
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
foot=$(command -v foot)

call() { ctl call construct "$@"; }
panes() { call construct.list | python3 -I -c 'import json, sys; print(" ".join(sorted(p["name"] for p in json.load(sys.stdin)["panes"])))'; }
until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}

session_start 1280x800 -- "$bin/chora-construct"
grant screen.capture input.inject
XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$GATE_ENV")
export XDG_RUNTIME_DIR
outer=$CHORA_SOCKET
ctl wait --timeout 10 id:construct-look >/dev/null || fail "no space"

# nested NAME N WM...: starts `chora -- WM...` as wayland-N, flies into
# its pane and types into a foot inside it.
nested() {
    local name=$1 n=$2 before pane sock
    shift 2
    sock=$XDG_RUNTIME_DIR/chora/wayland-$n.sock
    before=$(panes)
    ctl run -- chora -- "$@" >"$work/run-$name.json" || fail "$name: choractl run"
    run_pid "$(python3 -I -c 'import json, sys; print(json.load(sys.stdin)["pid"])' <"$work/run-$name.json")"
    until_ok 'test -S "$sock"' 80 || fail "$name: no IPC socket wayland-$n"
    test -S "$XDG_RUNTIME_DIR/wayland-$n" || fail "$name: no Wayland socket wayland-$n"
    CHORA_SOCKET=$outer ctl query id:construct-look >/dev/null || fail "$name: the outer session does not answer"
    until_ok 'test "$(panes)" != "$before"' 80 || fail "$name: the nested window became no pane"
    pane=$(comm -13 <(tr ' ' '\n' <<<"$before") <(panes | tr ' ' '\n'))
    ok "$name: the nested chora opened wayland-$n and its own IPC socket; its window is the pane $pane"

    WAYLAND_DISPLAY=$XDG_RUNTIME_DIR/wayland-$n CHORA_SOCKET=$sock \
        "$foot" -a inner -f "DejaVu Sans Mono:size=10" \
        sh -c 'read l; printf %s "$l" >"$0"; sleep 600' "$work/$name.typed" &
    pids+=($!)
    # The window manager reached the nested IPC socket, not the outer one.
    until_ok 'CHORA_SOCKET=$sock ctl query "kind:surface owner:$name" | grep -q "\"inner\""' 80 ||
        fail "$name: the nested $name took no foot"
    [ "$(CHORA_SOCKET=$outer ctl query "owner:$name")" = "[]" ] || fail "$name: $name has nodes in the outer session"
    call pane.fly "{\"name\": \"$pane\"}" >/dev/null || fail "$name: pane.fly"
    sleep 2
    # shellcheck disable=SC2028 # \n is the JSON escape, for the core
    echo "[{\"text\": {\"text\": \"hello $name\\n\"}}]" | ctl inject || fail "$name: inject"
    until_ok 'test "$(cat "$work/$name.typed" 2>/dev/null)" = "hello $name"' ||
        fail "$name: the foot got: $(cat "$work/$name.typed" 2>/dev/null)"
    sleep 1
    ctl capture "$shots/construct-nested-$name.png" >/dev/null || fail "$name: capture"
    ok "$name: a fly-in gave the pane the keyboard, and the text reached the foot inside"
    echo '[{"shortcut": {"keys": "Alt+Escape"}}]' | ctl inject || fail "$name: fly out"
    sleep 1.5
    ctl capture "$shots/construct-nested-$name-space.png" >/dev/null || fail "$name: capture the space"
}

nested tiling 2 chora-tiling
nested xp 3 chora-xp --theme "$repo/flavors/xp/theme"
