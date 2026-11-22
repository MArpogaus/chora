#!/bin/bash
# Gate of the pull-out (E3): a window leaves a face of the cube of the
# construct while its title bar is dragged. chora-xp gives it with the
# drag to chora-construct, which shows the face; there it becomes a pane
# that goes on with the drag and wobbles with a depth.
#
# The front face of a resting cube fills its screen, so the drag leaves
# the face over the edge of the screen, onto a second output.
#
# PULL_SHOT: a PNG path that keeps a capture of the pane in the pull.
# BEHIND_SHOT: a PNG path that keeps a capture of the pane after the step
# back.
# PAIR_ONLY: set, runs only the two-pane part of the cell (about 7 s), the
# reproducer of "the second window is no pane" under load.
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
# 2a. After the pull the step back (Alt+Escape) keeps the pane in view,
#    drawn, with no window lost, and a Super drag of the pane over a face
#    gives the window back to xp. A pane moved in front of the face, as
#    large as the face, does not cover the cube that stands back (co-21):
#    the faces that show meet at a shared edge within 1 px, the edge is as
#    long on both faces within 2 %, the mirror image of each face lies
#    under it, and no pane lies over a face.
#    Stepped in again, the pane is home at the rect it had before the step
#    back, and beside the cube at the same rect when it stands back again.
# 2b. Stood back (K12): a drag on the floating plane of a window turns
#    the cube and pulls nothing out; an injected drag on a plane pulls nothing where the construct takes
#    no injected input (CHORA_TEST_ACCEPT_INJECTED_KEYS=0). A
#    window half over the right edge of the face shows in full on its
#    plane, past the edge.
# 2c. Two panes that cover a cube that steps back, both on its left, take
#    places beside it that overlap neither each other nor a face; the
#    mirror of each face lies under it (top, height, depth) and the
#    shared edges of the faces agree within 1 px.
# 3. Without the flag the core refuses the give of an injected drag:
#    past the edge the window is still with xp, and no pane shows it;
#    back in the face it goes on with the drag, and stays after the
#    release.
# 5. On one output the title drag cannot leave the face (co-33): in use
#    mode the press on a title bar shows the hint that names the path that
#    works, the helper (`cube.pull`), and it moves no window across; the
#    hint is gone with the release. Stood back, a plain drag on a plane
#    turns the cube with no hint (K12), and with a second output the
#    designed pull works with no hint. In use mode the program of the
#    face keeps Super (rw-1): Super pressed and let go alone opens the
#    Start menu of xp, and Super with F1 of the construct between opens
#    none, each key a press and a release of its own.
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
    local rounds=${2:-40}
    local end=$((SECONDS + rounds / 2))
    while :; do
        eval "$1" && return 0
        [ "$SECONDS" -lt "$end" ] || return 1
        sleep 0.25
    done
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
pane_name_of() { call window.list '{}' | python3 -I -c 'import json, sys; print(next((w["name"] for w in json.load(sys.stdin)["windows"] if w["name"] == sys.argv[1] and w["pane"] != "none"), ""))' "$1"; }
pane_rect() {
    ctl query "id:construct-pane-$surface" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
r = nodes[0]["rects"].get("HEADLESS-1") if nodes else None
print(" ".join("%.1f" % v for v in r) if r else "")'
}
pane_middle() {
    ctl query "id:construct-pane-$surface" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
r = nodes[0]["rects"].get("HEADLESS-1") if nodes else None
print("%d %d" % (r[0] + r[2] / 2, r[1] + r[3] / 2) if r else "")'
}
hint_text() {
    call window.list '{}' >/dev/null
    ctl query "id:construct-hints-pull-word" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
print(nodes[0].get("text", "") if nodes and nodes[0].get("visible", True) else "")'
}
prop() { ctl query "id:$1" | python3 -I -c "import json, sys; n = json.load(sys.stdin); print(n[0].get('$2', False) if n else 'gone')"; }
width() { ctl query "id:$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(round(n[0]["rects"]["HEADLESS-1"][2]) if n and n[0]["rects"] else 0)'; }
stood_back='test "$(prop construct-cube-face-0 forward)" = False && test "$(width construct-cube-face-0)" -lt 1100 && test "$(width construct-cube-face-0)" -gt 300'
closed_cube_breaches() {
    python3 -I -c '
import json, os, re, socket
s = socket.socket(socket.AF_UNIX)
s.connect(os.environ["CHORA_SOCKET"])
f = s.makefile("rwb")
f.write(b"{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"scene.boxes\"}\n")
f.flush()
boxes = next(m for m in map(json.loads, f) if m.get("id") == 1)["result"]["HEADLESS-1"]["boxes"]
faces, mirrors, panes = {}, {}, []
for b in boxes:
    n = b.get("node") or ""
    if m := re.fullmatch(r"construct-cube-face-(\d+)", n):
        faces[int(m[1])] = b
    elif m := re.fullmatch(r"construct-cube-mirror-(\d+)", n):
        mirrors[int(m[1])] = b
    elif re.fullmatch(r"construct-pane-[^-]+", n):
        panes.append(b)
def near(p, q):
    return abs(p[0] - q[0]) <= 1 and abs(p[1] - q[1]) <= 1
def length(p, q):
    return ((p[0] - q[0]) ** 2 + (p[1] - q[1]) ** 2) ** 0.5
shown = {k: b for k, b in faces.items() if b.get("corners")}
if len(shown) < 2:
    print(f"stood back only the faces {sorted(shown)} show")
for k, a in shown.items():
    b = shown.get((k + 1) % 4)
    if b is None:
        continue
    ca, cb = a["corners"], b["corners"]
    edge = [(ca[i], ca[(i + 1) % 4]) for i in range(4)]
    other = [(cb[i], cb[(i + 1) % 4]) for i in range(4)]
    pair = next(((e, o) for e in edge for o in other if (near(e[0], o[0]) and near(e[1], o[1])) or (near(e[0], o[1]) and near(e[1], o[0]))), None)
    if pair is None:
        print(f"the faces {k} and {(k + 1) % 4} share no edge: {ca} and {cb}")
    elif abs(length(*pair[0]) - length(*pair[1])) > 1:
        print(f"the shared edge of the faces {k} and {(k + 1) % 4} differs in length: {pair}")
for k, a in shown.items():
    m = mirrors.get(k)
    if m is None or not m.get("corners"):
        print("face %d has no mirror under it: %s" % (k, m and m["rect"]))
        continue
    fc, mc = a["corners"], m["corners"]
    gap = []
    for top, bottom in ((0, 3), (1, 2)):
        height = fc[bottom][1] - fc[top][1]
        gap.append((mc[bottom][1] - fc[bottom][1]) / height)
        if (abs(mc[top][0] - fc[top][0]) > 1 or abs(mc[bottom][0] - fc[bottom][0]) > 1
                or abs((mc[top][1] - mc[bottom][1]) - height) > 1 or gap[-1] < 0):
            print("the mirror of face %d does not lie under it: %s for %s" % (k, mc, fc))
            break
    else:
        if abs(gap[0] - gap[1]) > 0.02:
            print("the mirror of face %d hangs at another depth at its two edges: %s for %s" % (k, mc, fc))
for i, p in enumerate(panes):
    for q in panes[i + 1:]:
        r, t = p["rect"], q["rect"]
        w = min(r[0] + r[2], t[0] + t[2]) - max(r[0], t[0])
        h = min(r[1] + r[3], t[1] + t[3]) - max(r[1], t[1])
        if w > 1 and h > 1:
            print("%s %s and %s %s overlap" % (p["node"], [round(v) for v in r], q["node"], [round(v) for v in t]))
for p in panes:
    for k, a in shown.items():
        r, q = p["rect"], a["rect"]
        w = min(r[0] + r[2], q[0] + q[2]) - max(r[0], q[0])
        h = min(r[1] + r[3], q[1] + q[3]) - max(r[1], q[1])
        if w > 1 and h > 1 and boxes.index(p) > boxes.index(a):
            print("%s %s lies over face %d %s" % (p["node"], [round(v) for v in r], k, [round(v) for v in q]))
'
}
window_count() {
    call window.list '{}' | python3 -I -c 'import json, sys; print(len(json.load(sys.stdin)["windows"]))'
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
    [ -z "${PULL_RACE:-}" ] || y=$((y + 300))
    until_ok 'ctl query "id:xp-title-$surface" | grep -q VIRTUAL-1' 40 || fail "the window has no title bar"
    inject "[{\"pointer_move\": {\"position\": [$((x + 90)), $((y - 12))]}},
        {\"button\": {\"state\": \"pressed\", \"hold\": true}}]"
    inject '[{"pointer_move": {"position": [900, 300]}}]'
    until_ok 'read -r _ _ _ nx _ <<<"$(probe)"; test "$nx" != "$x"' 20 ||
        fail "the drag of the title bar does not move the window: $(probe)"
    [ "$(owner)" = xp ] || fail "the window left xp inside its face: $(probe)"
    [ -z "$(hint_text)" ] || fail "with a second output the title drag shows the pull hint: $(hint_text)"
    inject "[{\"pointer_move\": {\"position\": [200, 300], \"output\": \"$edge\"}}]"
}
end_session() {
    inject '[{"button": {"state": "released"}}]'
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
}

pair_why() {
    local windows
    windows=$(call window.list '{}' 2>/dev/null)
    ctl query '*' 2>/dev/null | WINDOWS=$windows python3 -I -c '
import json, sys
try:
    nodes = json.load(sys.stdin)
except ValueError:
    nodes = []
import os
try:
    windows = json.loads(os.environ.get("WINDOWS") or "{}").get("windows", [])
except ValueError:
    windows = []
by_name = {w["name"]: w["surface"] for w in windows}
second = by_name.get(sys.argv[1], sys.argv[1])
def line(n):
    surf = n.get("surface") or {}
    r = next(iter((n.get("rects") or {}).values()), None)
    return "%s owner=%s parent=%s visible=%s role=%s size=%s rect=%s" % (
        n["id"], n.get("owner"), n.get("parent"), n.get("visible"),
        surf.get("role") or surf.get("kind"), surf.get("size"),
        [round(v) for v in r] if r else None)
mine = [n for n in nodes if n["id"] == second]
tree = [n for n in nodes if n["id"].startswith(("construct-pane-", "construct-windows")) or n.get("parent") in ("construct-windows",)]
print("[second node: %s; pane tree: %s; surfaces: %s]" % (
    "; ".join(map(line, mine)) or "none",
    "; ".join(map(line, tree)) or "none",
    "; ".join(line(n) for n in nodes if n.get("kind") == "surface") or "none"))' "$1"
    echo "[keyboard $(keyboard_holder)]"
}
keyboard_holder() {
    for look in construct xp; do
        ctl state "$look" | python3 -I -c '
import json, sys
focus = json.load(sys.stdin).get("focus") or {}
print("%s:%s/%s" % (sys.argv[1], focus.get("holder"), focus.get("surface")), end=" ")' "$look"
    done
}
xp_let_go() { ctl state xp | python3 -I -c 'import json, sys; sys.exit(any(w["surface"] == sys.argv[1] for w in json.load(sys.stdin)["windows"]))' "$surface"; }
xp_holds_no_keys() { [ "$(ctl state xp | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["focus"].get("holder"))')" != xp ]; }
pair_cell() {
CHORA_TEST_FLAGS=--test-injected-press-is-real pull pair
until_ok 'test "$(owner)" = construct' 40 || fail "the window did not go to the construct past the edge: $(probe)"
inject "[{\"pointer_move\": {\"position\": [260, 340], \"output\": \"$edge\"}},
    {\"button\": {\"state\": \"released\"}}]"
settle
first=$(pane_name "$surface")
until_ok xp_let_go 20 || fail "xp still lists the window after the drop: $(keyboard_holder)"
until_ok xp_holds_no_keys 20 ||
    fail "after the drop xp holds the keyboard for its icons behind the window of the pane: $(keyboard_holder); the new window goes to the manager that holds it"
"$bin/chora-probe" window 200 40 120 --server-deco >"$work/pair-probe2.log" 2>&1 &
pids+=($!)
until_ok '[ "$(probe | wc -l)" = 2 ]' 80 || fail "the second probe window did not map: $(probe)"
second=$(call window.list '{}' | python3 -I -c '
import json, sys
print(next(w["name"] for w in json.load(sys.stdin)["windows"] if w["surface"] != sys.argv[1]))' "$surface")
until_ok 'test "$(pane_name_of "$second")" != ""' 20 || fail "the second window is no pane: $(call window.list '{}') $(pair_why "$second")"
call pane.move "{\"name\": \"$first\", \"right\": -700}" >/dev/null || fail "pane.move $first"
call pane.resize "{\"name\": \"$first\", \"scale\": 3}" >/dev/null || fail "pane.resize $first"
call pane.move "{\"name\": \"$second\", \"right\": -150}" >/dev/null || fail "pane.move $second"
inject '[{"shortcut": {"keys": "Alt+Escape"}}]'
until_ok "$stood_back" 40 || fail "Alt+Escape did not step back with two panes: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
settle
breaches=$(closed_cube_breaches)
[ -z "$breaches" ] || fail "stood back with two panes the cube is not closed or the panes overlap (rv-1): $breaches"
ok "stood back with two panes on one side the faces meet, the mirrors lie under them and no pane covers a face or the other pane"
end_session
}

if [ -n "${PAIR_ONLY:-}" ]; then
    pair_cell
    exit 0
fi

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
[ -z "${PULL_SHOT:-}" ] || ctl capture "$PULL_SHOT" >/dev/null
inject "[{\"pointer_move\": {\"position\": [220, 320], \"output\": \"$edge\"}},
    {\"button\": {\"state\": \"released\"}}]"
settle
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
until_ok '[ "$(shown_width)" -ge 200 ]' 40 ||
    fail "the pane of the pulled window is not in front of the face at 1:1 or nearer: $(shown_width) px wide for 200"
[ "$(pane_of "$surface")" = shown ] || fail "the pane of the pulled window is not in view: $(pane_of "$surface")"
ok "the pulled window shows as a pane in front of the face, $(shown_width) px wide"
sleep 2
rests=$(node "construct-pane-$surface")
[ "$(node "$surface" | python3 -I -c 'import json, sys; print(json.load(sys.stdin)[0])')" = "construct-pane-$surface" ] ||
    fail "the camera flew into the window after the pull-out: $(node "$surface")"
inject '[{"pointer_move": {"position": [300, 200]}}]'
inject '[{"pointer_move": {"position": [900, 600]}}]'
settle
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

for how in super plain; do
    if [ "$how" = super ]; then
        mod_down='{"key": {"key": "Super_L", "state": "pressed"}},'
        mod_up=',
    {"key": {"key": "Super_L", "state": "released"}}'
    else
        mod_down=''
        mod_up=''
    fi
    CHORA_TEST_FLAGS=--test-injected-press-is-real pull behind
    until_ok 'test "$(owner)" = construct' 40 || fail "the window did not go to the construct past the edge: $(probe)"
    inject "[{\"pointer_move\": {\"position\": [260, 340], \"output\": \"$edge\"}},
        {\"button\": {\"state\": \"released\"}}]"
    settle
    windows_before=$(window_count)
    name=$(pane_name "$surface")
    call pane.move "{\"name\": \"$name\", \"right\": -700}" >/dev/null || fail "pane.move $name"
    call pane.resize "{\"name\": \"$name\", \"scale\": 3}" >/dev/null || fail "pane.resize $name"
    home_rect=$(pane_rect)
    inject '[{"shortcut": {"keys": "Alt+Escape"}}]'
    until_ok "$stood_back" 40 || fail "Alt+Escape did not step back after the pull-out: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
    settle
    [ -z "${BEHIND_SHOT:-}" ] || ctl capture "$BEHIND_SHOT" >/dev/null
    breaches=$(closed_cube_breaches)
    [ -z "$breaches" ] || fail "stood back after the pull-out the cube is not closed (co-21): $breaches"
    ok "stood back after the pull-out the faces meet, the mirrors lie under them and the pane covers no face"
    pane_drawn() {
        ctl query "id:construct-pane-$surface" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
print(bool(nodes and nodes[0].get("visible") and nodes[0]["rects"].get("HEADLESS-1")))'
    }
    [ "$(pane_drawn)" = True ] ||
        fail "stepped back after the pull-out the pane is not drawn: $(ctl query "id:construct-pane-$surface" | tr -d "\n " | cut -c1-900)"
    [ "$(window_count)" = "$windows_before" ] || fail "the window count changed with the step back: $windows_before, then $(window_count)"
    [ "$(owner)" = construct ] || fail "the window is not the construct's after the step back: $(probe)"
    ok "stepped back after the pull-out the pane of the window exists and is drawn, and no window is lost"
    aside_rect=$(pane_rect)
    call view.step_in '{}' >/dev/null || fail "view.step_in"
    until_ok 'test "$(prop construct-cube-face-0 forward)" = True' 40 || fail "view.step_in did not step in: $(prop construct-cube-face-0 forward)"
    settle
    [ "$(pane_rect)" = "$home_rect" ] || fail "the pane is not home after the step in: $(pane_rect), before the step back $home_rect"
    call view.step_back '{}' >/dev/null || fail "view.step_back"
    until_ok "$stood_back" 40 || fail "the second step back did not stand back"
    settle
    [ "$(pane_rect)" = "$aside_rect" ] || fail "the pane is not beside the cube again: $(pane_rect), the first time $aside_rect"
    ok "stepped in again the pane is home at $home_rect, and beside the cube again when stood back"
    read -r px py <<<"$(pane_middle)"
    face=$(ctl query id:construct-cube-face-0 | python3 -I -c '
import json, sys
r = json.load(sys.stdin)[0]["rects"]["HEADLESS-1"]
print("%d %d" % (r[0] + r[2] * 0.4, r[1] + r[3] * 0.5))')
    inject "[{\"pointer_move\": {\"position\": [$px, $py]}},
        $mod_down
        {\"button\": {\"state\": \"pressed\", \"hold\": true}},
        {\"pointer_move\": {\"position\": [$((px + 20)), $py]}},
        {\"pointer_move\": {\"position\": [${face% *}, ${face#* }]}},
        {\"button\": {\"state\": \"released\"}}$mod_up]"
    until_ok 'probe | grep -q " xp VIRTUAL-1 "' 40 ||
        fail "the pane dragged back ($how) over a face stood back did not go to xp: $(probe), pane $(pane_of "$surface")"
    [ "$(pane_of "$surface")" = none ] || fail "the pane stays after the drag back over the face: $(pane_of "$surface")"
    [ "$(window_count)" = "$windows_before" ] || fail "the window count changed with the drag back: $windows_before, then $(window_count)"
    until_ok 'test "$(task_on VIRTUAL-1)" = 1' 20 || fail "the window dragged back over a face has no task button on that face: $(task_on VIRTUAL-1)"
    [ "$(pane_drawn)" != True ] || fail "a pane is still drawn in front of the cube after the drag back: $(ctl query "id:construct-pane-$surface" | tr -d "\n " | cut -c1-600)"
    ok "stood back, the pane dragged ($how) over a face is a window of xp again, on the taskbar of that face, with no pane drawn: $(probe)"
    end_session
done

pair_cell

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
    until_ok "$stood_back" 40 || fail "view.step_back did not stand back: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
    until_ok 'test -n "$(plane_middle)"' 40 || fail "no floating plane of the window"
    settle
    read -r px py <<<"$(plane_middle)"
    inject "[{\"pointer_move\": {\"position\": [$px, $py]}},
        {\"button\": {\"state\": \"pressed\", \"hold\": true}},
        {\"pointer_move\": {\"position\": [$((px + 30)), $py]}},
        {\"pointer_move\": {\"position\": [$((px + 60)), $py]}}]"
}

CHORA_TEST_FLAGS=--test-injected-press-is-real stood plane_beside
[ -z "$(hint_text)" ] || fail "stood back, a plain drag on a plane shows the pull hint: $(hint_text)"
inject '[{"pointer_move": {"position": [1200, 100]}}, {"button": {"state": "released"}}]'
settle
[ "$(owner)" = xp ] || fail "a drag on a floating plane pulled the window out: $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "a drag on a floating plane made a pane: $(pane_of "$surface")"
ok "stood back, a drag on a floating plane pulls nothing out; the cube turns instead: $(probe)"
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
until_ok "$stood_back" 40 || fail "view.step_back did not stand back with the window over the edge: forward $(prop construct-cube-face-0 forward), width $(width construct-cube-face-0)"
until_ok 'test -n "$(plane_middle)"' 40 || fail "no floating plane of the window"
settle
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
settle
[ "$(owner)" = xp ] || fail "the window left xp after the refused give: $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "a refused give made a pane: $(pane_of "$surface")"
ok "after the refused give the window goes on with the drag and stays with xp: $(probe)"
end_session

env_file=$work/hint.env CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
call cube.open '{}' >/dev/null || fail "cube.open"
until_ok 'ctl query id:xp-bar-VIRTUAL-1 | grep -q xp-bar' 80 || fail "xp shows no taskbar on the front face"
inject '[{"pointer_move": {"position": [640, 400]}}]'
"$bin/chora-probe" window 40 120 200 --server-deco >"$work/hint-probe.log" 2>&1 &
pids+=($!)
until_ok 'probe | grep -q " xp VIRTUAL-1 "' 80 || fail "the probe window came to no xp face: $(probe)"
read -r surface _ _ x y <<<"$(probe)"
until_ok 'ctl query "id:xp-title-$surface" | grep -q VIRTUAL-1' 40 || fail "the window has no title bar"
[ -z "$(hint_text)" ] || fail "the pull hint shows before a drag: $(hint_text)"
inject "[{\"pointer_move\": {\"position\": [$((x + 90)), $((y - 12))]}},
    {\"button\": {\"state\": \"pressed\", \"hold\": true}},
    {\"pointer_move\": {\"position\": [600, 300]}},
    {\"pointer_move\": {\"position\": [0, 300]}}]"
case "$(hint_text)" in
*"pull this window out"*) ;;
*) fail "one output: a title drag to the screen edge shows no pull hint: '$(hint_text)'" ;;
esac
[ -z "${HINT_SHOT:-}" ] || ctl capture "$HINT_SHOT" >/dev/null
inject '[{"pointer_move": {"position": [300, 300]}}]'
[ "$(owner)" = xp ] || fail "a one-screen title drag moved the window out of xp: $(probe)"
[ "$(pane_of "$surface")" = none ] || fail "a one-screen title drag made a pane: $(pane_of "$surface")"
inject '[{"button": {"state": "released"}}]'
[ -z "$(hint_text)" ] || fail "the pull hint stays after the drag: $(hint_text)"
ok "one output: a title drag to the edge and back shows the hint \"pull this window out\", moves no window across, and the hint is gone after the drag"
start_menu_open() { ctl query id:xp-item-0 | python3 -I -c 'import json, sys; n = json.load(sys.stdin); sys.exit(0 if n and n[0]["rects"] else 1)'; }
help_shown() { ctl query id:construct-help | python3 -I -c 'import json, sys; n = json.load(sys.stdin); sys.exit(0 if n and n[0].get("opacity") == 1.0 else 1)'; }
with_super() {
    local keys=""
    for k in "$@"; do
        keys="$keys{\"key\": {\"key\": \"$k\", \"state\": \"pressed\"}}, {\"key\": {\"key\": \"$k\", \"state\": \"released\"}}, "
    done
    inject "[{\"key\": {\"key\": \"Super_L\", \"state\": \"pressed\"}}, $keys{\"key\": {\"key\": \"Super_L\", \"state\": \"released\"}}]"
}
with_super
until_ok start_menu_open 20 || fail "rw-1: in use mode a tap of Super does not reach the xp of the face: no Start menu"
inject '[{"key": {"key": "Escape", "state": "pressed"}}, {"key": {"key": "Escape", "state": "released"}}]'
until_ok '! start_menu_open' 20 || fail "Escape does not close the Start menu of the tap"
with_super F1
until_ok help_shown 12 || fail "rw-1: Super+F1 as separate presses shows no key list of the construct"
holds '! start_menu_open' 1 || fail "rw-1: Super with a chord of the construct opens the Start menu of xp, as a lone tap"
with_super F1
until_ok '! help_shown' || fail "a second Super+F1 keeps the key list"
ok "rw-1: in use mode a tap of Super opens the Start menu of the xp of the face, and Super with F1 of the construct opens none"
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
