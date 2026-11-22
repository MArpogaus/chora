#!/bin/bash
# Gate of the single focus holder in a face of the cube (invariant I3 of
# tests/acts, persona round 5): the construct opens the cube, one click
# lands on the desktop of the xp face, and then exactly one of construct and
# xp says it holds the focus in `state.get` (xp, which has the node of its
# icons focused; the construct must say nobody).
# No window that shows false holds the keys with the cube open, and a key
# typed after the click reaches no such window (co-46).
#
# Usage: tests/gate-construct-i3.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-xp chora-probe
temp_config
export PATH=$bin:$PATH

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
surface_held() { ctl call construct state.get '{}' | python3 -I -c 'import json, sys; sys.exit(0 if json.load(sys.stdin)["focus"].get("surface") else 1)'; }
holder() { ctl call "$1" state.get '{}' | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["focus"].get("holder") or "-")'; }

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
"$bin/chora-probe" window 200 100 50 --report >"$XDG_CONFIG_HOME/pane.log" 2>&1 &
pids+=($!)
until_ok 'ctl call construct window.list | grep -q "\"surface\""' 80 || fail "the window of the probe became no pane: $(cat "$XDG_CONFIG_HOME/pane.log")"
until_ok 'surface_held || { echo "[{\"pointer_move\": {\"position\": [640, 400]}}, {\"click\": {}}]" | ctl inject >/dev/null; false; }' 40 || fail "the click on the pane gave the construct no window focus: $(ctl call construct state.get "{}")"
ctl call construct cube.open >/dev/null || fail "cube.open failed"
until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "xp shows no taskbar on the front face"
until_ok 'ctl call construct state.get "{}" | python3 -I -c "import json, sys; sys.exit(1 if json.load(sys.stdin)[\"busy\"] else 0)"' 80 || fail "the cube is still busy: $(ctl call construct state.get "{}")"
hidden_holder() { ctl call construct state.get '{}' | python3 -I -c 'import json, sys; d = json.load(sys.stdin); hidden = {w["surface"] for w in d["windows"] if not w["shown"]}; sys.exit(0 if d["focus"].get("surface") in hidden else 1)'; }
! hidden_holder || fail "the cube is open over the window of the pane and that window, shown false, holds the keys: $(ctl call construct state.get "{}")"
click_xp() { echo '[{"pointer_move": {"position": [40, 30]}}, {"click": {}}]' | ctl inject >/dev/null; }
click_xp
until_ok 'false' 1
echo '[{"key": {"key": "x", "state": "pressed"}}, {"key": {"key": "x", "state": "released"}}]' | ctl inject >/dev/null
until_ok 'false' 4
! grep -q '^key' "$XDG_CONFIG_HOME/pane.log" || fail "a key reached the window of the pane, which the cube hides (shown false): $(cat "$XDG_CONFIG_HOME/pane.log") $(ctl call construct state.get "{}")"
both=0
for _ in $(seq 40); do
    construct=$(holder construct)
    xp=$(holder xp)
    if [ "$construct" != - ] && [ "$xp" != - ]; then
        both=$((both + 1))
    else
        both=0
    fi
    if [ "$both" -ge 3 ]; then
        fail "construct and xp each say they hold the focus after a click in the xp face with a window in a pane (construct: $construct, xp: $xp)"
    fi
    [ "$xp" != - ] && [ "$construct" = - ] && break
    until_ok 'false' 1
done
ok "one focus holder after a click in the xp face with a window in a pane (construct: $construct, xp: $xp)"
