#!/bin/bash
# Gate of the way out of a pane (co-76, rpc1-2 of rc1-pass r1) and of
# Super+x in a face of the cube (co-75, rpc1-1): the close acts only on the
# window that the person sees focused.
# 0. Flown into a pane in the walk, after its 3 s of hints, the hint
#    `Step back` stays on the screen, and a click on it steps out (no key
#    known, no pointer rest at the bottom).
# 1. That window is a pane in the walk; the cube opens beside it
#    (Super+c, it stands back) and Return steps into face 0, so the pane
#    waits at home in front of the cube, out of view.
# 2. A window of the xp of face 0 has the keys; Super+x closes it, and the
#    pane, its window and its process stay.
# 3. With no window focused in the face, Super+x closes nothing: the pane
#    out of view stays.
# 4. Flown into that pane while the cube stays in use mode (pane.fly),
#    Super+x closes it (rcc1-1).
# Both windows are chora-probe --exit-on-close: a close request ends it.
#
# Usage: tests/gate-construct-close.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-xp chora-probe
temp_config
export PATH=$bin:$PATH

key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
core_focus() { ctl query '*' | python3 -I -c 'import json, sys; print("\n".join(n["id"] for n in json.load(sys.stdin) if n.get("focused")))'; }
xp_focus() { ctl call xp state.get '{}' 2>/dev/null | python3 -I -c 'import json, sys; f = json.load(sys.stdin)["focus"]; print(f.get("holder") or "-", f.get("surface") or "-")' 2>/dev/null || echo "- -"; }
quiet() { ctl call construct state.get '{}' | python3 -I -c 'import json, sys; sys.exit(1 if json.load(sys.stdin)["busy"] else 0)'; }
mode() { ctl call construct cube.list | python3 -I -c 'import json, sys; print(json.load(sys.stdin).get("mode", ""))'; }
pane_sources() { ctl query kind:surface | python3 -I -c 'import json, sys; print(" ".join(n["id"] for n in json.load(sys.stdin) if n.get("owner") == "construct"))'; }
panes_n() { ctl call construct construct.list | python3 -I -c 'import json, sys; print(len(json.load(sys.stdin)["panes"]))'; }
rect_of() { ctl query "id:$1" | jq -r '.[0].rects | to_entries[0].value | @tsv' 2>/dev/null; }
alive() { kill -0 "$1" 2>/dev/null; }
dump() { echo "core: $(core_focus | tr '\n' ' ') xp: $(xp_focus) panes: $(panes_n) cube: $(ctl call construct cube.list | tr -d ' \n' | cut -c1-200)"; }

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"

"$bin/chora-probe" window 200 40 40 --exit-on-close >/dev/null 2>&1 &
pane_pid=$!
pids+=("$pane_pid")
until_ok 'test "$(panes_n)" = 1' 40 || fail "the window in the walk became no pane: $(dump)"
until_ok quiet 80 || fail "the construct is still busy after the pane"
# shellcheck disable=SC2034 # until_ok reads it
read -r pane < <(pane_sources)
until_ok 'core_focus | grep -qx "$pane"' 8 || key Return
until_ok 'core_focus | grep -qx "$pane"' 40 || fail "the camera is in no pane: $(dump)"
sleep 5
read -r x y w h < <(rect_of construct-hint-step-back)
[ -n "${h:-}" ] || fail "co-76: after the 3 s of the hints nothing in the pane leads out: no hint Step back ($(dump))"
read -r _ _ _ kh < <(rect_of construct-hint-keys)
[ -n "${kh:-}" ] || fail "co-76: after the 3 s of the hints nothing in the pane leads to the key list ($(dump))"
echo "[{\"pointer_move\": {\"position\": [$((${x%.*} + ${w%.*} / 2)), $((${y%.*} + ${h%.*} / 2))]}}, {\"click\": {}}]" | ctl inject >/dev/null
until_ok 'ctl query id:construct-look | jq -e ".[0].focused == true" >/dev/null' 40 || fail "co-76: a click on the hint Step back did not step out of the pane: $(dump)"
ok "flown into a pane, after the 3 s of its hints, Step back and All keys stay, and a click on Step back steps out"
until_ok quiet 40 || true
key Super+c
until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "Super+c opened no cube with xp: $(dump)"
until_ok quiet 80 || fail "the cube is still busy: $(ctl call construct state.get '{}')"
until_ok 'test "$(mode)" = navigate' 40 || fail "the cube of Super+c does not stand back: $(dump)"
key Return
until_ok 'test "$(mode)" = use' 40 || fail "Return did not step into face 0: $(dump)"
until_ok quiet 80 || fail "the cube is still busy after the step in"
alive "$pane_pid" && test "$(panes_n)" = 1 || fail "the pane went before any close: $(dump)"
ok "a pane waits out of view while the cube shows face 0"

click_face() { echo '[{"pointer_move": {"position": [640, 300]}}, {"click": {}}]' | ctl inject >/dev/null; }
until_ok 'xp_focus | grep -q "^xp" || { click_face; false; }' 40 || fail "a click on face 0 gave xp no keys: $(dump)"
"$bin/chora-probe" window 200 100 50 --exit-on-close >/dev/null 2>&1 &
face_pid=$!
pids+=("$face_pid")
until_ok 'xp_focus | grep -q "^xp s"' 80 || fail "the window on face 0 is no focused window of xp: $(dump)"
read -r _ face < <(xp_focus)
until_ok 'core_focus | grep -qx "$face"' 40 || fail "the core does not focus $face of face 0: $(dump)"
key Super+x
until_ok '! alive "$face_pid"' 40 || fail "co-75: Super+x left the focused face window $face open: $(dump)"
sleep 1
alive "$pane_pid" || fail "co-75: Super+x in a face closed the pane out of view: $(dump)"
test "$(panes_n)" = 1 || fail "co-75: Super+x in a face took the pane out of view: $(dump)"
ok "Super+x in a face closed the focused face window $face and kept the pane out of view ($(dump))"

until_ok '! core_focus | grep -qx "$face" && ! xp_focus | grep -q "^xp s"' 40 || fail "co-75: a face window still has the keys after its close, the test of no window focused does not hold: $(dump)"
key Super+x
sleep 2
alive "$pane_pid" || fail "co-75: Super+x with no window focused in the face closed the pane out of view: $(dump)"
test "$(panes_n)" = 1 || fail "co-75: Super+x with no window focused in the face took the pane: $(dump)"
ok "Super+x with no window focused in the face closed nothing out of view ($(dump))"

pane_name=$(ctl call construct construct.list | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["panes"][0]["name"])')
ctl call construct pane.fly "{\"name\": \"$pane_name\"}" >/dev/null || fail "rcc1-1: pane.fly $pane_name: $(dump)"
until_ok 'core_focus | grep -qx "$pane"' 40 || fail "rcc1-1: the camera is in no pane flown from the face: $(dump)"
until_ok 'test "$(mode)" = use' 8 || fail "rcc1-1: the cube left its use mode: $(dump)"
key Super+x
until_ok '! alive "$pane_pid"' 40 || fail "rcc1-1: Super+x left the pane flown into from a face open: $(dump)"
ok "Super+x in a pane flown into from a face closed that pane ($(dump))"
