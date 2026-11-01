#!/bin/bash
# Gate of step K1 of the construct track: a terminal pane takes typed
# text after a fly-in, pixel-sharp at 1:1. chora starts into
# chora-construct; foot becomes a pane; keys walk; Return flies into the
# pane; the injected text reaches foot; the output then shows the window
# pixel for pixel as a capture of the window alone; Alt+Escape flies out;
# a walk during a drag keeps its place.
#
# Usage: tests/gate-construct-k1.sh   BIN: directory with the binaries
#   THEME  light (default) or dark
#   SHOTS  a directory that keeps the captures as construct-THEME-*.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct
need foot
theme=${THEME:-light}
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work}
mkdir -p "$work/chora" "$shots"
printf 'theme: %s\n' "$theme" >"$work/chora/chora-construct.yaml"

# A debug core on the software renderer takes longer than a frame for the
# fog and a pane; the watchdog would take their shaders away (as
# tests/xp.sh). The frame rate of the space is the gate of K3, not this.
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
session_start 1280x800 -- "$bin/chora-construct"
grant screen.capture input.inject
shot() { ctl capture "$@" >/dev/null || fail "capture $*"; }
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject || fail "inject $1"; }
# The JSON of the first node of a selector, as python sees it.
node() { ctl query "$1" | python3 -I -c "import json, sys; n = json.load(sys.stdin); print($2 if n else '')"; }

ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
shot "$shots/construct-$theme-space.png"

foot -f "DejaVu Sans Mono:size=10" sh -c \
    "echo 'chora-construct K1: type a line'; read line; printf %s \"\$line\" >$work/typed; echo \"got: \$line\"; sleep 600" &
pids+=($!)
ctl wait --timeout 10 'kind:surface owner:construct' >/dev/null || fail "foot became no pane"
surface=$(node 'kind:surface owner:construct' 'n[0]["id"]')
pane=construct-pane-$surface
sleep 1
before=$(node "id:$pane" 'n[0]["rects"]')
[ -n "$before" ] || fail "the pane $pane does not show"
shot "$shots/construct-$theme-pane.png"
ok "foot is the pane $pane at $before"

for k in e e e a a s s; do key "$k"; done
sleep 0.6
after=$(node "id:$pane" 'n[0]["rects"]')
[ "$after" != "$before" ] || fail "the walk did not move the pane: $after"
shot "$shots/construct-$theme-walk.png"
ok "the walk moved the pane to $after"

key Return
sleep 1
[ "$(node "id:$surface" 'n[0]["opacity"]')" = 1.0 ] || fail "the fly-in did not show the window"
# shellcheck disable=SC2028 # \n is the JSON escape, for the core
echo '[{"text": {"text": "hello construct\n"}}]' | ctl inject || fail "inject text"
for _ in $(seq 30); do
    [ -s "$work/typed" ] && break
    sleep 0.1
done
[ "$(cat "$work/typed" 2>/dev/null)" = "hello construct" ] || fail "foot got: $(cat "$work/typed" 2>/dev/null)"
ok "after the fly-in, foot took the typed text"

sleep 0.5
shot "$shots/construct-$theme-flown.png"
shot --node "$surface" "$work/window.png"
diff=$(python3 -I "$repo/tests/png-same.py" "$shots/construct-$theme-flown.png" "$work/window.png") ||
    fail "compare: $diff"
[ "$diff" = "0 0" ] || fail "the output differs from the window alone: $diff (pixels, worst channel)"
ok "the output shows the window at 1:1, pixel for pixel"

key Alt+Escape
sleep 1
[ "$(node "id:$surface" 'n[0]["opacity"]')" = 0.0 ] || fail "Alt+Escape did not fly out"
[ "$(node id:construct-look 'n[0]["focused"]')" = True ] || fail "the walk has no keyboard after the fly-out"
back=$(node "id:$pane" 'n[0]["rects"]')
[ "$back" = "$after" ] || fail "the fly-out did not return to the pose before: $back, not $after"
shot "$shots/construct-$theme-out.png"
ok "Alt+Escape flew out, and the walk has the keyboard again"

# The mouse: the wheel walks, a drag looks, a click on the pane flies in.
before=$(node "id:$pane" 'n[0]["rects"]')
echo '[{"pointer_move": {"position": [100, 700]}}, {"scroll": {"dy": -60}}]' | ctl inject || fail "inject scroll"
sleep 0.6
after=$(node "id:$pane" 'n[0]["rects"]')
[ "$after" != "$before" ] || fail "the wheel did not walk"
echo '[{"pointer_move": {"position": [100, 700]}}, {"button": {"state": "pressed"}},
  {"pointer_move": {"position": [160, 700]}}, {"button": {"state": "released"}}]' | ctl inject ||
    fail "inject drag"
sleep 0.3
# A walk during a drag stays: drag right, walk with w, drag back. The
# view turns back, and the pane is nearer, so wider, than before.
width() { node "id:$pane" 'round(list(n[0]["rects"].values())[0][2])'; }
wide=$(width)
echo '[{"pointer_move": {"position": [100, 700]}}, {"button": {"state": "pressed"}},
  {"pointer_move": {"position": [130, 700]}}]' | ctl inject || fail "inject drag"
key w
key w
sleep 0.5
echo '[{"pointer_move": {"position": [100, 700]}}, {"button": {"state": "released"}}]' | ctl inject ||
    fail "inject drag back"
sleep 0.3
[ "$(width)" -gt "$wide" ] || fail "the walk during the drag was lost: width $wide, then $(width)"
ok "a walk during a drag stays: the pane grew from $wide to $(width)"
centre=$(node "id:$pane" '"%d, %d" % (lambda r: (r[0] + r[2] / 2, r[1] + r[3] / 2))(list(n[0]["rects"].values())[0])')
[ -n "$centre" ] || fail "the pane left the view after the drag"
echo "[{\"pointer_move\": {\"position\": [$centre]}}, {\"click\": {}}]" | ctl inject || fail "inject click"
sleep 1
[ "$(node "id:$surface" 'n[0]["opacity"]')" = 1.0 ] || fail "a click on the pane at $centre did not fly in"
ok "the wheel walked, a drag looked, and a click at $centre flew into the pane"
