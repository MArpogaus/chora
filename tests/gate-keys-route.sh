#!/bin/bash
# Gate of the key-made choice of a window manager (D55, M4, construct rx-1)
# in the test mode, where an injected key counts as a key of the person
# (--test-injected-press-is-real). One headless session: tiling manages the
# screen, xp the virtual output that tests/pane-host.py shows in a pane.
# Tiling starts first and holds the Super chords, xp the Alt ones.
# A terminal of xp in the pane has the keyboard, the pointer rests on the
# free screen below the pane, the claim of tiling.
#
# 1. A chord of tiling that opens no window (Super+Ctrl+r, reload) chooses
#    tiling where the pointer is. Alt+F4 of xp then closes its window: the
#    chord of the screen does not take the keys from the program on the
#    face (rx-1), and no key stays chosen for tiling.
# 2. A new terminal of xp has the keyboard again; Super+Return of tiling
#    opens its terminal in tiling, not in the face of xp that has the
#    keyboard (M4).
#
# Usage: tests/gate-keys-route.sh   BIN: directory with the binaries
#   SHOTS: directory for the logs (default: a temp dir)
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -uo pipefail
export CHORA_TEST_FLAGS=--test-injected-press-is-real
. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need python3
need_bins chora choractl chora-tiling chora-xp
temp_config
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
key() { inject "[{\"shortcut\": {\"keys\": \"$1\"}}]"; }
windows_of() { ctl query "kind:surface owner:$1" | jq length; }

session_start 1280x800
grant input.inject screen.capture methods.call methods.privileged
screen=$(ctl outputs | jq -r '.[0].name')
python3 -I "$repo/tests/pane-host.py" "$screen" 600x400+20+20 >"$shots/pane-host.log" 2>&1 &
pids+=($!)
until_ok 'grep -qx ready "$shots/pane-host.log"' 50 || fail "no pane host: $(tail -n 3 "$shots/pane-host.log")"
face=$(awk 'NF == 5 {print $1; exit}' "$shots/pane-host.log")
[ -n "$face" ] || fail "no virtual output: $(cat "$shots/pane-host.log")"

plugin() {
    local name=$1
    shift
    "$bin/chora-$name" "$@" >"$shots/$name.log" 2>&1 &
    pids+=($!)
    ctl wait "owner:$name" --timeout 20 >/dev/null || fail "$name does not start: $(tail -n 3 "$shots/$name.log")"
}
plugin tiling --output "$screen"
plugin xp --theme "$repo/flavors/xp/theme" --output "$face"

probe_n=0
face_window() {
    probe_n=$((probe_n + 1))
    inject '[{"pointer_move": {"position": [320, 220]}}]'
    foot -f "DejaVu Sans Mono:size=10" -a "probe$probe_n" sh -c 'sleep 900' >"$shots/foot$probe_n.log" 2>&1 &
    run_pid $!
    w=$(ctl wait "app_id:probe$probe_n kind:surface owner:xp" --timeout 20 | jq -r '.[0].id') ||
        fail "the terminal under the pane does not go to xp: $(ctl query kind:surface | jq -c '[.[] | {id, owner}]')"
    until_ok '[ "$(ctl query "id:$w" | jq -r ".[0].focused")" = true ]' 40 || fail "xp does not focus $w"
    inject '[{"pointer_move": {"position": [960, 640]}}]'
}

face_window
key Super+Ctrl+r
key Alt+F4
until_ok '[ "$(windows_of xp)" = 0 ]' 40 ||
    fail "rx-1: Alt+F4 after a chord of tiling does not reach xp, which owns the keyboard focus (swallowed for the chosen tiling)"
ok "rx-1: Alt+F4 reaches xp after a chord of tiling with the pointer on the screen"

face_window
key Super+Return
until_ok '[ "$(windows_of tiling)" -gt 0 ] || [ "$(windows_of xp)" -gt 1 ]' 60 ||
    fail "Super+Return opened no window"
[ "$(windows_of xp)" = 1 ] ||
    fail "M4: Super+Return of tiling with the pointer on its claim opened its window in xp, which has the keyboard"
[ "$(windows_of tiling)" -gt 0 ] || fail "M4: Super+Return of tiling opened no window of tiling"
ok "M4: Super+Return of tiling opens its window in tiling, not in the face of xp that has the keyboard"
ok "gate-keys-route: the key-made choice follows the person (${SECONDS}s)"
