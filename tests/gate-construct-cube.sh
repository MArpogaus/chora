#!/bin/bash
# Gate of the cube of the construct (D69 step 10, D76): the construct adds
# four virtual outputs, starts one chora-xp on them and shows each as a
# forwarding face of a cube in its 3D context.
#
# 1. cube.open: four virtual outputs come, each with an xp taskbar; a
#    second cube.open is refused.
# 2. A probe window opens on the front face, which shows its output at
#    1:1: a click near its corner reaches it at that point.
# 2a. Standing back (K4, docs/cube-modes.md): view.step_back shows the
#    cube from a step back, the front face forwards nothing and the chips
#    TURN LEFT, TURN RIGHT and STEP IN show; a still cube draws nothing,
#    by render.stats; Ctrl+Alt+Right and Ctrl+Alt+Left turn it; Return
#    steps in; Alt+Escape steps back from a face, which shows the chip
#    STEP BACK; a click on the chip STEP IN steps in.
# 3. A slow cube.turn forwards nothing while it runs; then the next face
#    is at the front, and the face before looks away and hides. xp moves
#    the window to the output of the next face.
# 4. When xp ends, the construct removes the outputs and the cube, and
#    the probe window is not lost: a screen shows it, or a client other
#    than the ended xp owns it.
# 5. A window manager that never connects: the outputs go after the
#    wait. cube.close removes a cube.
# 6. A construct that starts again removes the virtual outputs that it
#    left.
#
# Usage: tests/gate-construct-cube.sh   BIN: directory with the binaries
#   SHOT  a PNG path that keeps a capture of the cube during a turn
# pid_of NAME gives the first process of the binary NAME in this session.
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
virtuals() { ctl outputs | python3 -I -c 'import json, sys; print(" ".join(sorted(o["name"] for o in json.load(sys.stdin) if o.get("virtual"))))'; }
bars() {
    local n=0 o
    for o in $(virtuals); do
        [ -n "$(ctl query "id:xp-bar-$o" | python3 -I -c 'import json, sys; print(len(json.load(sys.stdin)) or "")')" ] && n=$((n + 1))
    done
    echo "$n"
}
prop() { ctl query "id:$1" | python3 -I -c "import json, sys; n = json.load(sys.stdin); print(n[0].get('$2', False) if n else 'gone')"; }
probe() {
    ctl query kind:surface | python3 -I -c '
import json, sys
for n in json.load(sys.stdin):
    if n["surface"].get("app_id") == "chora-probe":
        for out, r in n["rects"].items():
            print(n["id"], n.get("owner"), out, round(r[0]), round(r[1]))
            break
        else:
            print(n["id"], n.get("owner"), "-", 0, 0)'
}
pid_of() {
    python3 -I - "$CHORA_SOCKET" "$1" <<'EOF' | head -n 1
import os, sys
for pid in sorted(filter(str.isdigit, os.listdir("/proc")), key=int):
    try:
        cmd = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
        env = open(f"/proc/{pid}/environ", "rb").read().split(b"\0")
    except OSError:
        continue
    if cmd[0].endswith(sys.argv[2].encode()) and b"CHORA_SOCKET=" + sys.argv[1].encode() in env:
        print(pid)
EOF
}

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"

opened=$(call cube.open '{}') || fail "cube.open: $opened"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "no four virtual outputs: $(virtuals)"
until_ok 'test "$(bars)" = 4' 80 || fail "xp shows $(bars) taskbars on $(virtuals)"
ok "the cube shows four xp taskbars on $(virtuals)"
if call cube.open '{}' >/dev/null 2>&1; then fail "a second cube.open was not refused"; fi
ok "a second cube.open is refused"

inject '[{"pointer_move": {"position": [640, 400]}}]'
"$bin/chora-probe" window 40 120 200 --report >"$work/probe.log" 2>&1 &
pids+=($!)
until_ok 'probe | grep -q " xp VIRTUAL"' 80 || fail "the probe window came to no xp face: $(probe)"
read -r surface _ out x y <<<"$(probe)"
front=$(echo "$opened" | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["outputs"][0])')
[ "$out" = "$front" ] || fail "the probe window went to $out, not to the front face $front"
inject "[{\"pointer_move\": {\"position\": [$((x + 3)), $((y + 2))]}}, {\"click\": {}}]"
until_ok "grep -q '^press' $work/probe.log" 20 || fail "a click on the front face did not reach the probe window"
grep '^press' "$work/probe.log" | tail -n 1 | awk '{ dx = $2 - 3; dy = $3 - 2; exit !(dx * dx + dy * dy <= 1) }' ||
    fail "the front face is not 1:1: the click at (3, 2) of the window came at $(grep '^press' "$work/probe.log" | tail -n 1)"
ok "a click near the corner of the xp window on the front face came at (3, 2): 1:1"

width() { ctl query "id:$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(round(n[0]["rects"]["HEADLESS-1"][2]) if n and n[0]["rects"] else 0)'; }
drawn() { ctl render-stats | python3 -I -c 'import json, sys; print(sorted((k, v["damaged_frames"], v["passes"]) for k, v in json.load(sys.stdin)["outputs"].items()))'; }
still() {
    local before
    for _ in 1 2; do
        before=$(drawn)
        sleep 2
        [ "$before" != "$(drawn)" ] || return 0
    done
    return 1
}
key() { inject "[{\"shortcut\": {\"keys\": \"$1\"}}]"; }
stood_back='test "$(prop construct-cube-face-0 forward)" = False && test "$(width construct-cube-face-0)" -lt 1100 && test "$(width construct-cube-face-0)" -gt 300'
stepped_in='test "$(prop construct-cube-face-0 forward)" = True && test "$(width construct-cube-face-0)" = 1280'
call view.step_back >/dev/null || fail "view.step_back"
until_ok "$stood_back" 20 || fail "no step back: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
for chip in turn-left turn-right step-in; do
    [ "$(prop "construct-mode-$chip" visible)" = True ] || fail "the chip $chip does not show in navigate mode"
done
[ "$(prop construct-mode-step-back visible)" != True ] || fail "the chip STEP BACK shows in navigate mode"
ok "view.step_back: the cube stands back, the face forwards nothing, three chips show"
sleep 2
still || fail "a still cube draws: $(drawn)"
ok "a still cube renders 0 passes and has no damage, by render.stats"
key Ctrl+Alt+Right
until_ok 'test "$(prop construct-cube-face-1 visible)" = True && test "$(prop construct-cube-face-0 visible)" = False' 20 || fail "Ctrl+Alt+Right did not turn the cube"
key Ctrl+Alt+Left
until_ok 'test "$(prop construct-cube-face-0 visible)" = True && test "$(prop construct-cube-face-1 visible)" = False' 20 || fail "Ctrl+Alt+Left did not turn the cube back"
ok "Ctrl+Alt+Right and Ctrl+Alt+Left turn the cube in navigate mode"
inject '[{"key": {"key": "Return", "state": "pressed"}}, {"key": {"key": "Return", "state": "released"}}]'
until_ok "$stepped_in" 20 || fail "Return did not step in: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
[ "$(prop construct-mode-step-back visible)" = True ] || fail "the chip STEP BACK does not show in use mode"
key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back from the face"
inject '[{"pointer_move": {"node": "construct-mode-step-in", "point": [8, 8]}}, {"click": {}}]'
until_ok "$stepped_in" 20 || fail "a click on the chip STEP IN did not step in"
ok "Return steps in, Alt+Escape steps back, and a click on STEP IN steps in"

call cube.turn '{"by": 1, "ms": 3000}' >/dev/null || fail "a slow cube.turn"
sleep 1
[ -n "${SHOT:-}" ] && ctl capture "$SHOT" >/dev/null
[ "$(prop construct-cube-face-0 forward)" = False ] && [ "$(prop construct-cube-face-1 forward)" = False ] ||
    fail "a face forwards during the turn: $(prop construct-cube-face-0 forward) $(prop construct-cube-face-1 forward)"
call cube.turn '{"by": 1}' >/dev/null || fail "cube.turn during a turn"
until_ok 'test "$(prop construct-cube-face-1 visible)" = True && test "$(prop construct-cube-face-0 visible)" = False && test "$(prop construct-cube-face-1 forward)" = True' 40 ||
    fail "the cube did not turn: face 0 $(prop construct-cube-face-0 visible), face 1 $(prop construct-cube-face-1 visible) $(prop construct-cube-face-1 forward)"
[ "$(prop construct-cube-face-2 visible)" = False ] || fail "a turn during a turn went on to face 2"
ok "a slow turn forwarded nothing, a turn during it was ignored, and face 1 is at the front"
second=$(echo "$opened" | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["outputs"][1])')
ctl call xp window.move "{\"surface\": \"$surface\", \"output\": \"$second\"}" >/dev/null || fail "xp window.move"
until_ok 'probe | grep -q " $second "' 20 || fail "the probe window did not move to $second: $(probe)"
ok "a window moved from the face of $front to the face of $second, which is at the front"

xp_pid=$(pid_of chora-xp)
[ -n "$xp_pid" ] || fail "no xp process of this session"
run_pid "$xp_pid"
kill "$xp_pid"
until_ok 'test -z "$(virtuals)"' 40 || fail "the virtual outputs stay after xp ended: $(virtuals)"
until_ok 'test "$(prop construct-cube-face-0 visible)" = gone' || fail "the cube stays after xp ended"
ok "xp ended: the construct removed the outputs and the cube"
until_ok 'read -r _ owner out _ <<<"$(probe)"; test "$out" = HEADLESS-1 || { test "$owner" != xp && test "$owner" != None; }' 60 ||
    fail "the probe window $surface is lost after xp ended: $(probe)"
ok "the probe window is not lost: $(probe)"

call settings.set '{"cube_manager": "sleep 60"}' >/dev/null || fail "settings.set cube_manager"
call cube.open '{}' >/dev/null || fail "cube.open with a manager that never connects"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "no outputs for the waiting cube"
until_ok 'test -z "$(virtuals)"' 80 || fail "the outputs of a manager that never connected stay: $(virtuals)"
ok "the outputs of a window manager that never connected went after the wait"
call cube.open '{}' >/dev/null || fail "cube.open again"
call cube.close >/dev/null || fail "cube.close"
until_ok 'test -z "$(virtuals)"' || fail "cube.close left $(virtuals)"
ok "cube.close removed the cube and its outputs"

call settings.set '{"cube_manager": "chora-xp"}' >/dev/null || fail "settings.set cube_manager back"
call cube.open '{}' >/dev/null || fail "cube.open before the restart"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "no outputs before the restart"
construct=$(pid_of chora-construct)
kill -9 "$construct"
"$bin/chora-construct" >"$work/construct2.log" 2>&1 &
pids+=($!)
until_ok 'test -z "$(virtuals)"' 16 || fail "the restarted construct left its outputs: $(virtuals)"
ok "a restarted construct removed the outputs that it left"
xp_pid=$(pid_of chora-xp)
[ -z "$xp_pid" ] || kill "$xp_pid"
