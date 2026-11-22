#!/bin/bash
# Two looks on one output, as the persona session tiling+xp has them (p2r2
# ricer, T7 and I3): tiling starts first and holds the give keys, xp joins
# on the same output without `--output`. A foot starts with xp. Cells:
#
#   keys  xp focuses its window, and the key list of tiling takes the
#         keyboard: one look says it holds the focus
#   give  Super+Alt+1 gives the window of xp to tiling: tiling owns and
#         focuses it, and it shows on the screen, not under the desktop of
#         xp (the pixel at its centre is not the pixel there before)
#   click a click on the desktop of xp takes the keys to its icons, and a
#         click on the window of tiling takes them back: the core focuses
#         the window, tiling holds the focus, xp none (rw-1)
#   node  tiling focuses the window, and the Start menu of xp opens and
#         closes (method start_menu): its menu, then its desktop icons take
#         the keyboard with a node; one look says it holds the focus
#
# The holder is the act invariant I3 of tests/acts/invariants.py (one_holder
# prints its breaches), on
# state.get of both looks and scene.query. It runs from the matrix row of
# tests/merge-gate.sh beside tests/gate-handover-matrix.sh.
#
# Usage: tests/gate-handover-shared.sh   BIN: directory with the binaries
#   SHOTS: directory for the logs and captures (default: a temp dir)
set -uo pipefail
. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need python3
need_bins chora choractl chora-tiling chora-xp
temp_config
accept_injected() {
    printf 'test_accept_injected_keys: true\n' >"$XDG_CONFIG_HOME/chora-$1.yaml"
}
accept_injected tiling
accept_injected xp
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
export PATH=$bin:$PATH

session_start 1280x800
grant input.inject screen.capture methods.call methods.privileged
"$bin/chora-tiling" >"$shots/tiling.log" 2>&1 &
pids+=($!)
ctl wait owner:tiling --timeout 20 >/dev/null || fail "tiling does not start: $(tail -n 3 "$shots/tiling.log")"
"$bin/chora-xp" --theme "$repo/flavors/xp/theme" >"$shots/xp.log" 2>&1 &
pids+=($!)
ctl wait id:xp-icons --timeout 20 >/dev/null || fail "xp does not start: $(tail -n 3 "$shots/xp.log")"
sleep 1

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
focused() {
    for _ in $(seq $(($1 * 5))); do
        [ "$(ctl query "id:$w" | jq -r '.[0].focused')" = true ] && return 0
        sleep 0.2
    done
    return 1
}
one_holder() {
    local dir=$shots/$1
    mkdir -p "$dir"
    ctl state tiling >"$dir/tiling.json" || fail "$1: state of tiling"
    ctl state xp >"$dir/xp.json" || fail "$1: state of xp"
    ctl query >"$dir/nodes.json" || fail "$1: scene.query"
    python3 -I - "$repo/tests/acts" "$dir" <<'EOF'
import json, pathlib, sys, types
sys.path.insert(0, sys.argv[1])
import invariants
d = pathlib.Path(sys.argv[2])
states = {look: json.loads((d / f"{look}.json").read_text()) for look in ("tiling", "xp")}
snap = types.SimpleNamespace(states=states, nodes=json.loads((d / "nodes.json").read_text()))
for breach in invariants.i3(None, snap, None):
    print(breach)
EOF
}
holds() {
    local breaches
    breaches=$(one_holder "$1")
    [ -z "$breaches" ] || fail "$1: $breaches"
}

ctl capture "$shots/before.png" >/dev/null || fail "capture before"
foot -f "DejaVu Sans Mono:size=10" -a probe -o colors.background=c81e1e sh -c 'sleep 900' >"$shots/foot.log" 2>&1 &
foot_pid=$!
run_pid "$foot_pid"
w=$(ctl wait "app_id:probe kind:surface owner:xp" --timeout 20 | jq -r '.[0].id') ||
    fail "foot does not show in xp: $(ctl query kind:surface | jq -c '[.[] | {id, owner}]')"
focused 10 || fail "xp does not focus $w"

ctl call tiling show_keys >/dev/null || fail "keys: show_keys"
sleep 0.5
holds keys
ctl call tiling show_keys >/dev/null || fail "keys: hide the key list"
sleep 0.5
ok "keys: the key list of tiling over a window of xp, one holder"

key Super+Alt+1
ctl wait "id:$w owner:tiling" --timeout 10 >/dev/null || fail "give: $w stays with $(ctl query "id:$w" | jq -r '.[0].owner')"
sleep 1
focused 5 || fail "give: tiling does not focus $w"
holds give
rect=$(ctl state tiling | jq -r --arg w "$w" '.windows[] | select(.surface == $w and .shown) | .rect | map(floor) | join(" ")')
[ -n "$rect" ] || fail "give: tiling does not show $w"
read -r x y rw rh <<<"$rect"
cx=$((x + rw / 2)) cy=$((y + rh / 2))
ctl capture "$shots/give.png" >/dev/null || fail "capture give"
was=$(pixel "$shots/before.png" "$cx" "$cy")
now=$(pixel "$shots/give.png" "$cx" "$cy")
[ "$now" != "$was" ] || fail "give: $w is not on the screen: at $cx,$cy the desktop of xp ($now), capture $shots/give.png"
ok "give: Super+Alt+1 hands $w to tiling, focused and on the screen ($now at its centre)"

click() {
    echo "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"button\": {\"state\": \"pressed\"}}, {\"button\": {\"state\": \"released\"}}]" |
        ctl inject >/dev/null || fail "inject click $1,$2"
    sleep 0.5
}
holder_of() { ctl state "$1" | jq -r '.focus.holder // "none"'; }
click 5 400
[ "$(holder_of xp)" = xp ] || fail "click: the desktop of xp is clicked and xp holds no focus: $(ctl state xp | jq -c .focus)"
click "$cx" "$cy"
focused 5 || fail "click: a click on $w does not give it the keys back (tiling $(ctl state tiling | jq -c .focus), xp $(ctl state xp | jq -c .focus))"
holds click
[ "$(holder_of tiling)" = tiling ] && [ "$(holder_of xp)" = none ] ||
    fail "click: holders tiling $(holder_of tiling), xp $(holder_of xp)"
ok "click: the desktop of xp takes the keys, a click on $w at $cx,$cy takes them back to tiling"

ctl call xp start_menu '{"open": true}' >/dev/null || fail "node: open the Start menu of xp"
sleep 0.5
holds node-menu
ctl call xp start_menu '{"open": false}' >/dev/null || fail "node: close the Start menu of xp"
sleep 0.5
holds node-desktop
ok "node: the Start menu of xp opens and closes over the window of tiling, one holder"
start_menu_open() { ctl query id:xp-item-0 | jq -e '.[0].rects != null and (.[0].rects | length) > 0' >/dev/null; }
ctl call xp start_menu '{"open": true}' >/dev/null || fail "rx-1: open the Start menu of xp"
until_ok start_menu_open 20 || fail "rx-1: the Start menu of xp does not open"
kill "$foot_pid"
window_gone() { [ "$(ctl query "id:$w" | jq length)" = 0 ]; }
until_ok window_gone 40 || fail "rx-1: $w does not go away"
icons_focused() { [ "$(ctl query id:xp-icons | jq -r '.[0].focused')" = true ]; }
if until_ok icons_focused 8; then
    fail "rx-1: the window of tiling went and the desktop icons of xp took the keys from its open Start menu"
fi
start_menu_open || fail "rx-1: the Start menu of xp closed with the window of tiling"
ok "rx-1: a window of tiling that exits leaves the keys with the open Start menu of xp"
ok "gate-handover-shared: tiling and xp on one output hand over with one holder (${SECONDS}s)"
