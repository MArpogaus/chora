#!/bin/bash
# Gate of the keyboard between two manager panes of the construct: a
# click into the pane of tiling gives the keys to tiling, not to xp.
#
# 1. xp and tiling run each in a pane, and a terminal in the pane of xp
#    has the keyboard (the tour, sh-23). A click into the empty pane of
#    tiling takes the keyboard from it, then a tap of Super: the Start
#    menu of xp stays closed.
# 2. Super+Return then opens a window of tiling on the output of its
#    pane, and xp keeps its one window.
#
# Usage: tests/gate-construct-pane-keys.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
export CHORA_TEST_FLAGS=--test-injected-press-is-real
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-xp chora-tiling
need foot
temp_config
export PATH=$bin:$PATH

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
call() { ctl call construct "$@"; }
inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
rect() { ctl query "id:construct-manager-$1-source" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(*[round(v) for v in n[0]["rects"]["HEADLESS-1"]] if n and n[0]["rects"] else [0, 0, 0, 0])'; }
menu_open() { ctl query id:xp-item-0 | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print("yes" if n and n[0]["rects"] else "no")'; }
windows_of() { ctl query "kind:surface owner:$1" | python3 -I -c 'import json, sys; print(len(json.load(sys.stdin)))'; }
key() { inject "[{\"key\": {\"key\": \"$1\", \"state\": \"pressed\"}}, {\"key\": {\"key\": \"$1\", \"state\": \"released\"}}]"; }

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
call pane.manager '{"manager": "xp", "size": [560, 400]}' >/dev/null || fail "pane.manager xp"
call pane.manager '{"manager": "tiling", "size": [560, 400]}' >/dev/null || fail "pane.manager tiling"
ctl wait owner:tiling --timeout 20 >/dev/null || fail "tiling is not live"
ctl wait id:xp-start-VIRTUAL-1 --timeout 20 >/dev/null || ctl wait 'id:xp-bar-*' --timeout 20 >/dev/null || fail "xp is not live"
until_ok 'read -r _ _ w _ <<<"$(rect tiling)"; test "$w" -gt 100' 40 || fail "the pane of tiling does not show: $(rect tiling)"
until_ok 'read -r _ _ w _ <<<"$(rect xp)"; test "$w" -gt 100' 40 || fail "the pane of xp does not show: $(rect xp)"
sleep 2

focused_of() { ctl query "kind:surface owner:$1" | python3 -I -c 'import json, sys; print(sum(1 for n in json.load(sys.stdin) if n.get("focused")))'; }
read -r qx qy qw qh <<<"$(rect xp)"
inject "[{\"pointer_move\": {\"position\": [$((qx + qw / 2)), $((qy + qh / 2))]}}, {\"click\": {}}]"
sleep 0.5
foot sh >/dev/null 2>&1 &
run_pid $!
until_ok 'test "$(windows_of xp)" -gt 0' 60 || fail "a terminal started after a click into the pane of xp opened no window of xp (tiling has $(windows_of tiling))"
until_ok 'test "$(focused_of xp)" = 1' 40 || fail "the terminal in the pane of xp has no keyboard"
ok "a terminal in the pane of xp has the keyboard"

read -r px py pw ph <<<"$(rect tiling)"
inject "[{\"pointer_move\": {\"position\": [$((px + pw / 2)), $((py + ph / 2))]}}, {\"click\": {}}]"
sleep 0.5
[ "$(focused_of xp)" = 0 ] || fail "a click into the empty pane of tiling leaves the keyboard on the terminal of xp: typed keys stray into it"
ok "a click into the empty pane of tiling takes the keyboard from the terminal of xp"
inject '[{"key": {"key": "Super_L", "state": "pressed"}}, {"key": {"key": "Super_L", "state": "released"}}]'
sleep 1.5
[ "$(menu_open)" = no ] || fail "a tap of Super after a click into the pane of tiling opens the Start menu of xp"
ok "a tap of Super after a click into the pane of tiling leaves the Start menu of xp closed"

inject '[{"key": {"key": "Super_L", "state": "pressed"}}, {"key": {"key": "Return", "state": "pressed"}}, {"key": {"key": "Return", "state": "released"}}, {"key": {"key": "Super_L", "state": "released"}}]'
until_ok 'test "$(windows_of tiling)" -gt 0' 60 || fail "Super+Return after a click into the pane of tiling opened no window of tiling (xp has $(windows_of xp))"
[ "$(windows_of xp)" = 1 ] || fail "Super+Return after a click into the pane of tiling opened a window of xp"
ok "Super+Return after a click into the pane of tiling opens a window of tiling"
ok "gate-construct-pane-keys: the keys follow the click into a manager pane"
