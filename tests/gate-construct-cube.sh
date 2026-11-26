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
#    In use mode Escape reaches the window, Super+d of the xp of the
#    faces puts the window away and back, the turn keys turn the cube,
#    and a turn key during a turn queues one turn. Alt+Escape steps back
#    and takes the keyboard; Return steps in and the window of before
#    takes typed text again, but not after a step into another face. A click on a face and a click on the chip
#    STEP IN step in, and so does a click during the step back. In navigate mode the edge of the face under the
#    pointer is lit, and a drag on a face turns the cube by one face. A drag
#    that begins beside the cube leaves it where it is.
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
# 2c. A window that a face starts while the key sheet is open or from
#    the launcher (the terminal and the launcher of the gate are chora-probe
#    windows that report their keys): Super+t with the key sheet open over
#    a face starts a window of xp on the front face, no pane of the
#    construct (rcr1-1); Escape closes the sheet and that window has the
#    keyboard again (co-78). A program of the launcher in a face that
#    holds a window is a window of xp, shown on that face, and takes
#    keys, in 5 of 5 runs (co-77). After the windows of the programs end, a
#    click on the first window gives it the keyboard, also when xp answers
#    1.5 s late (rfs2-2): a key typed before the focus comes is lost, so the
#    gate waits for the focus.
# 3. A slow cube.turn forwards nothing while it runs; at a third of it
#    the screen shows two faces; the gate prints the frame time of the
#    turn; then the next face
#    is at the front, and the face before looks away and hides. xp moves
#    the window to the output of the next face.
# 4. When xp ends, the construct removes the outputs and the cube, and
#    the probe window is not lost: a screen shows it, or a client other
#    than the ended xp owns it.
# 5. A window manager that never connects: the outputs go after the
#    wait. cube.close removes a cube, and Super+C opens a cube, stood
#    back; the next cube.open waits until the core lets the ended xp go.
# 6. A construct that is killed (C2) leaves the xp of the cube flat on
#    the screen with every window within 3 s. A construct that starts
#    again reopens the cube around that xp, with the windows on its faces.
# 7. The way into a face out of a pane (co-79): the cube stands back by
#    Super+C, the launcher starts a program beside it and the camera flies
#    in; Alt+Escape steps out of it, and the cube turns to face 1. Stood
#    back, the hint bar names the give key of the face manager, Super Alt
#    2, and that key puts the window on the face at the front, steps into
#    it and gives it the keyboard; then the bar names the pull key.
# 8. Out of every pane a told key leads into a face (co-79 r6): the
#    Terminal row of the start card starts a pane with no cube open;
#    Alt+Escape steps out into the space, where the bar names Super C Cube;
#    that key stands back from a new cube, where the bar names Super Alt 2
#    for the window of the pane, which then is on face 0 and takes keys.
#    Pulled out of the face again by Super+Alt+4, Alt+Escape out of that
#    pane stands back from the cube too, and Super Alt 2 is told again.
# 9. The key sheet in a pane is honest (rpc2-1, rpc2-2): with the cube
#    stood back, a program of the launcher flown into, a click on All keys
#    lists only keys that work with a window holding the keyboard: no
#    Return Fly in, no Step into the face, no Esc; Escape closes it, and a
#    click on Step back names Super Alt 2 as it does without the sheet.
#
# Usage: tests/gate-construct-cube.sh   BIN: directory with the binaries
#   SHOT  a PNG path that keeps a capture of the cube during a turn
#   SHEET_SHOT, BACK_SHOT  PNG paths that keep a capture of the key list in a
#         pane (step 9) and of the stood-back bar after it
#   INJECT_LAG  seconds before each injection (default 0): the start of a
#         choractl on a loaded machine, the stress of the keys in a row
#   CUBE_LOAD   1 starts 4 busy shell loops per CPU for the run (a loaded host): the
#         cube must settle in time with the shell and ctl calls slowed down
#   CLOCK_EDGE  seconds before a minute change at which a still check starts
#         (default: at once): the clocks of the construct and of xp draw at
#         the change, the stress of a check across a minute
# pid_of NAME gives the first process of the binary NAME in this session.
# The xp of the cube outlives its core by 30 s, so the EXIT trap ends it with the gate (merge-gate fails a step that leaves a chora process).
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
settle_poll() {
    local began=$1 poll_start
    settle_min=0
    while :; do
        poll_start=$(now_ms)
        eval "$2" && break
        settle_min=$((poll_start - began))
        [ $((poll_start - began)) -le 5000 ] || return 1
        sleep 0.02
    done
    settle_max=$(($(now_ms) - began))
}
call() { ctl call construct "$@"; }
inject() { sleep "${INJECT_LAG:-0}"; echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
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

mkdir -p "$work/chora"
cat >"$work/launch.sh" <<'LAUNCH'
#!/bin/sh
# a program of the launcher, no child of it, as fuzzel starts one
n=$(date +%s%N)
(setsid chora-probe window 40 200 80 --report >"$1/prog.$n.log" 2>&1 &)
LAUNCH
cat >"$work/chora/chora-construct.yaml" <<CONFIG
terminal: exec chora-probe window 200 80 40 --report >$work/term.log 2>&1
launcher: sh $work/launch.sh $work
CONFIG
session_start 1280x800 -- "$bin/chora-construct"
if [ "${CUBE_LOAD:-0}" = 1 ]; then
    for _ in $(seq $((4 * $(nproc)))); do
        (while :; do :; done) &
        pids+=($!)
    done
fi
end_cube_xp() {
    local xp
    xp=$(pid_of chora-xp)
    [ -z "$xp" ] || kill "$xp" 2>/dev/null
}
trap 'status=$?; end_cube_xp; gate_cleanup $status' EXIT
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
first_probe=$!
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
scene_dump() { ctl query '*' >"$1" 2>/dev/null || echo '[]' >"$1"; }
drew_what() {
    python3 -I - "$work/still-before.json" "$work/still-after.json" <<'PY'
import json, sys
def load(path):
    try:
        return {n["id"]: n for n in json.load(open(path))}
    except (OSError, ValueError):
        return {}
before, after = load(sys.argv[1]), load(sys.argv[2])
lines = []
for id_ in sorted(set(before) | set(after)):
    a, b = before.get(id_), after.get(id_)
    if a is None or b is None:
        lines.append(f"{id_}: {'new' if a is None else 'gone'}")
        continue
    keys = [k for k in sorted(set(a) | set(b)) if a.get(k) != b.get(k)]
    for k in keys:
        lines.append(f"{id_}.{k}: {json.dumps(a.get(k))[:60]} -> {json.dumps(b.get(k))[:60]}")
print("; ".join(lines[:12]) if lines else f"no node changed in {len(after)} nodes")
PY
}
clock_clear() {
    local second
    second=$((10#$(date +%S)))
    [ "$second" -ge 6 ] && [ "$second" -le 52 ]
}
still() {
    local before after at
    STILL_WHY=""
    [ -z "${CLOCK_EDGE:-}" ] || until_ok '[ "$((10#$(date +%S)))" = "$((60 - CLOCK_EDGE))" ]' 250
    for _ in 1 2; do
        until_ok 'clock_clear' 240 || { STILL_WHY="no second 6 to 52 of a minute came"; return 1; }
        before=$(drawn)
        at=$(date +%T)
        scene_dump "$work/still-before.json"
        sleep 2
        scene_dump "$work/still-after.json"
        after=$(drawn)
        [ "$before" != "$after" ] || return 0
        STILL_WHY="$STILL_WHY [window $at: $before -> $after; drew: $(drew_what)]"
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

help_shown() { ctl query id:construct-help | python3 -I -c 'import json, sys; n = json.load(sys.stdin); sys.exit(0 if n and n[0].get("opacity") == 1.0 else 1)'; }
before=$(keys_got 30)
plain a
until_ok 'test "$(keys_got 30)" -gt "$before"' 20 || fail "on the front face a plain a does not reach the probe window"
sleep 2
for chord in Super+q Super+Tab Super+1 Super+Shift+x Super+Ctrl+z Super+Alt+9 Super+Alt+x Super+Ctrl+Shift+x Super+Alt+Shift+x Super+Ctrl+Alt+x; do
    key "$chord"
done
sleep 1
reached=0
for code in 16 15 2 45 44 10; do
    [ "$(keys_got "$code")" = 0 ] || reached=1
done
start_menu_open() { ctl query id:xp-item-0 | python3 -I -c 'import json, sys; n = json.load(sys.stdin); sys.exit(0 if n and n[0]["rects"] else 1)'; }
if start_menu_open; then
    plain Escape
    until_ok '! start_menu_open' 20 || fail "Escape does not close the Start menu that the release of Super opened"
fi
before=$(keys_got 30)
plain a
until_ok 'test "$(keys_got 30)" -gt "$before"' 20 || fail "after the swallowed chords the probe window has no keyboard"
if [ "$reached" = 0 ]; then
    ok "co-25: on the front face Super+q, Tab, 1, Shift+x, Ctrl+z, Alt+9, Alt+x, Ctrl+Shift+x, Alt+Shift+x and Ctrl+Alt+x reach no window, a plain a does"
else
    fail "co-25: on the front face a Super chord that nothing binds reaches the probe window, as the face manager swallows none"
fi
key Super+F1
if until_ok help_shown 12; then
    key Super+F1
    until_ok '! help_shown' || fail "a second Super+F1 on the front face keeps the key list"
    ok "co-26: Super+F1 on the front face shows the key list of the construct, and a second one hides it"
else
    fail "co-26: Super+F1 on the front face shows no key list of the construct"
fi
key Super+F1
until_ok help_shown 12 || fail "co-71: Super+F1 on the front face shows no key list to close"
inject '[{"pointer_move": {"node": "construct-help-body-foot-0-hit", "point": [8, 8]}}, {"click": {}}]'
until_ok '! help_shown' 20 || fail "co-71: a click on the Close of the key list opened from a face keeps it"
key Super+F1
until_ok help_shown 12 || fail "co-71: Super+F1 shows no key list to close from outside"
inject '[{"pointer_move": {"position": [20, 400]}}, {"click": {}}]'
until_ok '! help_shown' 20 || fail "co-71: a click outside the key list opened from a face keeps it"
key Super+F1
until_ok help_shown 12 || fail "co-71: Super+F1 shows no key list to close by Escape"
plain Escape
until_ok '! help_shown' 20 || fail "co-71: Escape keeps the key list opened from a face"
ok "co-71: the key list opened from a face closes by its Close, by a click outside and by Escape"
before=$(keys_got 30)
plain a
until_ok 'test "$(keys_got 30)" -gt "$before"' 20 || fail "after the key list the probe window has no keyboard"
until_ok 'ctl query id:construct-hint-pull-out | grep -q construct-hint-pull-out' 20 ||
    fail "co-79: in a face with the keyboard in its window the hint bar names no Super Alt 4 Pull out"
ok "co-79: in a face with the keyboard in its window the hint bar names the pull key"

others() {
    ctl query kind:surface | python3 -I -c '
import json, sys
for n in json.load(sys.stdin):
    if n["surface"].get("app_id") == "chora-probe" and n["id"] != sys.argv[1]:
        r = next(iter(n["rects"].values()), [0, 0, 0, 0])
        print(n["id"], n.get("owner"), next(iter(n["rects"]), "-"), round(r[0]), round(r[1]), round(r[2]), round(r[3]))' "$surface"
}
core_focus() { ctl query '*' | python3 -I -c 'import json, sys; print(",".join(n["id"] for n in json.load(sys.stdin) if n.get("focused")) or "none")'; }
stall_for() { until_ok false "$1" || :; kill -CONT "$2"; }
end_others() {
    local xp_pid
    if [ "${1:-}" = stalled ]; then
        xp_pid=$(pid_of chora-xp)
        kill -STOP "$xp_pid"
        stall_for 6 "$xp_pid" &
        pids+=($!)
    fi
    python3 -I - "$CHORA_SOCKET" "$first_probe" <<'PY'
import os, signal, sys
for pid in sorted(filter(str.isdigit, os.listdir("/proc")), key=int):
    try:
        cmd = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
        env = open(f"/proc/{pid}/environ", "rb").read().split(b"\0")
    except OSError:
        continue
    if cmd[0].endswith(b"chora-probe") and b"CHORA_SOCKET=" + sys.argv[1].encode() in env and pid != sys.argv[2]:
        os.kill(int(pid), signal.SIGTERM)
PY
    until_ok 'test -z "$(others)"' 40 || fail "a probe window of a program of the face stays after its end: $(others)"
    inject "[{\"pointer_move\": {\"position\": [$((x + 3)), $((y + 2))]}}, {\"click\": {}}]"
    until_ok 'test "$(prop "$surface" focused)" = True' 40 || fail "after the end of a program of the face the click gave the first probe window no focus: focused $(core_focus)"
    before=$(keys_got 30)
    plain a
    until_ok 'test "$(keys_got 30)" -gt "$before"' 20 || fail "after the end of a program of the face the first probe window has no keyboard: focused $(core_focus), surfaces $(probe | tr '\n' ';')"
}
xp_shows() { ctl state xp | python3 -I -c 'import json, sys; w = [w for w in json.load(sys.stdin)["windows"] if w["surface"] == sys.argv[1]]; sys.exit(0 if w and w[0]["shown"] else 1)' "$1"; }
keys_in() { grep -c '^key 30$' "$1"; }

key Super+F1
until_ok help_shown 12 || fail "co-78: Super+F1 shows no key sheet over the face"
key Super+t
until_ok 'test -n "$(others)"' 80 || fail "rcr1-1: Super+t with the key sheet open started no window"
until_ok 'others | grep -q " xp VIRTUAL"' 40 || fail "rcr1-1: Super+t with the key sheet open over a face started no window of the face manager xp: $(others)"
read -r second owner out wx wy ww wh <<<"$(others)"
[ "$out" = "$front" ] || fail "rcr1-1: the window of Super+t is on $out, not on the face $front"
until_ok 'xp_shows "$second"' 20 || fail "rcr1-1: the face manager does not show the window $second"
ok "rcr1-1: Super+t with the key sheet open over a face starts a window of the face (xp, $out)"
help_shown || fail "co-78: the key sheet closed by itself when the window opened"
before=$(keys_in "$work/term.log")
plain a
! until_ok 'test "$(keys_in "$work/term.log")" -gt "$before"' 8 || fail "rcm1-2: a key reached the window while the key sheet was open"
plain Escape
if help_shown; then plain Escape; fi
until_ok '! help_shown' 20 || fail "co-78: Escape keeps the key sheet after Super+t started a window in the face"
before=$(keys_in "$work/term.log")
plain a
until_ok 'test "$(keys_in "$work/term.log")" -gt "$before"' 20 || fail "co-78: after Escape the window that Super+t started has no keyboard"
ok "co-78: Escape closes the key sheet after Super+t opened a window of the face, and that window has the keyboard"
end_others stalled

for run in 1 2 3 4 5; do
    key Super+space
    until_ok 'test -n "$(others)"' 100 || fail "co-77: run $run: the program of the launcher opened no window in the face"
    until_ok 'others | grep -q " xp VIRTUAL"' 40 || fail "co-77: run $run: the program of the launcher in a face is held by none of xp: $(others)"
    read -r second owner out wx wy ww wh <<<"$(others)"
    [ "$out" = "$front" ] || fail "co-77: run $run: the program is on $out, not on the face $front"
    until_ok 'xp_shows "$second"' 20 || fail "co-77: run $run: the face manager does not show the window $second"
    { [ "$wx" -ge 0 ] && [ "$wy" -ge 0 ] && [ $((wx + ww)) -le 1280 ] && [ $((wy + wh)) -le 800 ]; } || fail "co-77: run $run: the window lies outside the face: $wx,$wy ${ww}x$wh"
    log=$(find "$work" -name "prog.*.log" -printf "%T@ %p\n" | sort -n | tail -n 1 | cut -d" " -f2)
    before=$(keys_in "$log")
    plain a
    until_ok 'test "$(keys_in "$log")" -gt "$before"' 20 || fail "co-77: run $run: keys do not reach the program of the launcher"
    end_others
done
ok "co-77: a program of the launcher in a face that holds a window is shown on that face, held by the face manager xp, and takes keys, 5 of 5 runs"

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
key Super+Tab
sleep 1
stood_back_reached=$(keys_got 15)
if start_menu_open; then
    plain Escape
    until_ok '! start_menu_open' 20 || fail "stood back, Escape does not close the Start menu"
fi
if [ "$stood_back_reached" = 0 ]; then
    ok "co-25: stood back, Super+Tab reaches no window"
else
    fail "co-25: stood back, Super+Tab reaches the probe window, as the face manager swallows none"
fi
until_ok 'test "$(prop construct-cube-face-1 visible)" = True' 20 || fail "stood back, the next face does not show: the cube rests square"
[ "$(prop construct-cube-face-3 visible)" = False ] || fail "stood back, the face before the front shows"
until_ok 'test "$(prop construct-cube-mirror-0 visible)" = True && test "$(prop construct-cube-mirror-1 visible)" = True' 20 || fail "stood back, the cube has no mirror image in the floor"
[ "$(prop construct-cube-mirror-3 visible)" = False ] || fail "the mirror image of a face that looks away shows"
shadow_width=$(prop construct-fog uniforms | python3 -I -c 'import ast, sys; print(ast.literal_eval(sys.stdin.read())["s0"][2])')
python3 -I -c 'import sys; sys.exit(not float(sys.argv[1]) > 0)' "$shadow_width" || fail "the cube has no shadow on the floor: s0 $shadow_width"
ok "stood back, the cube rests turned toward the next face, with its shadow and its mirror image in the floor"
label_at() { ctl query id:construct-face-label-glass | python3 -I -c 'import json, sys; r = json.load(sys.stdin)[0]["rects"]["HEADLESS-1"]; print("%.2f %.2f" % (r[0] + r[2] / 2, r[1]))'; }
lowest() { ctl query "owner:construct" | python3 -I -c 'import json, sys; print("%.2f" % max(r[1] + r[3] for n in json.load(sys.stdin) if n["id"].startswith("construct-cube-face-") and n.get("visible") for r in [n["rects"]["HEADLESS-1"]]))'; }
sleep 1
read -r label_x label_y <<<"$(label_at)"
python3 -I -c 'import sys; sys.exit(not abs(float(sys.argv[1]) - 640.0) <= 1.0)' "$label_x" || fail "the face label is not on the view axis: x $label_x"
python3 -I -c 'import sys; sys.exit(not abs(float(sys.argv[1]) - float(sys.argv[2]) - 8.0) <= 1.0)' "$label_y" "$(lowest)" || fail "the face label is not 8 px under the lowest corner of the cube: label y $label_y, lowest $(lowest)"
cube_centre=$(ctl query "owner:construct" | python3 -I -c 'import json, sys; rs = [n["rects"]["HEADLESS-1"] for n in json.load(sys.stdin) if n["id"].startswith("construct-cube-face-") and n.get("visible")]; print("%.2f" % ((min(r[0] for r in rs) + max(r[0] + r[2] for r in rs)) / 2))')
python3 -I -c 'import sys; sys.exit(not abs(float(sys.argv[1]) - 640.0) <= 1.0)' "$cube_centre" || fail "the cube is not centred on the view axis: centre $cube_centre"
ok "the cube stood back is centred on the view axis, and the face label hangs on the axis 8 px under its lowest corner"
call cube.turn '{"by": 1, "ms": 3000}' >/dev/null || fail "a slow cube.turn for the label"
sleep 1
read -r mid_x mid_y <<<"$(label_at)"
python3 -I -c 'import sys; sys.exit(not max(abs(float(sys.argv[1]) - float(sys.argv[3])), abs(float(sys.argv[2]) - float(sys.argv[4]))) > 2.0)' "$label_x" "$label_y" "$mid_x" "$mid_y" || fail "the face label stays at its place while the cube turns: $label_x $label_y, then $mid_x $mid_y"
until_ok 'prop construct-face-label-words text | grep -q "face 2 of 4"' 40 || fail "the slow turn did not end on face 2"
sleep 3
read -r end_x end_y <<<"$(label_at)"
python3 -I -c 'import sys; sys.exit(not (abs(float(sys.argv[1]) - 640.0) <= 1.0 and abs(float(sys.argv[2]) - float(sys.argv[3]) - 8.0) <= 1.0))' "$end_x" "$end_y" "$(lowest)" || fail "after the turn the label is not on the axis 8 px under the cube: label $end_x $end_y, lowest $(lowest)"
key Ctrl+Alt+Left
until_ok 'front_face0_label' 20 || fail "Ctrl+Alt+Left did not turn back after the label check"
sleep 1
ok "the face label follows the lowest corner of the cube while it turns and ends on the axis under it"
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
still || fail "a still cube draws: $STILL_WHY"
ok "a still cube renders 0 passes and has no damage, by render.stats"
call view.step_in >/dev/null || fail "view.step_in"
until_ok "$stepped_in" 20 || fail "view.step_in did not step in"
[ "$(prop construct-cube-face-1 visible)" = False ] || fail "after the step in the next face still shows: the face is not square"
[ "$(prop construct-cube-mirror-0 visible)" = False ] || fail "after the step in the mirror image of the cube shows"
step_keys() { local k out=""; for k in 0 1 2; do out="$out$(prop "construct-hints-step-back-keys-$k-text" text | grep -v -e False -e gone)"; done; echo "$out"; }
until_ok 'test "$(prop construct-hint-step-back visible)" = True' 20 || fail "no hint Step back in use mode: $(prop construct-hint-step-back visible)"
[ "$(prop construct-mode-step-back visible)" != True ] || fail "the corner chip STEP BACK shows in use mode: v2 has none"
[ "$(prop construct-hints-step-back-word text)" = "Step back" ] || fail "the hint Step back has the word $(prop construct-hints-step-back-word text)"
[ "$(step_keys)" = "AltEsc" ] || fail "the hint Step back names the keys $(step_keys)"
ok "in use mode no chip covers the face; the hint Alt Esc Step back shows at the bottom centre"

before=$(keys_got 1)
plain Escape
until_ok 'test "$(keys_got 1)" -gt "$before"' 20 || fail "Escape in use mode did not reach the window"
sleep 0.5
eval "$stepped_in" || fail "Escape in use mode moved the view"
ok "in use mode Escape reaches the window on the face, and the view stays"

key Super+d
until_ok 'test "$(prop "xp-win-$surface" visible)" = False' 20 || fail "Super+d did not reach the xp of the faces, which let its cube keys go: visible $(prop "xp-win-$surface" visible)"
key Super+d
until_ok 'test "$(prop "xp-win-$surface" visible)" = True' 20 || fail "a second Super+d did not bring the window back: visible $(prop "xp-win-$surface" visible)"
before=$(keys_got 1)
plain Escape
until_ok 'test "$(keys_got 1)" -gt "$before"' 20 || fail "the window has no keyboard after Super+d twice"
ok "the xp of the faces let its cube keys go and keeps its own: Super+d puts the window away and brings it back, with the keyboard"

sleep 3
ctl call xp settings.set '{"bindings": {"Super+k": "desktop"}}' >/dev/null || fail "settings.set bindings Super+k of xp"
key Super+k
until_ok 'test "$(prop "xp-win-$surface" visible)" = False' 20 || fail "Super+k, bound by the xp of the faces 3 s after it connected, did not reach it: visible $(prop "xp-win-$surface" visible) (rv-1)"
key Super+k
until_ok 'test "$(prop "xp-win-$surface" visible)" = True' 20 || fail "a second Super+k did not bring the window back: visible $(prop "xp-win-$surface" visible)"
ok "rv-1: a chord that the xp of the faces binds late is its own, the construct holds none on a face"

key Ctrl+Alt+Right
until_ok 'front_is 1' 20 || fail "Ctrl+Alt+Right did not turn the cube in use mode"
key Ctrl+Alt+Left
until_ok 'front_is 0' 20 || fail "Ctrl+Alt+Left did not turn the cube back in use mode"
inject '[{"shortcut": {"keys": "Ctrl+Alt+Right"}}, {"shortcut": {"keys": "Ctrl+Alt+Right"}}, {"shortcut": {"keys": "Ctrl+Alt+Right"}}]'
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
[ "$(prop construct-mode-step-back visible)" != True ] || fail "the chip STEP BACK shows in use mode"
until_ok 'test "$(prop construct-hint-step-back visible)" = True' 20 || fail "the hint Step back does not show after the step in"
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
key Alt+Escape
quick 'test "$(prop construct-cube-face-0 forward)" = False' || fail "Alt+Escape did not start a step back"
inject '[{"pointer_move": {"position": [640, 400]}}, {"click": {}}]'
until_ok "$stepped_in" 20 || fail "a click during the step back is lost"
ok "a click on a face during the step back steps in"

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
sleep 2
rest_width=$(width construct-cube-face-0)
drag 1250 50
sleep 1
until_ok "$stood_back" 20 || fail "a drag that began beside the cube moved it out of the view: face 0 is $(width construct-cube-face-0) px wide, at rest $rest_width, the label says $(prop construct-face-label-words text)"
[ "$(width construct-cube-face-0)" = "$rest_width" ] || fail "a drag that began beside the cube moved it: face 0 is $(width construct-cube-face-0) px wide, at rest $rest_width"
ok "a drag that begins beside the cube leaves it where it is"
windows_before=$(ctl query kind:surface | jq length)
at_rest() {
    local seen
    seen=$(width construct-cube-face-0)
    for _ in 1 2 3; do
        sleep 0.25
        [ "$seen" = "$(width construct-cube-face-0)" ] || return 1
    done
}
until_ok at_rest 40 || fail "the cube did not come to rest after the step back: face 0 is $(width construct-cube-face-0) px wide"
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
        [ "$(prop construct-mode-step-in-text opacity)" = 0.6 ] || fail "Step in is not dimmed while the cube turns: $(prop construct-mode-step-in-text opacity)"
        ;;
    esac
done
until_ok 'prop construct-face-label-words text | grep -q "face 2 of 4"' 8 || fail "at 49 degrees the label does not name the next face: $(prop construct-face-label-words text)"
ok "a slow drag turns the cube with the hand at every step, the label changes at 45 degrees and Step in dims"
let_go
released=$(now_ms)
settle_poll "$released" 'test "$(( $(width construct-cube-face-1) - rest_width ))" -ge -3 && test "$(( $(width construct-cube-face-1) - rest_width ))" -le 3' || settle_min=hang
[ "$settle_min" != hang ] && [ "$settle_min" -le 700 ] || fail "the cube was still turning $settle_min ms after a release at 49 degrees: face 0 is $(width construct-cube-face-0) px wide, face 1 $(width construct-cube-face-1), at rest $rest_width, the label says $(prop construct-face-label-words text)"
front_is_next=$(prop construct-cube-face-1 visible)
[ "$front_is_next" = True ] || fail "a release at 49 degrees did not go on to the next face"
until_ok 'test "$(prop construct-mode-step-in-text opacity)" != 0.6' 8 || fail "Step in stays dim after the settle"
ok "a release at 49 degrees goes on to the next face (not settled at $settle_min ms, settled by $settle_max ms with the calls)"
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
settle_poll "$turn_start" 'test "$(prop construct-cube-face-3 visible)" = True && test "$(prop construct-cube-face-2 visible)" = False' || settle_min=hang
[ "$settle_min" != hang ] && [ "$settle_min" -le 700 ] || fail "a turn key left the cube unturned $settle_min ms after the key"
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
"$bin/choractl" grant construct input.grab-any || fail "the grant input.grab-any"
call settings.set '{"fly_out": "Escape"}' >/dev/null || fail "settings.set fly_out Escape with the grant"
sleep 0.5
plain Escape
until_ok "$stood_back" 20 || fail "Escape alone did not step back with input.grab-any"
plain Return
until_ok "$stepped_in" 20 || fail "Return did not step in after the step back by Escape"
until_ok 'test "$(step_keys)" = Esc' 20 || fail "the hint does not name Escape: $(step_keys)"
call settings.set '{"fly_out": "Alt+Escape"}' >/dev/null || fail "settings.set fly_out Alt+Escape"
ok "with input.grab-any and fly_out Escape, Escape alone steps back, Return steps in, and the hint names Escape"

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
hint_rests='test "$(prop construct-hint-step-back visible)" = True'
steady() { ctl capture "$work/$1.png" >/dev/null && ctl capture "$work/$1-again.png" >/dev/null && cmp -s "$work/$1.png" "$work/$1-again.png"; }
floats() {
    until_ok "$hint_rests" 40 || { echo "the face names no way out after the step in: $(prop construct-hint-step-back visible)"; return 1; }
    until_ok 'steady flat' 40 || { echo "the face does not come to rest after the step in"; return 1; }
    call view.step_back >/dev/null || return 1
    until_ok "$stood_back" 20 || return 1
    sleep 1
    [ "$(prop "construct-cube-float-0-$surface" visible)" = True ] && [ "$(prop "construct-cube-float-0-$upper" visible)" = True ] ||
        { echo "the planes do not show: $(prop "construct-cube-float-0-$surface" visible) $(prop "construct-cube-float-0-$upper" visible)"; return 1; }
    [ "$(opacity "$surface")" = 0 ] && [ "$(opacity "$upper")" = 0 ] || { echo "the windows stay in the face: opacity $(opacity "$surface") $(opacity "$upper")"; return 1; }
    lower_lift=$(lifted "$surface")
    upper_lift=$(lifted "$upper")
    [ "$lower_lift" -gt 800 ] || { echo "the lower window is not lifted: $lower_lift per mille of its flat width"; return 1; }
    [ "$upper_lift" -gt $((lower_lift + 5)) ] || { echo "the upper window is not lifted farther: $upper_lift, lower $lower_lift"; return 1; }
    [ -z "${FLOAT_SHOT:-}" ] || ctl capture "$FLOAT_SHOT" >/dev/null
    still || { echo "a still cube with floating windows draws: $STILL_WHY"; return 1; }
    call view.step_in >/dev/null || return 1
    until_ok "$stepped_in" 20 || return 1
    until_ok 'test "$(prop "construct-cube-float-0-$surface" visible)" = gone && test "$(opacity "$surface")" = 1 && test "$(opacity "$upper")" = 1' 20 ||
        { echo "the windows did not land: plane $(prop "construct-cube-float-0-$surface" visible), opacity $(opacity "$surface") $(opacity "$upper")"; return 1; }
    until_ok "$hint_rests" 40 || { echo "the face names no way out after the landing: $(prop construct-hint-step-back visible)"; return 1; }
    until_ok 'steady landed' 40 || { echo "the face does not come to rest after the landing"; return 1; }
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
deaf_answer=0
deaf_times=
for _ in 1 2 3; do
    deaf_start=$(now_ms)
    timeout 2 "$bin/choractl" --name agent call construct construct.list >/dev/null 2>&1 || deaf_answer=1
    deaf_times="$deaf_times $(($(now_ms) - deaf_start))"
done
kill -CONT "$manager"
[ "$deaf_answer" = 0 ] || fail "the construct does not answer within 2 s while the window manager is deaf: three calls took$deaf_times ms"
[ "$deaf_plane" = True ] || fail "the plane went although the window manager did not answer: the window shows nowhere ($deaf_plane)"
until_ok 'test "$(prop "construct-cube-float-0-$surface" visible)" = gone && test "$(opacity "$surface")" = 1' 40 ||
    fail "the windows did not land after the window manager answered again: plane $(prop "construct-cube-float-0-$surface" visible), opacity $(opacity "$surface")"
until_ok "$stepped_in" 20 || fail "no step in after the window manager answered again"
ok "a window keeps its plane while the window manager does not answer, the construct answers three calls (after$deaf_times ms), and the window lands when it does"
run_pid "$second_probe"
kill "$second_probe"
until_ok 'test -z "$(others)"' 40 || fail "the second probe window stays"

frames() { ctl render-stats | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["outputs"]["HEADLESS-1"]["damaged_frames"])'; }
turn_frames=$(frames)
turn_start=$(date +%s%N)
call cube.turn '{"by": 1, "ms": 10000}' >/dev/null || fail "a slow cube.turn"
until_ok '[ "$(width construct-cube-face-0)" -gt 0 ] && [ "$(width construct-cube-face-1)" -gt 0 ]' 40 ||
    fail "during the turn the screen does not show two faces: widths $(width construct-cube-face-0) and $(width construct-cube-face-1)"
until_ok '[ "$(prop "construct-cube-float-0-$surface" visible)" != gone ]' 40 || fail "the window does not float during a turn"
[ -n "${SHOT:-}" ] && ctl capture "$SHOT" >/dev/null
[ "$(prop construct-cube-face-0 forward)" = False ] && [ "$(prop construct-cube-face-1 forward)" = False ] ||
    fail "a face forwards during the turn: $(prop construct-cube-face-0 forward) $(prop construct-cube-face-1 forward)"
call cube.turn '{"by": 1}' >/dev/null || fail "cube.turn during a turn"
until_ok 'test "$(prop construct-cube-face-1 visible)" = True && test "$(prop construct-cube-face-0 visible)" = False && test "$(prop construct-cube-face-1 forward)" = True' 80 ||
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
until_ok 'prop construct-face-card-head text | grep -q "^Tiling did not start"' 80 || fail "a manager that never connected says nothing on its face: $(prop construct-face-card-head text)"
prop construct-face-card-reason text | grep -q "did not connect within 10 s" || fail "the card gives no reason: $(prop construct-face-card-reason text)"
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
mkdir -p "$work/fail"
starts=$work/mc-starts
: >"$starts"
cat >"$work/fail/chora-xp" <<SH
#!/bin/sh
echo start >>"$starts"
if [ "\$(wc -l <"$starts")" -ge 3 ]; then exec "$bin/chora-xp" "\$@"; fi
echo "it needs foot 1.26 or later." >&2
sleep 2
exit 1
SH
chmod +x "$work/fail/chora-xp"
call settings.set "{\"cube_manager\": \"$work/fail/chora-xp\", \"theme\": \"light\"}" >/dev/null || fail "settings.set cube_manager to a manager that fails"
call_ms() {
    local start=${EPOCHREALTIME/./}
    call "$@" >/dev/null || return 1
    echo $(((${EPOCHREALTIME/./} - start) / 1000))
}
list_ms=$(call_ms construct.list) || fail "construct.list before cube.open"
open_ms=$(call_ms cube.open '{}') || fail "cube.open with a manager that fails"
[ $((open_ms - list_ms)) -le 300 ] || fail "cube.open took $open_ms ms, $((open_ms - list_ms)) ms more than construct.list: more than the 300 ms of docs/cube-modes.md"
prop construct-face-card-head text | grep -q "^Starting XP$" || fail "no card Starting XP on the face when cube.open answered: $(prop construct-face-card-head text)"
ok "a face whose manager starts shows Starting XP when cube.open answers, $((open_ms - list_ms)) ms more than construct.list"
lit_frame "$work/starting.png" || fail "a black frame while the manager starts"
[ -z "${STARTING_SHOT:-}" ] || cp "$work/starting.png" "$STARTING_SHOT"
for _ in $(seq 12); do
    lit_frame "$work/clip.png" || fail "a black frame while the manager starts or fails"
    sleep 0.2
done
until_ok 'prop construct-face-card-head text | grep -q "^XP did not start\.$"' 40 || fail "the face never says it did not start: $(prop construct-face-card-head text)"
prop construct-face-card-reason text | grep -q "^chora-xp stopped after 2 s: it needs foot 1.26 or later\.$" || fail "the reason on the card: $(prop construct-face-card-reason text)"
lit_frame "$work/failed.png" || fail "a black frame once the manager failed"
ok "a manager that exits: the card says XP did not start, with the reason, and no frame is black"
call view.step_back >/dev/null || fail "view.step_back with a failed face"
until_ok "$stood_back" 20 || fail "no step back with a failed face"
until_ok 'prop construct-face-label-words text | grep -q "did not start"' 20 || fail "the face label does not say did not start: $(prop construct-face-label-words text)"
[ "$(prop construct-face-label-name text)" = "XP 1" ] || fail "the face label names $(prop construct-face-label-name text)"
[ "$(prop construct-face-label-dot-0 color)" = "[0.0, 0.0, 0.0, 0.0]" ] || fail "the dot of the failed face is not hollow: $(prop construct-face-label-dot-0 color)"
until_ok 'prop construct-mode-step-in-text text | grep -q "Try again"' 20 || fail "the middle hint does not say Try again: $(prop construct-mode-step-in-text text)"
[ -z "${FAILED_SHOT:-}" ] || ctl capture "$FAILED_SHOT" >/dev/null
ok "stood back, the label says did not start, its dot is hollow and the middle hint says Try again"
plain Return
until_ok 'test "$(wc -l <"$starts")" = 2' 20 || fail "Return did not start the manager again: $(wc -l <"$starts") starts"
quick 'prop construct-face-card-head text | grep -q "^Starting XP$"' || fail "no card Starting after Return: $(prop construct-face-card-head text)"
until_ok 'prop construct-face-card-head text | grep -q "^XP did not start\.$"' 40 || fail "the second start did not fail on the face: $(prop construct-face-card-head text)"
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
until_ok 'test "$(call cube.list 2>/dev/null | jq -r .mode)" = navigate' 40 || fail "co-79: Super+C did not stand back: $(call cube.list 2>&1 | head -c 300)"
until_ok 'test "$(bars)" = 4' 80 || fail "co-79: xp shows $(bars) taskbars"
had=" $(probe | awk '{print $1}' | tr '\n' ' ') "
fresh() { probe | awk -v had="$had" 'index(had, " " $1 " ") == 0'; }
key Super+space
until_ok 'fresh | grep -q " construct "' 100 || fail "co-79: the program of the launcher stood back is no pane: $(probe)"
read -r put _ <<<"$(fresh | head -n 1)"
until_ok 'test "$(prop construct-hint-step-back visible)" = True' 40 || fail "co-79: in the pane of the program no Step back shows"
key Alt+Escape
until_ok 'test "$(call cube.list | jq -r .mode)" = navigate && test "$(prop construct-mode-step-in visible)" = True' 40 || fail "co-79: Alt+Escape out of the pane did not stand back: $(call cube.list | jq -c '{mode, front}')"
key Ctrl+Alt+Right
until_ok 'test "$(call cube.list | jq -r "select(.turning == false) | .front")" = 1' 40 || fail "co-79: the cube did not turn to face 1: $(call cube.list | jq -c '{front, turning}')"
put_keys() { local k out=""; for k in 0 1 2; do out="$out $(prop "construct-hints-put-in-keys-$k-text" text | grep -v -e False -e gone)"; done; echo "$out"; }
until_ok 'test "$(put_keys)" = " Super Alt 2"' 20 ||
    fail "co-79: stood back out of a pane the hint bar names no way into a face (put-in keys:$(put_keys))"
ok "co-79: stood back out of a pane, the hint bar names Super Alt 2, the give key of the face manager"
front1=$(call cube.list | jq -r '.faces[1].output')
key Super+Alt+2
until_ok 'probe | grep -q "^$put xp $front1 "' 40 || fail "co-79: the told Super+Alt+2 did not put the window on the face at the front ($front1): $(probe | grep "^$put ")"
until_ok 'test "$(call cube.list | jq -r .mode)" = use' 40 || fail "co-79: after Super+Alt+2 the cube did not step into the face: $(call cube.list | jq -c '{mode, front}')"
until_ok 'xp_shows "$put"' 20 || fail "co-79: the face manager does not show the window $put"
log=$(find "$work" -name "prog.*.log" -printf "%T@ %p\n" | sort -n | tail -n 1 | cut -d" " -f2)
before=$(keys_in "$log")
plain a
until_ok 'test "$(keys_in "$log")" -gt "$before"' 20 || fail "co-79: after Super+Alt+2 the window in the face has no keyboard"
until_ok 'ctl query id:construct-hint-pull-out | grep -q construct-hint-pull-out' 20 || fail "co-79: in the face with the window put in, the bar names no pull key"
ok "co-79: Super+Alt+2 puts the window of the pane on face 1 at the front, steps in, the window takes keys, and the bar names the pull key"
python3 -I - "$CHORA_SOCKET" "$log" <<'PY'
import os, signal, sys
for pid in filter(str.isdigit, os.listdir("/proc")):
    try:
        cmd = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
        env = open(f"/proc/{pid}/environ", "rb").read().split(b"\0")
        out = os.readlink(f"/proc/{pid}/fd/1")
    except OSError:
        continue
    if cmd[0].endswith(b"chora-probe") and b"CHORA_SOCKET=" + sys.argv[1].encode() in env and out == sys.argv[2]:
        os.kill(int(pid), signal.SIGTERM)
PY
until_ok '! probe | grep -q "^$put "' 40 || fail "co-79: the program put into the face does not end: $(probe | grep "^$put ")"
call cube.close >/dev/null || fail "cube.close after the key"
until_ok 'test -z "$(virtuals)"' || fail "cube.close after the key left $(virtuals)"


until_ok "call cube.open '{}' >/dev/null 2>&1" || fail "cube.open before the kill: $(call cube.open '{}' 2>&1)"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "no outputs before the kill"
until_ok 'test "$(bars)" = 4' 80 || fail "no xp taskbars before the kill: $(bars)"
known=" $(probe | awk '{print $1}' | tr '\n' ' ') "
mine() { probe | awk -v known="$known" 'index(known, " " $1 " ") == 0'; }
"$bin/chora-probe" window 60 160 120 >"$work/probe6.log" 2>&1 &
probe6=$!
pids+=("$probe6")
until_ok 'mine | grep -q " xp VIRTUAL"' 80 || fail "the probe window of the kill came to no xp face: $(probe)"
xp_pid=$(pid_of chora-xp)
kill -9 "$(pid_of chora-construct)"
until_ok 'mine | grep -q " xp HEADLESS-1 " && ! mine | grep -q " VIRTUAL"' 12 ||
    fail "a dying construct left no flat xp with the window within 3 s: $(mine)"
[ "$(pid_of chora-xp)" = "$xp_pid" ] || fail "the xp of the cube did not outlive the construct: $(pid_of chora-xp)"
[ -n "$(ctl query id:xp-bar-HEADLESS-1 | jq '.[]')" ] || fail "the flat xp shows no taskbar on the screen"
ok "a dying construct leaves a flat xp on the screen with every window"
"$bin/chora-construct" >"$work/construct2.log" 2>&1 &
pids+=($!)
until_ok 'test "$(call cube.list 2>/dev/null | jq ".faces | length")" = 4' 80 ||
    fail "a restarted construct did not reopen the cube: $(call cube.list 2>&1)"
until_ok 'mine | grep -q " xp VIRTUAL" && ! mine | grep -q " HEADLESS-1 "' 40 ||
    fail "the window did not go back to a face of the reopened cube: $(mine)"
[ "$(pid_of chora-xp)" = "$xp_pid" ] || fail "the reopened cube runs another xp: $(pid_of chora-xp)"
ok "a restarted construct reopens the cube with the windows on its faces"
kill "$probe6"
call cube.close >/dev/null || fail "cube.close after the restart"
until_ok 'test -z "$(virtuals)"' || fail "cube.close after the restart left $(virtuals)"
until_ok 'test -z "$(pid_of chora-xp)"' 40 || fail "the xp of the reopened cube outlived cube.close: $(pid_of chora-xp)"

"$bin/chora-probe" window 300 200 --report >"$work/pane.log" 2>&1 &
pids+=($!)
until_ok 'call construct.list | grep -q "\"surface\""' 40 || fail "a window became no pane before the cube"
pane_node=construct-pane-$(call construct.list | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["panes"][0]["surface"])')
[ "$(prop "$pane_node" visible)" = True ] || fail "the pane of the probe window does not show while walking: $(prop "$pane_node" visible)"
call cube.open '{}' >/dev/null || fail "cube.open with a pane open"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "no outputs with a pane open"
call view.step_back >/dev/null || fail "view.step_back with a pane open"
until_ok '[ "$(prop construct-cube-face-0 forward)" = False ]' 20 || fail "no step back with a pane open"
beside() { [ "$(prop "$pane_node" visible)" = True ] && [ "$(call state.get '{}' | jq -r '.windows[0].shown')" = true ]; }
until_ok beside 20 || fail "the pane in front of the cube stood back does not show beside it (co-54): visible $(prop "$pane_node" visible), $(call state.get '{}' | jq -c .windows)"
until_ok '[ "$(prop construct-label visible)" != True ] || [ "$(prop construct-label-body opacity)" = 0.0 ]' 20 || fail "the pane label shows over the cube stood back"
until_ok '[ "$(edge construct-face-label-glass 1)" -gt 0 ]' 20 || fail "no face label with a pane open"
ok "stood back with a pane open, the pane goes beside the cube and shows, its label hides, the face label stays under the cube"
call view.step_in >/dev/null || fail "view.step_in with a pane open"

call cube.close >/dev/null || fail "cube.close before step 8"
until_ok 'test -z "$(virtuals)"' || fail "cube.close before step 8 left $(virtuals)"
python3 -I - "$CHORA_SOCKET" <<'PY'
import os, signal, sys
for pid in filter(str.isdigit, os.listdir("/proc")):
    try:
        cmd = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
        env = open(f"/proc/{pid}/environ", "rb").read().split(b"\0")
    except OSError:
        continue
    if cmd[0].endswith(b"chora-probe") and b"CHORA_SOCKET=" + sys.argv[1].encode() in env:
        os.kill(int(pid), signal.SIGTERM)
PY
center() { ctl query "id:$1" | python3 -I -c 'import json, sys; r = next(iter(json.load(sys.stdin)[0]["rects"].values())); print(round(r[0] + r[2] / 2), round(r[1] + r[3] / 2))'; }
cube_keys() { local k out=""; for k in 0 1; do out="$out $(prop "construct-hints-cube-keys-$k-text" text | grep -v -e False -e gone)"; done; echo "$out"; }
hint_words() { ctl query '*' | jq -r '[.[] | select((.id | test("^construct-hints-.*-word$")) and (.rects | length > 0)) | .text] | join(", ")'; }
had=" $(probe | awk '{print $1}' | tr '\n' ' ') "
until_ok 'ctl query id:construct-start-row-0 | jq -e ".[0].rects | length > 0" >/dev/null' 40 ||
    fail "co-79: no start card with its Terminal row on the empty floor: $(call construct.list | jq -c '[.panes[].name]') $(probe)"
read -r x y <<<"$(center construct-start-row-0)"
inject "[{\"pointer_move\": {\"position\": [$x, $y]}}, {\"click\": {}}]"
until_ok 'fresh | grep -q " construct "' 100 || fail "co-79: the Terminal row of the start card started no pane: $(probe)"
read -r term _ <<<"$(fresh | head -n 1)"
until_ok 'test "$(prop construct-hint-step-back visible)" = True' 40 || fail "co-79: in the pane of the Terminal row no Step back shows"
key Alt+Escape
until_ok 'test "$(cube_keys)" = " Super C"' 20 ||
    fail "co-79: stepped out of the pane of the Terminal row, with no cube open, the hint bar names no way to a cube (cube keys:$(cube_keys); bar: $(hint_words); focus: $(ctl query kind:surface | jq -c '[.[] | select(.focused) | .id]')): $(call cube.list | jq -c '{mode, faces: (.faces | length)}')"
ok "co-79: stepped out of the pane of the Terminal row, the hint bar names Super C Cube"
key Super+c
until_ok 'test "$(call cube.list 2>/dev/null | jq -r .mode)" = navigate' 40 || fail "co-79: the told Super+C did not stand back from a cube: $(call cube.list 2>&1 | head -c 300)"
until_ok 'test "$(bars)" = 4' 80 || fail "co-79: xp shows $(bars) taskbars after the told Super+C"
until_ok 'test "$(put_keys)" = " Super Alt 2"' 20 ||
    fail "co-79: stood back by the told Super+C, the hint bar names no way into a face for the window of the pane (put-in keys:$(put_keys))"
front0=$(call cube.list | jq -r '.faces[0].output')
until_ok '! ctl call xp window.move "{}" 2>&1 | grep -q "no exported method"' 20 || fail "co-79: the xp of the new cube exports no window.move"
key Super+Alt+2
until_ok 'probe | grep -q "^$term xp $front0 "' 40 || fail "co-79: the told Super+Alt+2 did not put the window of the Terminal row on the face at the front ($front0): $(probe | grep "^$term ")"
until_ok 'test "$(call cube.list | jq -r .mode)" = use' 40 || fail "co-79: after Super+Alt+2 the cube did not step into the face: $(call cube.list | jq -c '{mode, front}')"
before=$(keys_in "$work/term.log")
plain a
until_ok 'test "$(keys_in "$work/term.log")" -gt "$before"' 20 || fail "co-79: the window of the Terminal row put into the face has no keyboard"
until_ok 'ctl query id:construct-hint-pull-out | grep -q construct-hint-pull-out' 20 || fail "co-79: in the face with the window of the Terminal row, the bar names no pull key"
ok "co-79: Terminal row, Alt+Escape, the told Super+C and the told Super+Alt+2: the window is in face 0 and takes keys"
camera() { call construct.list | jq -c .camera; }
# shellcheck disable=SC2034 # used in the until_ok string
cam_before=$(camera)
key Super+Alt+4
until_ok 'probe | grep -q "^$term construct "' 40 || fail "co-79: Super+Alt+4 did not pull the window out of the face: $(probe | grep "^$term ")"
until_ok 'test "$(camera)" != "$cam_before" && test "$(camera)" = "$(camera)"' 40 || fail "co-79: the camera did not fly into the pulled pane: $(camera)"
until_ok 'test "$(prop construct-hint-step-back visible)" = True' 40 || fail "co-79: in the pulled pane no Step back shows"
key Alt+Escape
until_ok 'test "$(call cube.list | jq -r .mode)" = navigate && test "$(prop construct-mode-step-in visible)" = True' 40 ||
    fail "co-79: Alt+Escape out of the pulled pane did not stand back from the cube: $(call cube.list | jq -c '{mode, front}'), step-in chip $(prop construct-mode-step-in visible), bar: $(hint_words)"
until_ok 'test "$(put_keys)" = " Super Alt 2"' 20 ||
    fail "co-79: stood back out of the pulled pane the hint bar names no way into a face (put-in keys:$(put_keys))"
key Super+Alt+2
until_ok 'probe | grep -q "^$term xp $front0 "' 40 || fail "co-79: the told Super+Alt+2 did not put the pulled window back on the face at the front: $(probe | grep "^$term ")"
until_ok 'test "$(call cube.list | jq -r .mode)" = use' 40 || fail "co-79: after Super+Alt+2 out of the pulled pane the cube did not step in"
ok "co-79: Alt+Escape out of a pane pulled from a face stands back, and the told Super+Alt+2 puts the window back"
call cube.close >/dev/null || fail "cube.close after step 8"
until_ok 'test -z "$(virtuals)"' || fail "cube.close after step 8 left $(virtuals)"

python3 -I - "$CHORA_SOCKET" "$work/term.log" <<'PY'
import os, signal, sys
for pid in filter(str.isdigit, os.listdir("/proc")):
    try:
        cmd = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
        env = open(f"/proc/{pid}/environ", "rb").read().split(b"\0")
        out = os.readlink(f"/proc/{pid}/fd/1")
    except OSError:
        continue
    if cmd[0].endswith(b"chora-probe") and b"CHORA_SOCKET=" + sys.argv[1].encode() in env and out == sys.argv[2]:
        os.kill(int(pid), signal.SIGTERM)
PY
until_ok '! probe | grep -q "^$term "' 40 || fail "rpc2-1: the window of the Terminal row does not end: $(probe | grep "^$term ")"
sheet_words() { ctl query '*' | jq -r '[.[] | select((.id | startswith("construct-help")) and (.text != null)) | .text] | join("|")'; }
call cube.open '{}' >/dev/null || fail "rpc2-1: cube.open: $(call cube.open '{}' 2>&1 | head -c 300)"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "rpc2-1: cube.open gave $(virtuals)"
call view.step_back >/dev/null || fail "rpc2-1: view.step_back"
until_ok 'test "$(call cube.list 2>/dev/null | jq -r .mode)" = navigate' 40 || fail "rpc2-1: the cube did not stand back: $(call cube.list 2>&1 | head -c 300)"
until_ok 'test "$(bars)" = 4' 80 || fail "rpc2-1: xp shows $(bars) taskbars"
had=" $(probe | awk '{print $1}' | tr '\n' ' ') "
key Super+space
until_ok 'fresh | grep -q " construct "' 100 || fail "rpc2-1: the program of the launcher stood back is no pane: $(probe)"
until_ok 'test "$(prop construct-hint-step-back visible)" = True' 40 || fail "rpc2-1: in the pane of the program no Step back shows"
inject '[{"pointer_move": {"node": "construct-hint-keys", "point": [8, 8]}}, {"click": {}}]'
until_ok help_shown 12 || fail "rpc2-1: a click on the hint All keys in a pane shows no key list"
until_ok 'sheet_words | grep -q "Step back out of a pane"' 20 || fail "rpc2-1: the key list in a pane does not list the way out: $(sheet_words)"
for typed in "Fly into the pane ahead" "Walk forward, back, sideways" "Step into the face" "Close the cube, stood back" "Back out one step"; do
    if sheet_words | grep -q "$typed"; then
        fail "rpc2-1: the key list in a pane lists \"$typed\", which types into the window: $(sheet_words)"
    fi
done
[ -z "${SHEET_SHOT:-}" ] || ctl capture "$SHEET_SHOT" >/dev/null
ok "rpc2-1: the key list in a pane lists only the keys that work there"
plain Escape
until_ok '! help_shown' 20 || fail "rpc2-2: Escape keeps the key list opened in a pane"
until_ok 'ctl query id:construct-hint-step-back | jq -e ".[0].rects | length > 0" >/dev/null' 40 || fail "rpc2-2: after the key list in a pane no hint Step back shows: $(hint_words)"
inject '[{"pointer_move": {"node": "construct-hint-step-back", "point": [8, 8]}}, {"click": {}}]'
until_ok 'test "$(call cube.list | jq -r .mode)" = navigate && test "$(prop construct-mode-step-in visible)" = True' 40 || fail "rpc2-2: the hint Step back after the key list did not stand back: $(call cube.list | jq -c '{mode, front}'), camera $(call construct.list | jq -c .camera), focus $(ctl query kind:surface | jq -c '[.[] | select(.focused) | .id]'), bar: $(hint_words)"
until_ok 'test "$(put_keys)" = " Super Alt 2"' 20 ||
    fail "rpc2-2: stood back out of a pane after the key list the hint bar names no way into a face (put-in keys:$(put_keys); bar: $(hint_words))"
[ -z "${BACK_SHOT:-}" ] || ctl capture "$BACK_SHOT" >/dev/null
ok "rpc2-2: after the key list in a pane the step back names Super Alt 2 as it does without it"
call cube.close >/dev/null || fail "cube.close after step 9"
until_ok 'test -z "$(virtuals)"' || fail "cube.close after step 9 left $(virtuals)"
