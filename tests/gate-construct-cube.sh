#!/bin/bash
# Gate of the cube of the construct (D69 step 10, D76): the construct adds
# four virtual outputs, starts one chora-xp on them and shows each as a
# forwarding face of a cube in its 3D context.
#
# 1. Without a cube no chip shows: the start card of the empty floor
#    names the key Super+C of the cube, with no hint bar. cube.open
#    without params: four virtual outputs come, each with an xp taskbar;
#    a second cube.open is refused.
# 2. A probe window opens on the front face, which shows its output at
#    1:1: a click near its corner reaches it at that point.
# 2a. Standing back (K4, docs/cube-modes.md): view.step_back shows the
#    cube from a step back, the front face forwards nothing and the chips
#    TURN LEFT, TURN RIGHT and STEP IN show; a still cube draws nothing,
#    by render.stats. The method leaves the keyboard with the window.
#    The chips of navigate mode are the hints at the bottom centre, at
#    least 24 px high; the chip STEP BACK covers no desktop icon.
#    In use mode Escape reaches the window, the turn keys turn the cube,
#    and a turn key during a turn queues one turn. Alt+Escape steps back
#    and takes the keyboard; Return steps in and the window of before
#    takes typed text again, but not after a step into another face. A click on a face and a click on the chip
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
# 3. A slow cube.turn forwards nothing while it runs; at a third of it
#    the screen shows two faces; the gate prints the frame time of the
#    turn; then the next face
#    is at the front, and the face before looks away and hides. xp moves
#    the window to the output of the next face.
# 4. When xp ends, the construct removes the outputs and the cube, and
#    the probe window is not lost: a screen shows it, or a client other
#    than the ended xp owns it.
# 5. A window manager that never connects: the outputs go after the
#    wait. cube.close removes a cube, and Super+C opens a cube.
# 6. The xp of the cube ends with a construct that is killed (C2). A
#    construct that starts again removes the virtual outputs that it
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
quick() {
    for _ in $(seq 100); do
        eval "$1" && return 0
        sleep 0.02
    done
    return 1
}
now_ms() { echo $(($(date +%s%N) / 1000000)); }
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

[ "$(prop construct-open-cube visible)" = gone ] || fail "a chip OPEN CUBE shows: no buttons in the construct"
until_ok 'ctl query id:construct-start-row-2-keys-1-text | grep -q "\"text\": \"C\""' || fail "the start card does not name the key of the cube"
ctl query id:construct-hint-cube | grep -q construct-hint-cube && fail "a hint bar shows with the start card"
ok "no chip OPEN CUBE, and the start card names the key of the cube"
opened=$(call cube.open) || fail "cube.open without params: $opened"
until_ok '! ctl query id:construct-hint-cube | grep -q construct-hint-cube' || fail "the hint of the cube stays with a cube"
ok "cube.open without params opens four faces, and no hint of the cube shows"
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
front_face0_label() { prop construct-face-label-words text | grep -q "face 1 of 4"; }
front_is() { test "$(prop "construct-cube-face-$1" visible)" = True && test "$(prop "construct-cube-face-$1" forward)" = True; }
alpha_of() { prop "$1" color | python3 -I -c 'import ast, sys; print(ast.literal_eval(sys.stdin.read())[3])'; }
stood_back='test "$(prop construct-cube-face-0 forward)" = False && test "$(width construct-cube-face-0)" -lt 1100 && test "$(width construct-cube-face-0)" -gt 300'
stepped_in='test "$(prop construct-cube-face-0 forward)" = True && test "$(width construct-cube-face-0)" = 1280'

call view.step_back >/dev/null || fail "view.step_back"
until_ok "$stood_back" 20 || fail "no step back: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
for chip in turn-left turn-right step-in; do
    until_ok 'test "$(prop "construct-mode-'"$chip"'" visible)" = True' || fail "the chip $chip does not show in navigate mode"
done
[ "$(prop construct-mode-step-back visible)" != True ] || fail "the chip STEP BACK shows in navigate mode"
before=$(keys_got 30)
plain a
until_ok 'test "$(keys_got 30)" -gt "$before"' 20 || fail "view.step_back, a method, took the keyboard from the window"
ok "view.step_back: the cube stands back, the face forwards nothing, three chips show, the keyboard stays"
until_ok 'test "$(prop construct-cube-face-1 visible)" = True' 20 || fail "stood back, the next face does not show: the cube rests square"
[ "$(prop construct-cube-face-3 visible)" = False ] || fail "stood back, the face before the front shows"
until_ok 'test "$(prop construct-cube-mirror-0 visible)" = True && test "$(prop construct-cube-mirror-1 visible)" = True' 20 || fail "stood back, the cube has no mirror image in the floor"
[ "$(prop construct-cube-mirror-3 visible)" = False ] || fail "the mirror image of a face that looks away shows"
shadow_width=$(prop construct-fog uniforms | python3 -I -c 'import ast, sys; print(ast.literal_eval(sys.stdin.read())["s0"][2])')
python3 -I -c 'import sys; sys.exit(not float(sys.argv[1]) > 0)' "$shadow_width" || fail "the cube has no shadow on the floor: s0 $shadow_width"
ok "stood back, the cube rests turned toward the next face, with its shadow and its mirror image in the floor"
label_at() { ctl query id:construct-face-label-glass | python3 -I -c 'import json, sys; r = json.load(sys.stdin)[0]["rects"]["HEADLESS-1"]; print("%.2f %.2f" % (r[0] + r[2] / 2, r[1]))'; }
face_middle() { ctl query "id:construct-cube-face-$1" | python3 -I -c 'import json, sys; r = json.load(sys.stdin)[0]["rects"]["HEADLESS-1"]; print("%.2f" % (r[0] + r[2] / 2))'; }
sleep 1
read -r label_x label_y <<<"$(label_at)"
python3 -I -c 'import sys; sys.exit(not abs(float(sys.argv[1]) - float(sys.argv[2])) <= 1.0)' "$label_x" "$(face_middle 0)" || fail "the face label is not under the middle of the bottom edge of the front face: label $label_x, face $(face_middle 0)"
ok "the face label hangs under the middle of the bottom edge of the face turned to rest, to the pixel"
call cube.turn '{"by": 1, "ms": 3000}' >/dev/null || fail "a slow cube.turn for the label"
sleep 1
read -r mid_x mid_y <<<"$(label_at)"
python3 -I -c 'import sys; sys.exit(not max(abs(float(sys.argv[1]) - float(sys.argv[3])), abs(float(sys.argv[2]) - float(sys.argv[4]))) > 2.0)' "$label_x" "$label_y" "$mid_x" "$mid_y" || fail "the face label stays at its place while the cube turns: $label_x $label_y, then $mid_x $mid_y"
until_ok 'prop construct-face-label-words text | grep -q "face 2 of 4"' 40 || fail "the slow turn did not end on face 2"
sleep 3
read -r end_x _ <<<"$(label_at)"
python3 -I -c 'import sys; sys.exit(not abs(float(sys.argv[1]) - float(sys.argv[2])) <= 1.0)' "$end_x" "$(face_middle 1)" || fail "after the turn the label is not under the face at the front: label $end_x, face $(face_middle 1)"
key Ctrl+Alt+Left
until_ok 'front_face0_label' 20 || fail "Ctrl+Alt+Left did not turn back after the label check"
sleep 1
ok "the face label follows the bottom edge of the cube while it turns and ends under the face at the front"
shadow_layers=$(prop construct-face-label-shadow uniforms | python3 -I -c '
import ast, sys
u = ast.literal_eval(sys.stdin.read())
print([u[k] for k in ("near", "mid", "far")])')
[ "$shadow_layers" = "[[1.0, 1.0, 0.06, 0.0], [4.0, 6.0, 0.08, 0.0], [0.0, 1.0, 0.0, 0.0]]" ] || fail "the face label capsule has no short shadow: $shadow_layers"
ok "the face label capsule takes the short shadow, 0 1 2 at 0.06 and 0 4 12 at 0.08, no grey smudge under it"
[ "$(edge construct-mode-step-in 1)" -gt 400 ] || fail "the chips are in the upper half, over the title band: y $(edge construct-mode-step-in 1)"
[ "$(edge construct-mode-step-in 3)" -ge 24 ] || fail "a chip is $(edge construct-mode-step-in 3) px high, less than 24"
[ $(($(edge construct-mode-step-in 1) + $(edge construct-mode-step-in 3))) -le $((800 - 32)) ] || fail "the chips leave the margin of 32 at the bottom"
ok "the chips are the hints of the cube at the bottom centre, at least 24 px high"
sleep 2
still || fail "a still cube draws: $(drawn)"
ok "a still cube renders 0 passes and has no damage, by render.stats"
call view.step_in >/dev/null || fail "view.step_in"
until_ok "$stepped_in" 20 || fail "view.step_in did not step in"
[ "$(prop construct-cube-face-1 visible)" = False ] || fail "after the step in the next face still shows: the face is not square"
[ "$(prop construct-cube-mirror-0 visible)" = False ] || fail "after the step in the mirror image of the cube shows"
prop construct-mode-step-back-text text | grep -q "Alt+Esc" || fail "the chip STEP BACK does not name its key: $(prop construct-mode-step-back-text text)"
covered_icons() {
    { ctl query id:construct-mode-step-back; ctl query "owner:xp kind:image"; } | python3 -I -c '
import json, sys
chip, images = [json.loads(part) for part in sys.stdin.read().replace("]\n[", "]\0[").split("\0")]
c = chip[0]["rects"]["HEADLESS-1"]
for n in images:
    r = n["rects"].get(sys.argv[1])
    if n["id"].startswith("xp-icon-") and r and r[0] < c[0] + c[2] and c[0] < r[0] + r[2] and r[1] < c[1] + c[3] and c[1] < r[1] + r[3]:
        print(n["id"])' "$front"
}
until_ok 'test -z "$(covered_icons)"' 20 || fail "the chip STEP BACK covers the desktop icons $(covered_icons)"
ok "the chip STEP BACK covers no desktop icon of the face"

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
until_ok 'test "$(prop construct-mode-step-in visible)" = True' || fail "the hint STEP IN does not show"
inject '[{"pointer_move": {"node": "construct-mode-step-in", "point": [8, 8]}}, {"click": {}}]'
until_ok "$stepped_in" 20 || fail "a click on the chip STEP IN did not step in"
ok "a click on a face and a click on the chip STEP IN step in"

[ "$(prop construct-cube-edge visible)" != True ] || fail "the edge of a face is lit in use mode"
key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back for the hover"
inject '[{"pointer_move": {"position": [10, 10]}}, {"pointer_move": {"position": [640, 400]}}]'
until_ok 'test "$(prop construct-cube-edge visible)" = True' 20 || fail "the edge of the face under the pointer is not lit: $(prop construct-cube-edge visible)"
sleep 1
edge_units=$(prop construct-cube-edge border | python3 -I -c 'import ast, sys; print(ast.literal_eval(sys.stdin.read())["width"])')
python3 -I -c '
import sys
width, face = float(sys.argv[1]), float(sys.argv[2])
sys.exit(not abs(width * face / 1280 - 2.0) < 0.3)' "$edge_units" "$(width construct-cube-face-0)" || fail "the lit edge is $edge_units units of the face wide on a face $(width construct-cube-face-0) px wide, not 2 px on the screen"
[ "$(prop construct-cube-edge-glow visible)" = True ] || fail "the lit edge has no glow"
[ "$(alpha_of construct-mode-step-in)" != 0.0 ] || fail "the hint Step in has no hover fill while the pointer is on the face"
ok "the pointer on a face lights its edge at 2 px on the screen with a glow, and the hint Step in has its hover fill"
inject '[{"pointer_move": {"position": [10, 10]}}]'
until_ok 'test "$(prop construct-cube-edge visible)" = False' 20 || fail "the edge stays lit after the pointer left the face"
until_ok 'test "$(prop construct-cube-edge-glow visible)" = False' 20 || fail "the glow stays after the pointer left the face"
until_ok 'test "$(alpha_of construct-mode-step-in)" = 0.0' 20 || fail "the hint Step in keeps its hover fill after the pointer left the face: $(alpha_of construct-mode-step-in)"
ok "the edge, its glow and the hover fill go out when the pointer leaves the face"
inject '[{"pointer_move": {"position": [640, 400]}}]'
until_ok 'test "$(prop construct-cube-edge visible)" = True' 20 || fail "the edge is not lit again"
call cube.turn '{"by": 1, "ms": 2500}' >/dev/null || fail "a slow cube.turn for the hover"
quick 'test "$(prop construct-cube-edge visible)" = False' || fail "the edge stays lit while the cube turns"
sleep 1
[ "$(prop construct-cube-edge visible)" = False ] || fail "the edge came back during a turn of the cube"
until_ok 'test "$(prop construct-cube-edge visible)" = True' 20 || fail "the edge does not light again when the turn ended with the pointer on the face"
key Ctrl+Alt+Left
until_ok 'front_face0_label' 20 || fail "Ctrl+Alt+Left did not turn back after the hover"
plain Return
until_ok "$stepped_in" 20 || fail "Return did not step in after the pointer lit an edge: the keyboard is lost to the pointer"
ok "no edge shows during a turn, and Return steps in after the hover"
key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back after the hover"
sleep 1

drag() { inject "[{\"pointer_move\": {\"position\": [$1, 400]}}, {\"button\": {\"state\": \"pressed\", \"hold\": true}}, {\"pointer_move\": {\"position\": [$(( ($1 + $2) / 2 )), 400]}}, {\"pointer_move\": {\"position\": [$2, 400]}}, {\"button\": {\"state\": \"released\"}}]"; }
press_at() { inject "[{\"pointer_move\": {\"position\": [$1, 400]}}, {\"button\": {\"state\": \"pressed\", \"hold\": true}}]"; }
move_to() { inject "[{\"pointer_move\": {\"position\": [$1, 400]}}]"; }
let_go() { inject '[{"button": {"state": "released"}}]'; }
drag 780 80
until_ok 'test "$(prop construct-cube-face-1 visible)" = True && test "$(prop construct-cube-face-0 visible)" = False' 20 || fail "a drag to the left did not turn the cube to the next face"
sleep 1
[ "$(prop construct-cube-face-1 forward)" = False ] || fail "a drag on the cube stepped in"
drag 500 1200
until_ok 'test "$(prop construct-cube-face-0 visible)" = True && prop construct-face-label-words text | grep -q "face 1 of 4"' 20 || fail "a drag to the right did not turn the cube back"
sleep 1
press_at 640
move_to 700
sleep 0.4
let_go
sleep 1
eval "$stood_back" || fail "a short drag turned the cube or left navigate mode"
plain Return
until_ok "$stepped_in" 20 || fail "no step in after the drags"
ok "a drag of more than 45 degrees turns the cube by one face; a slow short drag returns and does not step in"

key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape did not step back for the live drag"
windows_before=$(ctl query kind:surface | jq length)
rest_width=$(width construct-cube-face-0)
navigating='test "$(prop construct-mode-step-in visible)" = True'
press_at 780
previous=$rest_width
for x in 680 580 480 380 280 180 80; do
    move_to "$x"
    sleep 0.25
    seen=$(width construct-cube-face-0)
    [ "$seen" -lt "$previous" ] || fail "the cube did not follow the hand at x $x: face 0 is $seen px wide, was $previous"
    previous=$seen
    case $x in
    380) prop construct-face-label-words text | grep -q "face 1 of 4" || fail "at 28 degrees the label names $(prop construct-face-label-words text)" ;;
    180)
        prop construct-face-label-words text | grep -q "face 1 of 4" || fail "at 42 degrees the label names $(prop construct-face-label-words text)"
        fill=$(alpha_of construct-face-label-dot-1)
        python3 -I -c 'import sys; sys.exit(not 0.45 < float(sys.argv[1]) < 0.9)' "$fill" || fail "the next dot is not partly filled at 42 degrees: alpha $fill"
        [ "$(prop construct-mode-step-in-text opacity)" = 0.45 ] || fail "Step in is not dimmed while the cube turns: $(prop construct-mode-step-in-text opacity)"
        ;;
    esac
done
until_ok 'prop construct-face-label-words text | grep -q "face 2 of 4"' 8 || fail "at 49 degrees the label does not name the next face: $(prop construct-face-label-words text)"
ok "a slow drag turns the cube with the hand at every step, the label changes at 45 degrees and Step in dims"
let_go
released=$(now_ms)
quick 'test "$(( $(width construct-cube-face-1) - rest_width ))" -ge -3 && test "$(( $(width construct-cube-face-1) - rest_width ))" -le 3'
settled_in=$(($(now_ms) - released))
[ "$settled_in" -le 700 ] || fail "the cube took $settled_in ms to settle after a release at 49 degrees"
front_is_next=$(prop construct-cube-face-1 visible)
[ "$front_is_next" = True ] || fail "a release at 49 degrees did not go on to the next face"
until_ok 'test "$(prop construct-mode-step-in-text opacity)" != 0.45' 8 || fail "Step in stays dim after the settle"
ok "a release at 49 degrees goes on to the next face ($settled_in ms with the calls)"
press_at 780
move_to 580
sleep 0.2
move_to 353
sleep 0.4
let_go
quick 'test "$(( $(width construct-cube-face-1) - rest_width ))" -ge -3 && test "$(( $(width construct-cube-face-1) - rest_width ))" -le 3'
prop construct-face-label-words text | grep -q "face 2 of 4" || fail "a release at 30 degrees did not return to face 2 of 4: $(prop construct-face-label-words text)"
ok "a release at 30 degrees returns to the face of before"
press_at 740
inject '[{"pointer_move": {"position": [680, 400]}}, {"pointer_move": {"position": [560, 400]}}, {"pointer_move": {"position": [440, 400]}}, {"button": {"state": "released"}}]'
until_ok 'prop construct-face-label-words text | grep -q "face 3 of 4" && test "$(prop construct-cube-face-1 visible)" = False' 20 || fail "a flick did not go on to the next face: $(prop construct-face-label-words text), face 1 $(prop construct-cube-face-1 visible)"
ok "a flick of 21 degrees goes on to the next face"
plain Return
until_ok 'front_is 2' 20 || fail "Return did not step into the face that the drag turned to"
key Alt+Escape
until_ok "$navigating" 20 || fail "Alt+Escape did not step back after the drags"
turn_start=$(now_ms)
key Ctrl+Alt+Right
quick 'test "$(prop construct-cube-face-3 visible)" = True && test "$(prop construct-cube-face-2 visible)" = False'
turn_took=$(($(now_ms) - turn_start))
[ "$turn_took" -le 700 ] || fail "a turn key took $turn_took ms"
plain Return
until_ok 'front_is 3' 20 || fail "Return did not step into the face after the turn key"
ok "after the drags and a turn key, Return steps into the face that is at the front"
key Alt+Escape
until_ok "$navigating" 20 || fail "Alt+Escape did not step back at the end of the drags"
[ "$(ctl query kind:surface | jq length)" = "$windows_before" ] || fail "a window is lost or left by the drags: $windows_before before, $(ctl query kind:surface | jq length) after"
ok "window audit: no window lost or left by the drags"
key Ctrl+Alt+Right
until_ok 'test "$(prop construct-cube-face-0 visible)" = True && test "$(prop construct-cube-face-3 visible)" = False' 20 || fail "the turn key did not bring face 0 back to the front"
plain Return
until_ok "$stepped_in" 20 || fail "no step in at the end of the drags"
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
plain Return
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
    [ "$lower_lift" -gt 900 ] || { echo "the lower window is not lifted: $lower_lift per mille of its flat width"; return 1; }
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

frames() { ctl render-stats | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["outputs"]["HEADLESS-1"]["damaged_frames"])'; }
turn_frames=$(frames)
turn_start=$(date +%s%N)
call cube.turn '{"by": 1, "ms": 3000}' >/dev/null || fail "a slow cube.turn"
sleep 0.4
[ "$(width construct-cube-face-0)" -gt 0 ] && [ "$(width construct-cube-face-1)" -gt 0 ] ||
    fail "at a third of the turn the screen does not show two faces: widths $(width construct-cube-face-0) and $(width construct-cube-face-1)"
until_ok '[ "$(prop "construct-cube-float-0-$surface" visible)" != gone ]' 6 || fail "the window does not float during a turn"
[ -n "${SHOT:-}" ] && ctl capture "$SHOT" >/dev/null
[ "$(prop construct-cube-face-0 forward)" = False ] && [ "$(prop construct-cube-face-1 forward)" = False ] ||
    fail "a face forwards during the turn: $(prop construct-cube-face-0 forward) $(prop construct-cube-face-1 forward)"
call cube.turn '{"by": 1}' >/dev/null || fail "cube.turn during a turn"
until_ok 'test "$(prop construct-cube-face-1 visible)" = True && test "$(prop construct-cube-face-0 visible)" = False && test "$(prop construct-cube-face-1 forward)" = True' 40 ||
    fail "the cube did not turn: face 0 $(prop construct-cube-face-0 visible), face 1 $(prop construct-cube-face-1 visible) $(prop construct-cube-face-1 forward)"
turn_ms=$(( ($(date +%s%N) - turn_start) / 1000000 ))
turn_frames=$(( $(frames) - turn_frames ))
[ "$turn_frames" -gt 0 ] || fail "the screen drew no frame during the turn"
echo "frame time: $turn_frames frames of the screen in $turn_ms ms of the turn, $(( turn_ms / turn_frames )) ms a frame"
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

printf '#!/bin/sh\nexec sleep 60\n' >"$work/chora-tiling"
chmod +x "$work/chora-tiling"
call settings.set "{\"cube_manager\": \"$work/chora-tiling\"}" >/dev/null || fail "settings.set cube_manager"
call cube.open '{}' >/dev/null || fail "cube.open with a manager that never connects"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "no outputs for the waiting cube"
until_ok 'prop construct-face-card-head text | grep -q "^Tiling did not start"' 40 || fail "a manager that never connected says nothing on its face: $(prop construct-face-card-head text)"
prop construct-face-card-reason text | grep -q "did not connect within 5 s" || fail "the card gives no reason: $(prop construct-face-card-reason text)"
[ "$(virtuals | wc -w)" = 4 ] || fail "the outputs went with the manager that never connected: $(virtuals)"
ok "a window manager that never connected: the face says it did not start, with the reason, and the outputs stay"
call cube.close >/dev/null || fail "cube.close after a manager that never connected"
until_ok 'test -z "$(virtuals)"' || fail "cube.close left $(virtuals)"
call cube.open '{}' >/dev/null || fail "cube.open again"
call cube.close >/dev/null || fail "cube.close"
until_ok 'test -z "$(virtuals)"' || fail "cube.close left $(virtuals)"
ok "cube.close removed the cube and its outputs"
lit_frame() {
    ctl capture "$1" >/dev/null || return 1
    python3 -I - "$1" <<'PYEND'
import sys
from PIL import Image
image = Image.open(sys.argv[1]).convert("RGB")
w, h = image.size
points = [(w // 16, h // 16), (w // 16, h - h // 16), (w - w // 16, h // 16), (w - w // 16, h - h // 16), (w // 8, h // 2)]
dark = [p for p in points if sum(image.getpixel(p)) < 90]
sys.exit(1 if len(dark) >= 3 else 0)
PYEND
}
surfaces_before=$(ctl query kind:surface | jq length)
starts=$work/mc-starts
: >"$starts"
cat >"$work/chora-mc" <<SH
#!/bin/sh
echo start >>"$starts"
if [ "\$(wc -l <"$starts")" -ge 3 ]; then exec "$bin/chora-xp" "\$@"; fi
echo "it needs foot 1.26 or later." >&2
sleep 2
exit 1
SH
chmod +x "$work/chora-mc"
call settings.set "{\"cube_manager\": \"$work/chora-mc\", \"theme\": \"light\"}" >/dev/null || fail "settings.set cube_manager to a manager that fails"
started=$(now_ms)
call cube.open '{}' >/dev/null || fail "cube.open with a manager that fails"
quick 'prop construct-face-card-head text | grep -q "^Starting Media center$"' || fail "no card Starting Media center on the face: $(prop construct-face-card-head text)"
shown_in=$(($(now_ms) - started))
[ "$shown_in" -le 300 ] || fail "the card Starting Media center took $shown_in ms, more than 300"
ok "a face whose manager starts shows Starting Media center within 300 ms ($shown_in ms)"
lit_frame "$work/starting.png" || fail "a black frame while the manager starts"
[ -z "${STARTING_SHOT:-}" ] || cp "$work/starting.png" "$STARTING_SHOT"
for _ in $(seq 12); do
    lit_frame "$work/clip.png" || fail "a black frame while the manager starts or fails"
    sleep 0.2
done
until_ok 'prop construct-face-card-head text | grep -q "^Media center did not start\.$"' 40 || fail "the face never says it did not start: $(prop construct-face-card-head text)"
prop construct-face-card-reason text | grep -q "^chora-mc stopped after 2 s: it needs foot 1.26 or later\.$" || fail "the reason on the card: $(prop construct-face-card-reason text)"
lit_frame "$work/failed.png" || fail "a black frame once the manager failed"
ok "a manager that exits: the card says Media center did not start, with the reason, and no frame is black"
call view.step_back >/dev/null || fail "view.step_back with a failed face"
until_ok "$stood_back" 20 || fail "no step back with a failed face"
until_ok 'prop construct-face-label-words text | grep -q "did not start"' 20 || fail "the face label does not say did not start: $(prop construct-face-label-words text)"
[ "$(prop construct-face-label-name text)" = "Media center" ] || fail "the face label names $(prop construct-face-label-name text)"
[ "$(prop construct-face-label-dot-0 color)" = "[0.0, 0.0, 0.0, 0.0]" ] || fail "the dot of the failed face is not hollow: $(prop construct-face-label-dot-0 color)"
until_ok 'prop construct-mode-step-in-text text | grep -q "Try again"' 20 || fail "the middle hint does not say Try again: $(prop construct-mode-step-in-text text)"
[ -z "${FAILED_SHOT:-}" ] || ctl capture "$FAILED_SHOT" >/dev/null
ok "stood back, the label says did not start, its dot is hollow and the middle hint says Try again"
plain Return
until_ok 'test "$(wc -l <"$starts")" = 2' 20 || fail "Return did not start the manager again: $(wc -l <"$starts") starts"
quick 'prop construct-face-card-head text | grep -q "^Starting Media center$"' || fail "no card Starting after Return: $(prop construct-face-card-head text)"
until_ok 'prop construct-face-card-head text | grep -q "^Media center did not start\.$"' 40 || fail "the second start did not fail on the face: $(prop construct-face-card-head text)"
ok "Return tries again, and the second failure shows on the face again"
call view.step_in >/dev/null || fail "view.step_in with a failed face"
until_ok 'test "$(width construct-cube-face-0)" = 1280' 20 || fail "no step in with a failed face: width $(width construct-cube-face-0)"
sleep 0.5
inject '[{"pointer_move": {"position": [300, 200]}}, {"click": {}}]'
until_ok 'test "$(wc -l <"$starts")" = 3' 20 || fail "a click on the face did not start the manager again: $(wc -l <"$starts") starts"
until_ok 'test "$(bars)" = 4' 80 || fail "no xp taskbars after the third start: $(bars)"
until_ok 'test "$(prop construct-face-card opacity)" = 0.0 || test "$(prop construct-face-card visible)" = gone' 20 || fail "the card stays after the manager came up: $(prop construct-face-card opacity)"
ok "a click on a failed face tries again, and the face works when the manager is up"
key Alt+Escape
until_ok "$stood_back" 20 || fail "Alt+Escape does not step back after the retry"
plain Return
until_ok "$stepped_in" 20 || fail "Return does not step in after the retry"
ok "after the retry the keys of the cube work again"
call cube.close >/dev/null || fail "cube.close after the retry"
until_ok 'test -z "$(virtuals)"' || fail "cube.close left $(virtuals)"
[ "$(ctl query kind:surface | jq length)" = "$surfaces_before" ] || fail "a window is lost or left: $surfaces_before surfaces before, $(ctl query kind:surface | jq length) after"
ok "window audit: no window lost or left by the failed faces"
call settings.set '{"cube_manager": "chora-xp"}' >/dev/null || fail "settings.set cube_manager back"
key Super+c
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "Super+C opened no cube: $(virtuals)"
ok "Super+C opens the cube"
call cube.close >/dev/null || fail "cube.close after the key"
until_ok 'test -z "$(virtuals)"' || fail "cube.close after the key left $(virtuals)"

call cube.open '{}' >/dev/null || fail "cube.open before the restart"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "no outputs before the restart"
construct=$(pid_of chora-construct)
kill -9 "$construct"
until_ok 'test -z "$(pid_of chora-xp)"' 40 || fail "the xp of the cube outlived the construct that died: $(pid_of chora-xp)"
ok "the xp of the cube ends with a construct that dies without a word"
"$bin/chora-construct" >"$work/construct2.log" 2>&1 &
pids+=($!)
until_ok 'test -z "$(virtuals)"' 16 || fail "the restarted construct left its outputs: $(virtuals)"
ok "a restarted construct removed the outputs that it left"
