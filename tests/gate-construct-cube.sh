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
#    by render.stats. The method leaves the keyboard with the window.
#    The chips are at least 24 px high, below the title band of a window
#    and above the taskbar. In use mode Escape reaches the window, the
#    turn keys turn the cube, and a turn key during a turn queues one
#    turn. Alt+Escape steps back and takes the keyboard; Return steps in
#    and the window of before takes typed text again, but not after a
#    step into another face. A click on a face and a click on the chip
#    STEP IN step in. In navigate mode the edge of the face under the
#    pointer is lit, and a drag on a face turns the cube by one face.
#    With the grant input.grab-any and fly_out Escape, Escape alone
#    steps back; without the grant the chip names no key. After the
#    switcher of xp showed and closed,
#    Alt+Escape steps back again; that the switcher has Alt+Escape while
#    it shows is a unit test of the construct, as injected keys do not
#    stay down between two requests.
# 2b. Floating windows: stood back, each window of xp on a face floats
#    in front of it as a plane of its own, the upper one farther, and
#    the face keeps the wallpaper and the taskbar; a still cube with
#    floating windows draws nothing; after the step in the windows are
#    in the face again, pixel for pixel. A window floats during a turn.
#    A window that opens while the cube floats gets a plane by the
#    event xp.floating, and loses it when it closes.
#    While the window manager does not answer, a window keeps its
#    plane, so it never shows nowhere; it lands when the answer comes.
#    FLOAT_SHOT  a PNG path that keeps a capture of the floating windows
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
edge() { ctl query "id:$1" | python3 -I -c "import json, sys; n = json.load(sys.stdin); print(round(n[0]['rects']['HEADLESS-1'][$2]) if n and n[0]['rects'] else -1)"; }
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
plain() { inject "[{\"key\": {\"key\": \"$1\", \"state\": \"pressed\"}}, {\"key\": {\"key\": \"$1\", \"state\": \"released\"}}]"; }
keys_got() { grep -c "^key $1\$" "$work/probe.log"; }
front_is() { test "$(prop "construct-cube-face-$1" visible)" = True && test "$(prop "construct-cube-face-$1" forward)" = True; }
stood_back='test "$(prop construct-cube-face-0 forward)" = False && test "$(width construct-cube-face-0)" -lt 1100 && test "$(width construct-cube-face-0)" -gt 300'
stepped_in='test "$(prop construct-cube-face-0 forward)" = True && test "$(width construct-cube-face-0)" = 1280'

call view.step_back >/dev/null || fail "view.step_back"
until_ok "$stood_back" 20 || fail "no step back: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
for chip in turn-left turn-right step-in; do
    [ "$(prop "construct-mode-$chip" visible)" = True ] || fail "the chip $chip does not show in navigate mode"
done
[ "$(prop construct-mode-step-back visible)" != True ] || fail "the chip STEP BACK shows in navigate mode"
before=$(keys_got 30)
plain a
until_ok 'test "$(keys_got 30)" -gt "$before"' 20 || fail "view.step_back, a method, took the keyboard from the window"
ok "view.step_back: the cube stands back, the face forwards nothing, three chips show, the keyboard stays"
[ "$(edge construct-mode-step-in 1)" -gt 400 ] || fail "the chips are in the upper half, over the title band: y $(edge construct-mode-step-in 1)"
[ "$(edge construct-mode-step-in 3)" -ge 24 ] || fail "a chip is $(edge construct-mode-step-in 3) px high, less than 24"
bar_top=$(ctl query "id:xp-panel-$front" | python3 -I -c "import json, sys; print(round(json.load(sys.stdin)[0]['rects']['$front'][1]))")
[ $(($(edge construct-mode-step-in 1) + $(edge construct-mode-step-in 3))) -le "$bar_top" ] || fail "the chips are over the taskbar at y $bar_top"
ok "the chips are at least 24 px high, below the title band and above the taskbar"
sleep 2
still || fail "a still cube draws: $(drawn)"
ok "a still cube renders 0 passes and has no damage, by render.stats"
call view.step_in >/dev/null || fail "view.step_in"
until_ok "$stepped_in" 20 || fail "view.step_in did not step in"
prop construct-mode-step-back-text text | grep -q "Alt+Esc" || fail "the chip STEP BACK does not name its key: $(prop construct-mode-step-back-text text)"

before=$(keys_got 1)
plain Escape
until_ok 'test "$(keys_got 1)" -gt "$before"' 20 || fail "Escape in use mode did not reach the window"
sleep 0.5
eval "$stepped_in" || fail "Escape in use mode moved the view"
ok "in use mode Escape reaches the window on the face, and the view stays"

key Ctrl+Alt+Right
until_ok 'front_is 1' 20 || fail "Ctrl+Alt+Right did not turn the cube in use mode"
key Ctrl+Alt+Left
until_ok 'front_is 0' 20 || fail "Ctrl+Alt+Left did not turn the cube back in use mode"
key Ctrl+Alt+Right
key Ctrl+Alt+Right
key Ctrl+Alt+Right
until_ok 'front_is 2' 20 || fail "a turn key during a turn did not queue one turn: $(prop construct-cube-face-1 forward) $(prop construct-cube-face-2 forward) $(prop construct-cube-face-3 forward)"
sleep 1
front_is 2 || fail "a third turn key during a turn turned on"
key Ctrl+Alt+Left
key Ctrl+Alt+Left
until_ok 'front_is 0' 20 || fail "two turn keys to the left did not come back to face 0"
ok "the turn keys turn the cube in use mode, and a turn key during a turn queues one turn"

key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back from the face"
before=$(keys_got 30)
plain a
sleep 0.5
[ "$(keys_got 30)" = "$before" ] || fail "after Alt+Escape the window still gets typed text"
plain Return
until_ok "$stepped_in" 20 || fail "Return did not step in: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
[ "$(prop construct-mode-step-back visible)" = True ] || fail "the chip STEP BACK does not show in use mode"
before=$(keys_got 30)
plain a
until_ok 'test "$(keys_got 30)" -gt "$before"' 20 || fail "typed text after the step in did not reach the window"
ok "Alt+Escape steps back, Return steps in, and typed text reaches the window of before"

key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back before a step in by a method"
call view.step_in >/dev/null || fail "view.step_in after a step back of the person"
until_ok "$stepped_in" 20 || fail "view.step_in did not step in after a step back of the person"
before=$(keys_got 30)
plain a
sleep 0.5
[ "$(keys_got 30)" = "$before" ] || fail "view.step_in, a method, gave the keyboard to the window"
key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back after a step in by a method"
plain Return
until_ok "$stepped_in" 20 || fail "Return did not step in after a step in by a method"
ok "a step in by a method moves no keyboard"

key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back a second time"
key Ctrl+Alt+Right
until_ok 'test "$(prop construct-cube-face-1 visible)" = True && test "$(prop construct-cube-face-0 visible)" = False' 20 || fail "Ctrl+Alt+Right did not turn the cube in navigate mode"
plain Return
until_ok 'front_is 1' 20 || fail "Return did not step into face 1"
before=$(keys_got 30)
plain a
sleep 0.5
[ "$(keys_got 30)" = "$before" ] || fail "the step into face 1 gave the keyboard to a window on face 0"
key Ctrl+Alt+Left
until_ok 'front_is 0' 20 || fail "Ctrl+Alt+Left did not turn back to face 0"
ok "a step into another face does not give the keyboard to the window of the face of before"

key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back a third time"
inject '[{"pointer_move": {"position": [640, 400]}}, {"click": {}}]'
until_ok "$stepped_in" 20 || fail "a click on the front face in navigate mode did not step in"
key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back a fourth time"
inject '[{"pointer_move": {"node": "construct-mode-step-in", "point": [8, 8]}}, {"click": {}}]'
until_ok "$stepped_in" 20 || fail "a click on the chip STEP IN did not step in"
ok "a click on a face and a click on the chip STEP IN step in"

[ "$(prop construct-cube-edge visible)" != True ] || fail "the edge of a face is lit in use mode"
key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back for the hover"
inject '[{"pointer_move": {"position": [10, 10]}}, {"pointer_move": {"position": [640, 400]}}]'
until_ok 'test "$(prop construct-cube-edge visible)" = True' 20 || fail "the edge of the face under the pointer is not lit: $(prop construct-cube-edge visible)"
inject '[{"pointer_move": {"position": [10, 10]}}]'
until_ok 'test "$(prop construct-cube-edge visible)" = False' 20 || fail "the edge stays lit after the pointer left the face"
ok "in navigate mode the edge of the face under the pointer is lit"

drag() { inject "[{\"pointer_move\": {\"position\": [$1, 400]}}, {\"button\": {\"state\": \"pressed\", \"hold\": true}}, {\"pointer_move\": {\"position\": [$(( ($1 + $2) / 2 )), 400]}}, {\"pointer_move\": {\"position\": [$2, 400]}}, {\"button\": {\"state\": \"released\"}}]"; }
drag 800 300
until_ok 'test "$(prop construct-cube-face-1 visible)" = True && test "$(prop construct-cube-face-0 visible)" = False' 20 || fail "a drag to the left did not turn the cube to the next face"
sleep 1
[ "$(prop construct-cube-face-1 forward)" = False ] || fail "a drag on the cube stepped in"
drag 500 1000
until_ok 'test "$(prop construct-cube-face-0 visible)" = True && test "$(prop construct-cube-face-1 visible)" = False' 20 || fail "a drag to the right did not turn the cube back"
sleep 1
drag 640 700
sleep 1
eval "$stood_back" || fail "a short drag turned the cube or left navigate mode"
call view.step_in >/dev/null || fail "view.step_in after the drags"
until_ok "$stepped_in" 20 || fail "no step in after the drags"
ok "a drag of a third of the view turns the cube by one face; a shorter drag neither turns it nor steps in"

call settings.set '{"fly_out": "Escape"}' >/dev/null || fail "settings.set fly_out Escape"
sleep 0.5
if prop construct-mode-step-back-text text | grep -q "Esc"; then fail "the chip names Escape although the construct cannot grab it: $(prop construct-mode-step-back-text text)"; fi
"$bin/choractl" grant construct input.grab-any || fail "the grant input.grab-any"
call settings.set '{"fly_out": "Escape"}' >/dev/null || fail "settings.set fly_out Escape with the grant"
until_ok 'prop construct-mode-step-back-text text | grep -q "  Esc$"' 20 || fail "the chip does not name Escape: $(prop construct-mode-step-back-text text)"
plain Escape
until_ok "$stood_back" 20 || fail "Escape alone did not step back with input.grab-any"
plain Escape
until_ok "$stepped_in" 20 || fail "Escape did not step in again"
call settings.set '{"fly_out": "Alt+Escape"}' >/dev/null || fail "settings.set fly_out Alt+Escape"
until_ok 'prop construct-mode-step-back-text text | grep -q "Alt+Esc"' 20 || fail "the chip does not name Alt+Escape again"
ok "with input.grab-any and fly_out Escape, Escape alone steps back and in; without the grant the chip names no key"

inject '[{"key": {"key": "Alt_L", "state": "pressed"}}, {"key": {"key": "Tab", "state": "pressed"}}, {"key": {"key": "Tab", "state": "released"}}, {"key": {"key": "Alt_L", "state": "released"}}]'
sleep 0.5
[ "$(prop xp-alttab visible)" = gone ] || fail "the switcher of xp stays after Alt went up"
eval "$stepped_in" || fail "Alt+Tab moved the view"
key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape does not step back after the switcher closed"
call view.step_in >/dev/null || fail "view.step_in after the switcher"
until_ok "$stepped_in" 20 || fail "no step in after the switcher: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
ok "after the switcher of xp closed, Alt+Escape steps back again"

"$bin/chora-probe" window 200 40 40 >"$work/probe2.log" 2>&1 &
second_probe=$!
pids+=("$second_probe")
others() { probe | awk -v s="$surface" '$1 != s && $2 == "xp" {print $1}' | head -n 1; }
until_ok 'test -n "$(others)"' 40 || fail "the second probe window came to no xp face: $(probe)"
upper=$(others)
opacity() { ctl query "id:xp-win-$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print("%g" % n[0].get("opacity", 1) if n else "gone")'; }
lifted() {
    ctl query "id:construct-cube-float-0-$1" | python3 -I -c '
import json, sys
n = json.load(sys.stdin)
face = float(sys.argv[1])
if not n or "HEADLESS-1" not in n[0]["rects"]:
    print(0)
else:
    print(round(1000 * n[0]["rects"]["HEADLESS-1"][2] / (n[0]["size"][0] * face / 1280)))' "$(width construct-cube-face-0)"
}
floats() {
    sleep 1
    ctl capture "$work/flat.png" >/dev/null || return 1
    call view.step_back >/dev/null || return 1
    until_ok "$stood_back" 20 || return 1
    sleep 1
    [ "$(prop "construct-cube-float-0-$surface" visible)" = True ] && [ "$(prop "construct-cube-float-0-$upper" visible)" = True ] ||
        { echo "the planes do not show: $(prop "construct-cube-float-0-$surface" visible) $(prop "construct-cube-float-0-$upper" visible)"; return 1; }
    [ "$(opacity "$surface")" = 0 ] && [ "$(opacity "$upper")" = 0 ] || { echo "the windows stay in the face: opacity $(opacity "$surface") $(opacity "$upper")"; return 1; }
    lower_lift=$(lifted "$surface")
    upper_lift=$(lifted "$upper")
    [ "$lower_lift" -gt 1030 ] || { echo "the lower window is not lifted: $lower_lift per mille of its flat width"; return 1; }
    [ "$upper_lift" -gt $((lower_lift + 5)) ] || { echo "the upper window is not lifted farther: $upper_lift, lower $lower_lift"; return 1; }
    [ -z "${FLOAT_SHOT:-}" ] || ctl capture "$FLOAT_SHOT" >/dev/null
    still || { echo "a still cube with floating windows draws: $(drawn)"; return 1; }
    call view.step_in >/dev/null || return 1
    until_ok "$stepped_in" 20 || return 1
    until_ok 'test "$(prop "construct-cube-float-0-$surface" visible)" = gone && test "$(opacity "$surface")" = 1 && test "$(opacity "$upper")" = 1' 20 ||
        { echo "the windows did not land: plane $(prop "construct-cube-float-0-$surface" visible), opacity $(opacity "$surface") $(opacity "$upper")"; return 1; }
    sleep 0.5
    ctl capture "$work/landed.png" >/dev/null || return 1
    cmp -s "$work/flat.png" "$work/landed.png" || { echo "the face after the step in differs from the face before the step back"; return 1; }
    echo "$lower_lift and $upper_lift"
}
rest() {
    call view.step_in >/dev/null 2>&1
    until_ok "$stepped_in" 20 && until_ok 'test "$(prop "construct-cube-float-0-$surface" visible)" = gone' 20
}
why=$(floats) || { rest && why=$(floats); } || fail "floating windows: $why"
ok "stood back, the windows float off the face, the upper one farther ($why per mille of the flat width), and land pixel for pixel"
call view.step_back >/dev/null || fail "view.step_back for a new window"
until_ok "$stood_back" 20 || fail "no step back for a new window"
"$bin/chora-probe" window 40 200 40 >"$work/probe3.log" 2>&1 &
third_probe=$!
pids+=("$third_probe")
newest() { probe | awk -v a="$surface" -v b="$upper" '$1 != a && $1 != b && $2 == "xp" {print $1}' | head -n 1; }
until_ok 'test -n "$(newest)"' 40 || fail "the third probe window came to no xp face: $(probe)"
new=$(newest)
until_ok 'test "$(prop "construct-cube-float-0-$new" visible)" = True && test "$(opacity "$new")" = 0' 20 ||
    fail "a window that opens while the cube floats does not float: plane $(prop "construct-cube-float-0-$new" visible), opacity $(opacity "$new")"
run_pid "$third_probe"
kill "$third_probe"
until_ok 'test "$(prop "construct-cube-float-0-$new" visible)" = gone' 40 || fail "the plane of a closed window stays"
[ "$(prop "construct-cube-float-0-$surface" visible)" = True ] || fail "the other windows do not float on after a window closed"
call view.step_in >/dev/null || fail "view.step_in after a new window"
until_ok "$stepped_in" 20 || fail "no step in after a new window"
ok "a window that opens while the cube floats gets a plane, and loses it when it closes"
manager=$(pid_of chora-xp)
[ -n "$manager" ] || fail "no xp process of this session"
call view.step_back >/dev/null || fail "view.step_back for a deaf window manager"
until_ok "$stood_back" 20 || fail "no step back for a deaf window manager"
until_ok 'test "$(opacity "$surface")" = 0' 20 || fail "the window does not float before the window manager stops"
kill -STOP "$manager"
call view.step_in >/dev/null || { kill -CONT "$manager"; fail "view.step_in with a deaf window manager"; }
sleep 3
deaf_plane=$(prop "construct-cube-float-0-$surface" visible)
timeout 2 "$bin/choractl" --name agent call construct construct.list >/dev/null 2>&1
deaf_answer=$?
kill -CONT "$manager"
[ "$deaf_answer" = 0 ] || fail "the construct does not answer while the window manager is deaf"
[ "$deaf_plane" = True ] || fail "the plane went although the window manager did not answer: the window shows nowhere ($deaf_plane)"
until_ok 'test "$(prop "construct-cube-float-0-$surface" visible)" = gone && test "$(opacity "$surface")" = 1' 40 ||
    fail "the windows did not land after the window manager answered again: plane $(prop "construct-cube-float-0-$surface" visible), opacity $(opacity "$surface")"
until_ok "$stepped_in" 20 || fail "no step in after the window manager answered again"
ok "a window keeps its plane while the window manager does not answer, and lands when it does"
run_pid "$second_probe"
kill "$second_probe"
until_ok 'test -z "$(others)"' 40 || fail "the second probe window stays"

call cube.turn '{"by": 1, "ms": 3000}' >/dev/null || fail "a slow cube.turn"
sleep 1
[ "$(prop "construct-cube-float-0-$surface" visible)" != gone ] || fail "the window does not float during a turn"
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
