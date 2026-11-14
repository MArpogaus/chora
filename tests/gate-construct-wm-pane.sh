#!/bin/bash
# Gate of window manager panes in the construct (D69 step 9, K7): the
# method pane.manager adds a virtual output, starts a window manager on
# it as a child and shows the output in a forwarding pane of the space.
#
# 1. pane.manager with xp and with tiling: each manager is live within
#    10 s on a virtual output of its own, and its pane shows on the
#    screen. A second pane of xp is refused, and so is cube.open while a
#    pane has xp.
# 2. While the construct has the keyboard, a window that opens under
#    the pane of a manager belongs to that manager, on the output of the
#    pane; a click on the empty space gives the construct the keyboard
#    back after a window in a pane had it. A click through the pane of xp
#    gives its window the keyboard, and a typed key reaches it, also
#    while xp answers the click late: xp stands still for 1 s after the
#    press. The keyboard moves when xp answers, so the key waits for it.
# 3. With nothing moving, render.stats counts no pass and no damaged
#    frame of the virtual outputs in 2 s. The screen is not measured:
#    the glow of the focused pane of the construct moves with the time.
# 4. construct.save and construct.load bring the panes back: the
#    managers run again on new outputs.
# 5. When the child of the xp pane ends, the pane and its output go, and
#    the window that xp had is not lost.
# 6. pane.remove of the tiling pane ends its child and removes its
#    output, and the window that tiling had is not lost.
#
# Usage: tests/gate-construct-wm-pane.sh   BIN: directory with the binaries
#   SHOT  a PNG path that keeps a capture of the two panes
# pid_of NAME gives the first process of the binary NAME in this session.
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-xp chora-tiling chora-probe
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
virtuals() { ctl outputs | python3 -I -c 'import json, sys; print(" ".join(sorted(o["name"] for o in json.load(sys.stdin) if o.get("virtual"))))'; }
panes() { call construct.list | python3 -I -c 'import json, sys; print(" ".join(sorted(p["name"] for p in json.load(sys.stdin)["panes"] if p["kind"] == "manager")))'; }
output_of() { call construct.list | python3 -I -c 'import json, sys; print(next((p["output"] for p in json.load(sys.stdin)["panes"] if p.get("manager") == sys.argv[1]), ""))' "$1"; }
rect() { ctl query "id:construct-manager-$1-source" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(*[round(v) for v in n[0]["rects"]["HEADLESS-1"]] if n and n[0]["rects"] else [0, 0, 0, 0])'; }
middle() { read -r px py pw ph <<<"$(rect "$1")"; echo "$((px + pw / 2)), $((py + ph / 2))"; }
focused() { ctl query "id:$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(n[0].get("focused") if n else "")'; }
windows_of() { ctl query "kind:surface owner:$1" | python3 -I -c 'import json, sys; print(" ".join(n["id"] for n in json.load(sys.stdin) if n["surface"].get("app_id") == "chora-probe"))'; }
window_on() { ctl query "id:$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(*[round(v) for v in n[0]["rects"].get(sys.argv[1], [0, 0, 0, 0])] if n else [0, 0, 0, 0])' "$2"; }
pid_of() {
    python3 -I - "$CHORA_SOCKET" "$1" <<'PY' | head -n 1
import os, sys
for pid in sorted(filter(str.isdigit, os.listdir("/proc")), key=int):
    try:
        cmd = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
        env = open(f"/proc/{pid}/environ", "rb").read().split(b"\0")
    except OSError:
        continue
    if cmd[0].endswith(sys.argv[2].encode()) and b"CHORA_SOCKET=" + sys.argv[1].encode() in env:
        print(pid)
PY
}

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"

call pane.manager '{"manager": "xp", "size": [560, 400]}' >/dev/null || fail "pane.manager xp"
call pane.manager '{"manager": "tiling", "size": [560, 400]}' >/dev/null || fail "pane.manager tiling"
until_ok 'test "$(virtuals | wc -w)" = 2' || fail "no two virtual outputs: $(virtuals)"
xp_out=$(output_of xp)
tiling_out=$(output_of tiling)
[ -n "$xp_out" ] && [ -n "$tiling_out" ] && [ "$xp_out" != "$tiling_out" ] || fail "the panes name no outputs of their own: xp $xp_out, tiling $tiling_out"
ctl wait "id:xp-bar-$xp_out" --timeout 10 >/dev/null || fail "xp is not live on $xp_out within 10 s"
ctl wait "owner:tiling" --timeout 10 >/dev/null || fail "tiling is not live within 10 s"
until_ok 'read -r _ _ w _ <<<"$(rect xp)"; test "$w" -gt 100' 20 || fail "the pane of xp does not show: $(rect xp)"
until_ok 'read -r _ _ w _ <<<"$(rect tiling)"; test "$w" -gt 100' 20 || fail "the pane of tiling does not show: $(rect tiling)"
ok "xp runs on $xp_out and tiling on $tiling_out, each in a pane: $(panes)"
if call pane.manager '{"manager": "xp"}' >/dev/null 2>&1; then fail "a second pane of xp was not refused"; fi
if call cube.open '{}' >/dev/null 2>&1; then fail "cube.open was not refused while a pane has xp"; fi
ok "a second pane of xp and a cube of xp are refused"

opens_under() {
    local before
    before=$(windows_of "$1")
    inject "[{\"pointer_move\": {\"position\": [$(middle "$1")]}}]"
    "$bin/chora-probe" window "$2" "$3" "$4" --report >"$work/probe-$1.log" 2>&1 &
    pids+=($!)
    until_ok "test \"\$(windows_of $1)\" != \"$before\"" 80 || return 1
}
opens_under xp 40 120 200 || fail "a window under the pane of xp did not go to xp: $(ctl query kind:surface | python3 -I -c 'import json, sys; print([(n["id"], n.get("owner")) for n in json.load(sys.stdin)])')"
xp_window=$(windows_of xp)
inject '[{"pointer_move": {"position": [100, 740]}}, {"click": {}}]'
sleep 0.5
opens_under tiling 200 40 40 || fail "a window under the pane of tiling did not go to tiling: panes xp $(rect xp), tiling $(rect tiling); $(ctl query kind:surface | python3 -I -c 'import json, sys; print([(n["id"], n.get("owner")) for n in json.load(sys.stdin)])')"
tiling_window=$(windows_of tiling)
until_ok 'read -r _ _ w _ <<<"$(window_on "$xp_window" "$xp_out")"; test "$w" -gt 0' 20 || fail "the window of xp does not show on $xp_out"
until_ok 'read -r _ _ w _ <<<"$(window_on "$tiling_window" "$tiling_out")"; test "$w" -gt 0' 20 || fail "the window of tiling does not show on $tiling_out"
ok "a window that opens under a pane belongs to the manager of that pane: $xp_window with xp, $tiling_window with tiling"

read -r px py pw ph <<<"$(rect xp)"
read -r wx wy ww wh <<<"$(window_on "$xp_window" "$xp_out")"
click="$((px + (wx + ww / 2) * pw / 560)), $((py + (wy + wh / 2) * ph / 400))"
xp_pid=$(pid_of chora-xp)
[ -n "$xp_pid" ] || fail "no xp process of this session"
kill -STOP "$xp_pid"
inject "[{\"pointer_move\": {\"position\": [$click]}}, {\"click\": {}}]"
until_ok "grep -q '^press' $work/probe-xp.log" 20 || fail "a click through the pane of xp at ($click) did not reach its window"
(sleep 1 && kill -CONT "$xp_pid") &
pids+=($!)
until_ok 'test "$(focused "$xp_window")" = True' 40 || fail "the click through the pane of xp did not give its window the keyboard"
inject '[{"key": {"key": "a", "state": "pressed"}}, {"key": {"key": "a", "state": "released"}}]'
until_ok "grep -q '^key 30\$' $work/probe-xp.log" 20 || fail "a typed key did not reach the window in the pane of xp"
ok "a click through the pane of xp and a typed key reached its window"
[ -z "${SHOT:-}" ] || ctl capture "$SHOT" >/dev/null

drawn() { ctl render-stats | python3 -I -c 'import json, sys; print(sorted((k, v["damaged_frames"], v["passes"]) for k, v in json.load(sys.stdin)["outputs"].items() if k.startswith("VIRTUAL")))'; }
still() {
    local before
    for _ in 1 2; do
        before=$(drawn)
        sleep 2
        [ "$before" != "$(drawn)" ] || return 0
    done
    return 1
}
sleep 2
still || fail "the outputs of still panes draw: $(drawn)"
ok "the outputs of still manager panes render 0 passes and have no damage: $(drawn)"

call construct.save '{"name": "managers"}' >/dev/null || fail "construct.save"
grep -q "manager: xp" "$work/chora/constructs/managers.yaml" || fail "the file has no pane of xp: $(cat "$work/chora/constructs/managers.yaml")"
call construct.load '{"name": "managers"}' >/dev/null || fail "construct.load"
until_ok 'test "$(panes | wc -w)" = 2 && test "$(virtuals | wc -w)" = 2' 40 || fail "the panes did not come back: $(panes), outputs $(virtuals): $(tail -n 3 "$XDG_RUNTIME_DIR/chora.log")"
xp_out=$(output_of xp)
tiling_out=$(output_of tiling)
ctl wait "id:xp-bar-$xp_out" --timeout 10 >/dev/null || fail "xp is not live again on $xp_out"
until_ok 'test -n "$(ctl query owner:tiling | python3 -I -c "import json, sys; print(len(json.load(sys.stdin)) or \"\")")"' 40 || fail "tiling is not live again"
ok "construct.save and construct.load bring both panes back: xp on $xp_out, tiling on $tiling_out"

owner_of() { ctl query "id:$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(n[0].get("owner") if n else "gone")'; }
shows() { ctl query "id:$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print("yes" if n and n[0]["rects"] else "no")'; }
xp_pid=$(pid_of chora-xp)
[ -n "$xp_pid" ] || fail "no xp process of this session"
run_pid "$xp_pid"
kill "$xp_pid"
until_ok 'test "$(panes)" = tiling' 40 || fail "the pane of xp stays after its child ended: $(panes)"
until_ok 'test "$(virtuals)" = "$tiling_out"' 40 || fail "the output of xp stays: $(virtuals)"
until_ok 'o=$(owner_of "$xp_window"); test "$o" != xp && test "$o" != None && test "$o" != gone && test "$(shows "$xp_window")" = yes' 60 ||
    fail "the window of xp is lost after xp ended: owner $(owner_of "$xp_window"), shows $(shows "$xp_window")"
ok "xp ended: its pane and its output went, and its window is with $(owner_of "$xp_window")"

tiling_pid=$(pid_of chora-tiling)
[ -n "$tiling_pid" ] || fail "no tiling process of this session"
call pane.remove '{"name": "tiling"}' >/dev/null || fail "pane.remove tiling"
until_ok 'test -z "$(virtuals)"' 40 || fail "the output of tiling stays after pane.remove: $(virtuals)"
until_ok '! kill -0 "$tiling_pid" 2>/dev/null' 40 || fail "the child of the tiling pane still runs"
[ -z "$(panes)" ] || fail "a manager pane stays: $(panes)"
until_ok 'o=$(owner_of "$tiling_window"); test "$o" != tiling && test "$o" != None && test "$o" != gone && test "$(shows "$tiling_window")" = yes' 60 ||
    fail "the window of tiling is lost after pane.remove: owner $(owner_of "$tiling_window"), shows $(shows "$tiling_window")"
ok "pane.remove ended tiling and removed its output, and its window is with $(owner_of "$tiling_window")"
ok "gate-construct-wm-pane: window managers run in panes of the construct"
