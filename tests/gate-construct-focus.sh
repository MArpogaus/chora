#!/bin/bash
# Gate of the keys on the faces of the cube (co-50, co-51 of persona round
# p2-r6-co): one holder at all times, and a window that does not show never
# holds the keys.
# 1. A window of the xp of face 0 has the keys; the cube turns to face 1:
#    the window shows no more, so the core focuses it no more and a key
#    typed then does not reach it (co-50, a pager click turns the same way).
# 2. On face 1 a window comes and takes the keys, as the stand-in of the
#    lock does, and closes: the keys go to the face (the icons of its xp),
#    not to nobody and not back to the window of face 0 (co-51).
# A click on the desktop of face 0 first gives xp the keys, so that the
# first window is a window of the face, as Start, Terminal makes one.
#
# Usage: tests/gate-construct-focus.sh   BIN: directory with the binaries
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
core_focus() { ctl query '*' | python3 -I -c 'import json, sys; print("\n".join(n["id"] for n in json.load(sys.stdin) if n.get("focused")))'; }
xp_focus() { ctl call xp state.get '{}' 2>/dev/null | python3 -I -c 'import json, sys; f = json.load(sys.stdin)["focus"]; print(f.get("holder") or "-", f.get("surface") or "-")' 2>/dev/null || echo "- -"; }
quiet() { ctl call construct state.get '{}' | python3 -I -c 'import json, sys; sys.exit(1 if json.load(sys.stdin)["busy"] else 0)'; }
dump() { echo "core: $(core_focus | tr '\n' ' ') xp: $(xp_focus) construct: $(ctl call construct state.get '{}' | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["focus"])')"; }

first_held() { core_focus | grep -qx "$first"; }
xp_new_window() {
    local h s
    read -r h s < <(xp_focus)
    test "$h" = xp && test "$s" != - && test "$s" != "$first"
}
lock_closed() { ! ctl query "id:$lock" | grep -q "\"$lock\""; }

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
ctl call construct cube.open >/dev/null || fail "cube.open failed"
until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "xp shows no taskbar on the front face"
until_ok quiet 80 || fail "the cube is still busy: $(ctl call construct state.get "{}")"
click_face() { echo '[{"pointer_move": {"position": [640, 300]}}, {"click": {}}]' | ctl inject >/dev/null; }
until_ok 'xp_focus | grep -q "^xp" || { click_face; false; }' 40 || fail "a click on face 0 gave xp no keys: $(dump)"

"$bin/chora-probe" window 200 100 50 --report >"$XDG_CONFIG_HOME/face0.log" 2>&1 &
pids+=($!)
until_ok 'xp_focus | grep -q "^xp s"' 80 || fail "the window on face 0 is no focused window of xp: $(dump)"
read -r _ first < <(xp_focus)
until_ok first_held 40 || fail "the core does not focus $first of face 0: $(dump)"

ctl call construct cube.turn '{"to": 1}' >/dev/null || fail "cube.turn failed"
until_ok 'ctl call construct cube.list | grep -q "\"turning\": *false" && ctl call construct cube.list | grep -q "\"front\": *1"' 80 ||
    fail "the cube did not turn to face 1: $(ctl call construct cube.list)"
until_ok quiet 80 || fail "the cube is still busy after the turn"
until_ok '! first_held' 20 ||
    fail "the cube shows face 1 and $first of face 0, which does not show, still holds the keys: $(dump)"
: >"$XDG_CONFIG_HOME/face0.log"
echo '[{"key": {"key": "x", "state": "pressed"}}, {"key": {"key": "x", "state": "released"}}]' | ctl inject >/dev/null
until_ok 'false' 4
! grep -q '^key' "$XDG_CONFIG_HOME/face0.log" || fail "a key reached $first of face 0, which does not show: $(dump)"
ok "a turn takes the keys from the window of a face that does not show ($(dump))"

timeout 6 "$bin/chora-probe" window 16 24 48 --title "Screen locked" >/dev/null 2>&1 &
pids+=($!)
until_ok xp_new_window 40 ||
    fail "the window on face 1 is no focused window of xp: $(dump)"
read -r _ lock < <(xp_focus)
until_ok lock_closed 60 || fail "the window $lock did not close"
until_ok 'core_focus | grep -q .' 20 || fail "the window $lock of face 1 closed and the core focuses nothing: the next key goes nowhere ($(dump))"
! first_held || fail "the keys went back to $first of face 0, which does not show: $(dump)"
read -r holder _ < <(xp_focus)
[ "$holder" = xp ] || fail "the keys are on face 1 but xp does not hold them: $(dump)"
ok "the keys stay with the front face after its focused window closed ($(dump))"
