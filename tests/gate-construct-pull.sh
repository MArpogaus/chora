#!/bin/bash
# Gate of the pull-out (E3): a window leaves a face of the cube of the
# construct while its title bar is dragged. chora-xp gives it with the
# drag to chora-construct, which shows the face; there it becomes a pane
# that goes on with the drag and wobbles with a depth.
#
# The front face of a resting cube fills its screen, so the drag leaves
# the face over the edge of the screen, onto a second output.
#
# 1. With the flag --test-injected-press-is-real of the core (headless
#    only), where an injected press gives its drag: inside the face the window moves and
#    stays with xp; past the edge it is a pane of the construct, and the
#    mesh of the pane has a depth; after a drop beside the cube it is
#    still its pane, with a name, in front of the face at least at
#    1:1. Then the pane rests when the pointer
#    moves, and the camera is in no window; with the focus glow on the
#    pane, a still desktop draws nothing (P1). A key typed during the
#    fly-in into the pane reaches the window. The window exists at
#    every step.
#    The task button of the window leaves the taskbar of its face.
# 2. A pane dropped over a face goes back to xp on the output of that
#    face, at the drop point: a move from face to face stays in xp.
#    Its task button is on the taskbar of that face only.
# 2a. Stood back (K12): a drag on the floating plane of a window pulls
#    the window out of the cube; dropped beside the cube it is a pane in
#    view, and a plain drag of that pane over the front face makes it a
#    window of xp again, the same act both ways; a pulled window dropped
#    over the front face is a window of xp again;
#    an injected drag on a plane pulls nothing where the construct takes
#    no injected input (CHORA_TEST_ACCEPT_INJECTED_KEYS=0). A
#    window half over the right edge of the face shows in full on its
#    plane, past the edge.
# 3. Without the flag the core refuses the give of an injected drag:
#    past the edge the window is still with xp, and no pane shows it;
#    back in the face it goes on with the drag, and stays after the
#    release.
# 4. A construct on a client socket opens no cube: the core refuses its
#    virtual outputs, so it has no face to pull from or to drop on, and
#    never calls the privileged window.move of xp.
#
# Usage: tests/gate-construct-pull.sh   BIN: directory with the binaries
# shellcheck disable=SC2016
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
probe() {
    ctl query kind:surface | python3 -I -c '
import json, sys
for n in json.load(sys.stdin):
    if n["surface"].get("app_id") == "chora-probe":
        out, r = next(iter(n["rects"].items()), ("-", [0, 0, 0, 0]))
        print(n["id"], n.get("owner"), out, round(r[0]), round(r[1]))'
}
owner() { probe | awk '{print $2}'; }
pane_of() {
    call window.list '{}' | python3 -I -c '
import json, sys
print(next((w["pane"] for w in json.load(sys.stdin)["windows"] if w["surface"] == sys.argv[1]), "lost"))' "$1"
}
pane_name() {
    call window.list '{}' | python3 -I -c '
import json, sys
print(next((w["name"] for w in json.load(sys.stdin)["windows"] if w["surface"] == sys.argv[1]), ""))' "$1"
}
node() {
    ctl query "id:$1" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
print(json.dumps([nodes[0].get("parent"), nodes[0]["rects"]]) if nodes else "gone")'
}
deepest() {
    ctl query "id:construct-pane-$1" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
depth = (nodes[0].get("mesh") or {}).get("depth") or [] if nodes else []
print(max((abs(z) for z in depth), default=0))'
}

pull() {
    env_file=$work/$1.env session_start 1280x800 -- "$bin/chora-construct"
    grant input.inject screen.capture
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
    call settings.set '{"effects": ["wobbly"]}' >/dev/null || fail "settings.set effects"
    call cube.open '{}' >/dev/null || fail "cube.open"
    until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "xp shows no taskbar on the front face"
    edge=$(timeout 10 "$bin/choractl" add-output "640x800+1280+0" |
        python3 -I -c 'import json, sys; print(json.load(sys.stdin)["name"])') || fail "no second output"
    inject '[{"pointer_move": {"position": [640, 400]}}]'
    "$bin/chora-probe" window 40 120 200 --server-deco --report >"$work/$1-probe.log" 2>&1 &
    pids+=($!)
    until_ok 'probe | grep -q " xp VIRTUAL-1 "' 80 || fail "the probe window came to no xp face: $(probe)"
    read -r surface _ _ x y <<<"$(probe)"
    until_ok 'ctl query "id:xp-title-$surface" | grep -q VIRTUAL-1' 40 || fail "the window has no title bar"
    inject "[{\"pointer_move\": {\"position\": [$((x + 90)), $((y - 24))]}},
        {\"button\": {\"state\": \"pressed\", \"hold\": true}}]"
    inject '[{"pointer_move": {"position": [900, 300]}}]'
    until_ok 'read -r _ _ _ nx _ <<<"$(probe)"; test "$nx" != "$x"' 20 ||
        fail "the drag of the title bar does not move the window: $(probe)"
    [ "$(owner)" = xp ] || fail "the window left xp inside its face: $(probe)"
    inject "[{\"pointer_move\": {\"position\": [200, 300], \"output\": \"$edge\"}}]"
}
end_session() {
    inject '[{"button": {"state": "released"}}]'
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
}

CHORA_TEST_FLAGS=--test-injected-press-is-real pull given
ok "inside its face the window moves with the drag and stays with xp"
until_ok 'test "$(owner)" = construct' 40 || fail "the window did not go to the construct past the edge: $(probe)"
until_ok 'test "$(pane_of "$surface")" != none && test "$(pane_of "$surface")" != lost' 20 ||
    fail "the window is no pane of the construct: $(pane_of "$surface")"
task_on() { ctl query "id:xp-task-$surface-$1" | python3 -I -c 'import json, sys; print(len(json.load(sys.stdin)))'; }
until_ok 'test "$(task_on VIRTUAL-1)" = 0' 20 || fail "the task button of a pulled-out window stays on the taskbar of its face"
ok "past the edge of its face the window is a pane of the construct, and its task button went"
inject "[{\"pointer_move\": {\"position\": [260, 340], \"output\": \"$edge\"}}]"
inject '[{"pointer_move": {"position": [900, 420]}}]'
until_ok 'python3 -I -c "import sys; sys.exit(float(sys.argv[1]) <= 0)" "$(deepest "$surface")"' 20 || fail "the mesh of the pane has no depth: $(deepest "$surface")"
ok "the pane wobbles on with a depth of $(deepest "$surface")"
inject "[{\"pointer_move\": {\"position\": [220, 320], \"output\": \"$edge\"}},
    {\"button\": {\"state\": \"released\"}}]"
sleep 1
[ "$(owner)" = construct ] || fail "the window is lost or left after the drop: $(probe)"
[ "$(pane_of "$surface")" != none ] || fail "the pane went after the drop"
[ -n "$(pane_name "$surface")" ] || fail "the pane of the window has no name"
ok "after a drop beside the cube the window is still the pane $(pane_name "$surface") of the construct: $(probe)"
shown_width() {
    ctl query "id:construct-pane-$surface" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
print(round(nodes[0]["rects"].get("HEADLESS-1", [0, 0, 0, 0])[2]) if nodes else 0)'
}
[ "$(shown_width)" -ge 200 ] ||
    fail "the pane of the pulled window is not in front of the face at 1:1 or nearer: $(shown_width) px wide for 200"
[ "$(pane_of "$surface")" = shown ] || fail "the pane of the pulled window is not in view: $(pane_of "$surface")"
ok "the pulled window shows as a pane in front of the face, $(shown_width) px wide"
sleep 2
rests=$(node "construct-pane-$surface")
[ "$(node "$surface" | python3 -I -c 'import json, sys; print(json.load(sys.stdin)[0])')" = "construct-pane-$surface" ] ||
    fail "the camera flew into the window after the pull-out: $(node "$surface")"
inject '[{"pointer_move": {"position": [300, 200]}}]'
inject '[{"pointer_move": {"position": [900, 600]}}]'
sleep 1
[ "$(node "construct-pane-$surface")" = "$rests" ] ||
    fail "the pane follows the pointer with no button down: $rests, then $(node "construct-pane-$surface")"
[ "$(node "$surface" | python3 -I -c 'import json, sys; print(json.load(sys.stdin)[0])')" = "construct-pane-$surface" ] ||
    fail "the camera flew into the window: $(node "$surface")"
ok "after the release the pane rests and the camera is in no window"
drawn() { ctl render-stats | python3 -I -c 'import json, sys; print(sorted((k, v["damaged_frames"], v["passes"]) for k, v in json.load(sys.stdin)["outputs"].items()))'; }
focused_pane() { ctl query id:construct-focus | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(bool(n and n[0].get("visible")))'; }
[ "$(focused_pane)" = True ] || fail "no pane shows the focus glow: $(focused_pane)"
still() {
    local before
    for _ in 1 2 3; do
        before=$(drawn)
        sleep 2
        [ "$before" != "$(drawn)" ] || return 0
    done
    return 1
}
still || fail "a still desktop with a focused pane draws: $(drawn), 2 s later $(sleep 2; drawn)"
ok "a still desktop with a focused pane draws nothing, by render.stats"
call pane.fly "{\"name\": \"$(pane_name "$surface")\"}" >/dev/null || fail "pane.fly"
inject '[{"key": {"key": "x", "state": "pressed"}}, {"key": {"key": "x", "state": "released"}}]'
[ "$(node "$surface" | python3 -I -c 'import json, sys; print(json.load(sys.stdin)[0])')" = "construct-pane-$surface" ] ||
    fail "the fly-in ended before the key: the check proves nothing: $(node "$surface")"
until_ok 'grep -q "^key 45$" "$work/given-probe.log"' 20 ||
    fail "a key typed during the fly-in did not reach the window: $(tail -n 3 "$work/given-probe.log")"
ok "a key typed during the fly-in reaches the window"
end_session

CHORA_TEST_FLAGS=--test-injected-press-is-real pull returned
until_ok 'test "$(owner)" = construct' 40 || fail "the window did not go to the construct past the edge: $(probe)"
inject '[{"pointer_move": {"position": [700, 400]}}, {"button": {"state": "released"}}]'
until_ok 'probe | grep -q " xp VIRTUAL-1 "' 40 || fail "a pane dropped over the front face did not go back to xp: $(probe)"
near_the_drop() {
    read -r _ _ _ nx ny <<<"$(probe)"
    [ "$nx" -gt 480 ] && [ "$nx" -lt 720 ] && [ "$ny" -gt 380 ] && [ "$ny" -lt 480 ]
}
until_ok near_the_drop 20 || fail "the window is not at the drop point (700, 400): $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "the pane stays after its window went back: $(pane_of "$surface")"
until_ok 'test "$(task_on VIRTUAL-1)" = 1 && test "$(task_on VIRTUAL-2)" = 0' 20 ||
    fail "the task button of the window that came back is not on the taskbar of its face only: $(task_on VIRTUAL-1) $(task_on VIRTUAL-2)"
ok "a pane dropped over a face is a window of xp on that face again, at the drop point: $(probe)"
end_session

plane_middle() {
    ctl query "id:construct-cube-float-0-$surface" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
r = nodes[0]["rects"].get("HEADLESS-1") if nodes and nodes[0].get("visible") else None
print("%d %d" % (r[0] + r[2] / 2, r[1] + r[3] / 2) if r else "")'
}
pane_middle() {
    ctl query "id:construct-pane-$surface" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
r = nodes[0]["rects"].get("HEADLESS-1") if nodes else None
print("%d %d" % (r[0] + r[2] / 2, r[1] + r[3] / 2) if r else "")'
}
stood() {
    env_file=$work/$1.env session_start 1280x800 -- "$bin/chora-construct"
    grant input.inject screen.capture
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
    call cube.open '{}' >/dev/null || fail "cube.open"
    until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "xp shows no taskbar on the front face"
    inject '[{"pointer_move": {"position": [640, 400]}}]'
    "$bin/chora-probe" window 40 120 200 --server-deco --report >"$work/$1-probe.log" 2>&1 &
    pids+=($!)
    until_ok 'probe | grep -q " xp VIRTUAL-1 "' 80 || fail "the probe window came to no xp face: $(probe)"
    read -r surface _ <<<"$(probe)"
    call view.step_back '{}' >/dev/null || fail "view.step_back"
    until_ok 'test -n "$(plane_middle)"' 40 || fail "no floating plane of the window"
    sleep 1
    read -r px py <<<"$(plane_middle)"
    inject "[{\"pointer_move\": {\"position\": [$px, $py]}},
        {\"button\": {\"state\": \"pressed\", \"hold\": true}},
        {\"pointer_move\": {\"position\": [$((px + 30)), $py]}},
        {\"pointer_move\": {\"position\": [$((px + 60)), $py]}}]"
}

CHORA_TEST_FLAGS=--test-injected-press-is-real stood plane_beside
until_ok 'test "$(owner)" = construct' 40 || fail "a drag on the floating plane did not pull the window out: $(probe)"
inject '[{"pointer_move": {"position": [1200, 100]}}, {"button": {"state": "released"}}]'
sleep 1
[ "$(owner)" = construct ] || fail "the window pulled from the stood-back cube is lost after the drop: $(probe)"
[ "$(pane_of "$surface")" = shown ] || fail "the pulled window is no pane in view: $(pane_of "$surface")"
ok "stood back, a drag on a floating plane pulls its window out; dropped beside the cube it is a pane: $(probe)"
call pane.move '{"name": "chora-probe", "right": 200, "ahead": 150}' >/dev/null || fail "pane.move chora-probe"
until_ok 'test -n "$(pane_middle)"' 40 || fail "the pulled pane shows nowhere on the screen"
read -r qx qy <<<"$(pane_middle)"
inject "[{\"pointer_move\": {\"position\": [$qx, $qy]}},
    {\"button\": {\"state\": \"pressed\", \"hold\": true}},
    {\"pointer_move\": {\"position\": [$((qx - 30)), $qy]}},
    {\"pointer_move\": {\"position\": [600, 350]}},
    {\"button\": {\"state\": \"released\"}}]"
until_ok 'probe | grep -q " xp VIRTUAL-1 "' 40 ||
    fail "a plain drag of the pulled pane over the front face did not give it back: $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "the pane stays after a plain drag gave it back: $(pane_of "$surface")"
ok "stood back, a plain drag of the pane over the front face gives it back, the act that pulled it out: $(probe)"
end_session

CHORA_TEST_FLAGS=--test-injected-press-is-real stood plane_back
until_ok 'test "$(owner)" = construct' 40 || fail "a drag on the floating plane did not pull the window out: $(probe)"
inject '[{"pointer_move": {"position": [600, 350]}}, {"button": {"state": "released"}}]'
until_ok 'probe | grep -q " xp VIRTUAL-1 "' 40 || fail "a pane dropped over the stood-back front face did not go back to xp: $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "the pane stays after its window went back: $(pane_of "$surface")"
ok "stood back, a pulled window dropped over the front face is a window of xp again: $(probe)"
end_session

CHORA_TEST_ACCEPT_INJECTED_KEYS=0 stood plane_injected
sleep 2
[ "$(owner)" = xp ] || fail "an injected drag on a floating plane pulled the window: $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "an injected plane drag made a pane: $(pane_of "$surface")"
inject '[{"button": {"state": "released"}}]'
ok "where the construct takes no injected input, an injected drag on a floating plane pulls nothing: $(probe)"
end_session

right_of() {
    ctl query "id:$1" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
r = nodes[0]["rects"].get("HEADLESS-1") if nodes else None
print(round(r[0] + r[2]) if r else -1)'
}
env_file=$work/edge.env CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
call cube.open '{}' >/dev/null || fail "cube.open"
until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "xp shows no taskbar on the front face"
inject '[{"pointer_move": {"position": [640, 400]}}]'
"$bin/chora-probe" window 40 120 200 --server-deco >"$work/edge-probe.log" 2>&1 &
pids+=($!)
until_ok 'probe | grep -q " xp VIRTUAL-1 "' 80 || fail "the probe window came to no xp face: $(probe)"
read -r surface _ _ x y <<<"$(probe)"
until_ok 'ctl query "id:xp-title-$surface" | grep -q VIRTUAL-1' 40 || fail "the window has no title bar"
inject "[{\"pointer_move\": {\"position\": [$((x + 50)), $((y - 12))]}},
    {\"button\": {\"state\": \"pressed\", \"hold\": true}},
    {\"pointer_move\": {\"position\": [$((x + 300)), $((y - 12))]}},
    {\"pointer_move\": {\"position\": [1220, $((y - 12))]}},
    {\"button\": {\"state\": \"released\"}}]"
until_ok 'read -r _ _ _ nx _ <<<"$(probe)"; test "$nx" -gt 1100' 20 || fail "the window does not hang over the right edge: $(probe)"
call view.step_back '{}' >/dev/null || fail "view.step_back"
until_ok 'test -n "$(plane_middle)"' 40 || fail "no floating plane of the window"
sleep 1
plane_right=$(right_of "construct-cube-float-0-$surface")
face_right=$(right_of construct-cube-face-0)
[ "$plane_right" -gt $((face_right + 20)) ] ||
    fail "the window over the right edge is clipped at the face: plane to $plane_right, face to $face_right"
ok "stood back, a window half over the right edge shows in full, to $plane_right past the face at $face_right"
end_session

pull injected
sleep 2
[ "$(owner)" = xp ] || fail "an injected drag gave the window past the edge: $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "an injected drag made a pane: $(pane_of "$surface")"
ok "an injected drag gives nothing: past the edge the window is still with xp"
read -r _ _ _ x _ <<<"$(probe)"
inject '[{"pointer_move": {"position": [500, 300]}}]'
until_ok 'read -r _ _ _ nx _ <<<"$(probe)"; test "$nx" != "$x"' 20 ||
    fail "after the refused give the window left its drag: $(probe)"
inject '[{"button": {"state": "released"}}]'
sleep 1
[ "$(owner)" = xp ] || fail "the window left xp after the refused give: $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "a refused give made a pane: $(pane_of "$surface")"
ok "after the refused give the window goes on with the drag and stays with xp: $(probe)"
end_session

session_start 1280x800 -- sleep 600
socket=$(timeout 10 "$bin/choractl" clients create construct |
    python3 -I -c 'import json, sys; print(json.load(sys.stdin)["path"])') || fail "no client socket"
CHORA_SOCKET=$socket "$bin/chora-construct" >"$work/confined.log" 2>&1 &
pids+=($!)
until_ok 'timeout 10 "$bin/choractl" call construct window.list "{}" >/dev/null 2>&1' 40 ||
    fail "the construct on a client socket exports no methods: $(tail -n 3 "$work/confined.log")"
refusal=$(timeout 10 "$bin/choractl" call construct cube.open '{}' 2>&1) &&
    fail "a construct on a client socket opened a cube: $refusal"
[[ $refusal == *"adds no virtual output"* ]] || fail "the refusal does not name the reason: $refusal"
timeout 10 "$bin/choractl" outputs | grep -q VIRTUAL && fail "a confined construct has a virtual output"
ok "a construct on a client socket opens no cube: $refusal"
kill "${pids[@]}" 2>/dev/null
wait 2>/dev/null
pids=()
