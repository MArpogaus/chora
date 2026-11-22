#!/bin/bash
# Wobbly windows of tiling: with the default settings a floating window
# that Super+drag moves bends, and its mesh is gone when it settles after
# the release. A tiled window never bends.
#
# Usage: tests/gate-tiling-wobbly.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling chora-probe
temp_config
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800 -- "$bin/chora-tiling"
grant input.inject

started=$(ctl run "$bin/chora-probe" window 40 120 200) || fail "the run of the window"
pid=$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")
[ -n "$pid" ] || fail "the run of the window"
run_pid "$pid"
ctl wait --timeout 10 'app_id:chora-probe kind:surface' >/dev/null || fail "no window"
sleep 2

inject() { echo "$1" | ctl inject - >/dev/null || fail "inject $1"; }
bent() {
    ctl query 'app_id:chora-probe kind:surface' | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
print(1 if nodes and nodes[0].get("mesh") else 0)'
}
bend_is() {
    local _
    for _ in $(seq "${2:-30}"); do
        [ "$(bent)" = "$1" ] && return 0
        sleep 0.2
    done
    return 1
}
drag() {
    inject '[{"pointer_move": {"position": [640, 400]}},
        {"key": {"key": "Super_L", "state": "pressed"}},
        {"button": {"state": "pressed", "hold": true}}]'
    inject '[{"pointer_move": {"position": [700, 430]}}]'
    inject '[{"pointer_move": {"position": [800, 470]}}]'
}

drag
[ "$(bent)" = 0 ] || fail "a tiled window bends under a drag"
inject '[{"button": {"state": "released"}}]'
ok "a tiled window does not wobble"

inject '[{"shortcut": {"keys": "Super+Ctrl+space"}}]'
settle
drag
bend_is 1 || fail "a floating window does not bend under a drag"
ok "a floating window wobbles under a drag"
inject '[{"button": {"state": "released"}}]'
bend_is 0 40 || fail "the mesh stays after the window settled"
ok "the mesh is gone when the window has settled"
