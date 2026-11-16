#!/bin/bash
# Gate of the way out of a cube of the construct (co-18, co-19, co-20):
# stood back, the cube is never a trap.
#
# 1. A drag that begins on a floating window turns the cube, where the
#    window cannot be pulled out (an injected drag), and no window is
#    lost: the window count, the owner and the output of each window are
#    the same before and after.
# 2. F1, with no modifier, shows the bindings overlay stood back; Escape
#    closes the overlay and leaves the cube.
# 3. Escape stood back closes the cube; its windows are all still there.
#
# Usage: tests/gate-construct-out.sh   BIN: directory with the binaries
#   OUT_SHOTS  a directory that keeps a capture after each step
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-xp chora-probe
temp_config
work=$XDG_CONFIG_HOME
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
call() { ctl call construct "$@"; }
inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
key() { inject "[{\"shortcut\": {\"keys\": \"$1\"}}]"; }
plain() { inject "[{\"key\": {\"key\": \"$1\", \"state\": \"pressed\"}}, {\"key\": {\"key\": \"$1\", \"state\": \"released\"}}]"; }
prop() { ctl query "id:$1" | python3 -I -c "import json, sys; n = json.load(sys.stdin); print(n[0].get('$2', False) if n else 'gone')"; }
virtuals() { ctl outputs | python3 -I -c 'import json, sys; print(" ".join(sorted(o["name"] for o in json.load(sys.stdin) if o.get("virtual"))))'; }
windows() {
    ctl query kind:surface | python3 -I -c '
import json, sys
for n in sorted(json.load(sys.stdin), key=lambda n: n["id"]):
    out = next(iter(n["rects"]), "-")
    print(n["id"], n.get("owner"), "face" if out.startswith("VIRTUAL") else out)'
}
plane_middle() {
    ctl query "id:construct-cube-float-0-$1" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
r = nodes[0]["rects"].get("HEADLESS-1") if nodes and nodes[0].get("visible") else None
print("%d %d" % (r[0] + r[2] / 2, r[1] + r[3] / 2) if r else "")'
}
shot() { [ -z "${OUT_SHOTS:-}" ] || ctl capture "$OUT_SHOTS/$1.png" >/dev/null || fail "capture $1"; }
stood_back='test "$(prop construct-cube-face-0 forward)" = False && test "$(prop construct-mode-step-in visible)" = True'

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
call cube.open '{}' >/dev/null || fail "cube.open"
until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "xp shows no taskbar on the front face"
inject '[{"pointer_move": {"position": [640, 400]}}]'
"$bin/chora-probe" window 40 120 200 --report >"$work/probe.log" 2>&1 &
pids+=($!)
until_ok 'windows | grep -q "xp face"' 80 || fail "the probe window came to no face: $(windows)"
surface=$(windows | awk '$2 == "xp" {print $1; exit}')
key Alt+Escape
until_ok "$stood_back" 20 || fail "no step back"
until_ok 'test -n "$(plane_middle "$surface")"' 40 || fail "no floating plane of the window"
sleep 2
before=$(windows)
read -r px py <<<"$(plane_middle "$surface")"
inject "[{\"pointer_move\": {\"position\": [$px, $py]}},
    {\"button\": {\"state\": \"pressed\", \"hold\": true}},
    {\"pointer_move\": {\"position\": [$((px - 100)), $py]}},
    {\"pointer_move\": {\"position\": [$((px - 300)), $py]}},
    {\"button\": {\"state\": \"released\"}}]"
until_ok 'prop construct-face-label-words text | grep -q "face 2 of 4"' 20 || fail "a drag that began on a floating window did not turn the cube: $(prop construct-face-label-words text), windows before $before, after $(windows); list $(call window.list "{}" | tr -d "\n "); pane $(ctl query "id:construct-pane-$surface" | tr -d "\n " | cut -c1-600)"
[ "$(windows)" = "$before" ] || fail "a drag on the cube changed its windows: before $before, after $(windows); list $(call window.list "{}" | tr -d "\n "); pane $(ctl query "id:construct-pane-$surface" | tr -d "\n " | cut -c1-600)"
shot drag-turn
ok "stood back, a drag that begins on a floating window turns the cube and loses no window: $(windows | tr '\n' ';')"
key Ctrl+Alt+Left
until_ok 'prop construct-face-label-words text | grep -q "face 1 of 4"' 20 || fail "no turn back to face 1"

plain F1
until_ok 'test "$(prop construct-help opacity)" = 1.0' 20 || fail "F1 stood back shows no bindings overlay: $(prop construct-help opacity)"
shot keys
ok "stood back, F1 shows the bindings overlay"
plain Escape
until_ok 'test "$(prop construct-help opacity)" != 1.0' 20 || fail "Escape did not close the overlay"
[ -n "$(virtuals)" ] || fail "Escape closed the cube with the overlay"
eval "$stood_back" || fail "Escape with the overlay left the stood-back view"

plain Escape
until_ok 'test -z "$(virtuals)"' 40 || fail "Escape stood back did not close the cube: $(virtuals)"
until_ok 'read -r _ owner out <<<"$(windows | grep "^$surface ")"; test "$out" = HEADLESS-1 || { test "$owner" != xp && test "$owner" != None; }' 60 ||
    fail "the window $surface is lost by closing the cube: before $before, after $(windows)"
[ "$(windows | wc -l)" = "$(echo "$before" | wc -l)" ] || fail "a window is lost by closing the cube: before $before, after $(windows)"
shot closed
ok "Escape stood back closes the cube, and no window is lost: $(windows | tr '\n' ';')"
