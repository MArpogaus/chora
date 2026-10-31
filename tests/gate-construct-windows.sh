#!/bin/bash
# Gate of the window list of the construct track (D66): windows of
# other clients as panes on request, and the hand-over back (F29-F31).
# No window gets lost: after each step, every surface that exists shows
# on the output (with an owner, visible, opaque and of a nonzero size) or
# is in the window list, and no surface of another owner has a prop that
# the construct changed.
#
# 1. Overlay over tiling with two foot windows: the load makes no pane
#    for them; window.list shows both; window.show takes one as a pane,
#    flies to it, and typed text arrives; construct.leave with one window
#    flown in and one hidden behind the camera gives both back with their
#    old geometry, and the keyboard to the window that had it.
# 2. Overlay lose cases: pane.remove of an own window that closes and of
#    one that refuses; kill -9 of a taken window while flown in and while
#    hidden; pane.remove of a taken window gives it back; a second
#    construct.load; a load keeps a window that tiling gave to the
#    overlay; kill -9 of chora-construct.
# 3. Flavor: a pane far behind the camera shows in the list, and
#    window.show flies to it; the window list flies to it as well; lose
#    cases as in 2; window.give and the give rows of the list hand panes
#    to tiling; kill -9 of the flavor leaves its windows to tiling.
# 4. One adoption and leave round over chora-xp and over chora-mc.
# 5. Drops with Super: the flavor on one output, tiling on a second and
#    mc. A pane dropped on the tiling output goes to tiling, one dropped
#    on the pane of an mc scene goes to mc, both visible; a drop on empty
#    space only moves the pane; a refused give leaves the pane.
#
# Usage: tests/gate-construct-windows.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the captures as construct-windows-*.png
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-mc chora-tiling chora-xp chora-probe
need foot
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work}
mkdir -p "$work/chora/constructs" "$shots"
# chora-construct starts its programs with `choractl run`.
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
foot=$(command -v foot)

shot() { ctl capture "$@" >/dev/null || fail "capture $*"; }
call() { ctl call construct "$@"; }
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject || fail "inject $1"; }
# shellcheck disable=SC2028 # \n is the JSON escape, for the core
type_text() { echo "[{\"text\": {\"text\": \"$1\\n\"}}]" | ctl inject || fail "inject text"; }
# until_ok CODE [N]: waits up to N/4 s, 10 s by default, for the shell
# code to succeed.
until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
# sh $work/foot.sh NAME: a foot with the app id NAME that writes its pid
# to $work/NAME.pid and a typed line to $work/NAME.typed.
cat >"$work/foot.sh" <<EOF
echo \$\$ >"$work/\$1.pid"
exec "$foot" -a "\$1" -f "DejaVu Sans Mono:size=10" sh -c 'echo \$0; read l; printf %s "\$l" >"$work/\$0.typed"; sleep 600' "\$1"
EOF
foot_bg() {
    sh "$work/foot.sh" "$1" &
    pids+=($!)
}
kill9() { kill -9 "$(cat "$work/$1.pid")" || fail "kill -9 $1"; }
end_foots() {
    local f
    for f in "$work"/*.pid; do
        [ -s "$f" ] && kill "$(cat "$f")" 2>/dev/null
        rm -f "$f"
    done
}
# The surface id of the window with the app id $1.
surface() { ctl query kind:surface | python3 -I -c "import json, sys; print(next((n['id'] for n in json.load(sys.stdin) if n['surface'].get('app_id') == '$1'), ''))"; }
owner() { ctl query "id:$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(n[0].get("owner") if n else "gone")'; }
panes() { call construct.list | python3 -I -c 'import json, sys; print(" ".join(sorted(p["name"] for p in json.load(sys.stdin)["panes"])))'; }
# The pane state of the window $1 in window.list.
state() { call window.list | python3 -I -c "import json, sys; print(next((w['pane'] for w in json.load(sys.stdin)['windows'] if w['name'] == '$1'), 'missing'))"; }
focused() { ctl query "$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(n[0].get("focused") if n else "")'; }
# open_at NAME: opens the window list with Tab and selects the row of
# the window NAME.
open_at() {
    local row
    row=$(call window.list | python3 -I -c "
import json, sys
rows = [w['name'] for w in json.load(sys.stdin)['windows'] if w['pane'] != 'shown']
print(rows.index('$1'))") || fail "$1 is not in the window list"
    key Tab
    ctl wait --timeout 5 "id:construct-list meta:state=open" >/dev/null || fail "Tab opened no list"
    for _ in $(seq "$row"); do key Down; done
}
# The geometry and the props of the surfaces, by app id.
geometry() {
    ctl query kind:surface | python3 -I -c '
import json, sys
for n in sorted(json.load(sys.stdin), key=lambda n: n["surface"].get("app_id") or ""):
    print(n["surface"].get("app_id"), n.get("owner"), n["rects"], n["opacity"], n["visible"], n["transform"])'
}
# no_lost STEP: every surface shows or is in the window list (F31). A
# surface without an owner, one reserved for a client that did not
# attach it, and one of the construct without a pane are lost: the
# window list lists every surface, also these.
no_lost() {
    ctl query kind:surface >"$work/surfaces.json" || fail "$1: query"
    ctl query owner:construct >"$work/nodes.json" 2>/dev/null || echo '[]' >"$work/nodes.json"
    call construct.list >"$work/panes.json" 2>/dev/null || echo '{"panes": []}' >"$work/panes.json"
    call window.list >"$work/windows.json" 2>/dev/null || echo '{"windows": []}' >"$work/windows.json"
    python3 -I - "$work" "$1" <<'EOF' || fail "$1: a window is lost"
import json, sys
work, step = sys.argv[1], sys.argv[2]
load = lambda f: json.load(open(f"{work}/{f}.json"))
nodes = {n["id"]: n for n in load("nodes")}
panes = {p["surface"] for p in load("panes")["panes"] if "surface" in p}
listed = {w["surface"] for w in load("windows")["windows"] if w["pane"] != "shown"}
bad = []
for n in load("surfaces"):
    sid, size = n["id"], n["surface"].get("size") or [0, 0]
    owner = n.get("owner")
    shows = n["visible"] and n["opacity"] > 0 and n["rects"]
    if min(size) <= 0:
        bad.append(f"{sid}: size {size}")
    elif n.get("to"):
        bad.append(f"{sid}: reserved for {n['to']}, which did not attach it")
    elif owner is None:
        bad.append(f"{sid}: no owner")
    elif owner == "construct":
        pane = nodes.get(f"construct-pane-{sid}", {})
        if sid not in panes:
            bad.append(f"{sid}: owned by the construct without a pane")
        elif not (pane.get("visible") and pane.get("rects")) and sid not in listed:
            bad.append(f"{sid}: a pane that neither shows nor is listed")
    elif n["opacity"] <= 0 or not n["visible"]:
        bad.append(f"{sid} of {owner}: opacity {n['opacity']}, visible {n['visible']}")
    elif not shows and sid not in listed:
        bad.append(f"{sid} of {owner}: neither shows nor is listed")
if bad:
    print(f"{step}:", *bad, sep="\n  ", file=sys.stderr)
    sys.exit(1)
EOF
}
# construct NAME [ARGS]: chora-construct in the background, its pid in $cpid.
construct() {
    local name=$1
    shift
    "$bin/chora-construct" "$@" >"$work/construct-$name.log" 2>&1 &
    cpid=$!
    pids+=("$cpid")
}
# session STARTUP: a headless session, its pid in $spid.
session() {
    session_start 1280x800 -- "$@"
    spid=${pids[-1]}
    # shellcheck disable=SC2154 # the trap sets s itself
    trap 's=$?; end_foots; gate_cleanup $s' EXIT
    grant screen.capture input.inject
}
quit() {
    end_foots
    kill "$cpid" "$spid" 2>/dev/null
    wait "$cpid" "$spid" 2>/dev/null
}
# chora-probe refuses to close; `choractl run` starts it outside the
# session, so it writes its pid for end_foots, as foot.sh does.
printf 'echo $$ >"%s/probe.pid"\nexec "%s/chora-probe" window 200 40 40\n' "$work" "$bin" >"$work/probe.sh"
probe="sh $work/probe.sh"
printf 'panes: []\n' >"$work/chora/constructs/empty.yaml"
cat >"$work/chora/constructs/own.yaml" <<EOF
panes:
- kind: program
  command: sh $work/foot.sh o1
  size: [640, 400]
- kind: program
  command: sh $work/foot.sh o2
EOF
cat >"$work/chora/constructs/flav.yaml" <<EOF
radius: 6000
panes:
$(for f in f1 f2 f3 f4 f5 f6; do printf -- '- kind: program\n  command: sh %s/foot.sh %s\n  size: [640, 400]\n' "$work" "$f"; done)
- kind: program
  command: $probe
EOF

# 1. Overlay over tiling.
session "$bin/chora-tiling"
foot_bg t1
until_ok 'test -n "$(surface t1)"' || fail "no t1"
foot_bg t2
until_ok 'test "$(ctl query "kind:surface owner:tiling" | grep -c app_id)" = 2' || fail "tiling took not both windows"
sleep 1
t1=$(surface t1)
t2=$(surface t2)
before=$(geometry)
had=$(ctl query kind:surface | python3 -I -c 'import json, sys; print(next(n["surface"]["app_id"] for n in json.load(sys.stdin) if n.get("focused")))')
construct overlay --overlay
until_ok 'call construct.list >/dev/null 2>&1' || fail "the overlay does not answer"
call construct.load '{"name": "empty"}' >/dev/null || fail "construct.load"
ctl wait --timeout 10 id:construct-look >/dev/null || fail "no space over tiling"
sleep 1
[ -z "$(panes)" ] || fail "the load made panes: $(panes)"
[ "$(state t1)/$(state t2)" = none/none ] || fail "window.list: t1 $(state t1), t2 $(state t2)"
no_lost "overlay load"
ok "the overlay load made no pane for the tiling windows; window.list shows both without a pane"

key Tab
ctl wait --timeout 5 "id:construct-list meta:state=open" >/dev/null || fail "Tab opened no list"
key Down
sleep 0.5
shot "$shots/construct-windows-overlay.png"
key Escape
ctl wait --gone --timeout 5 "id:construct-list meta:state=open" >/dev/null || fail "Escape left the list open"
ok "Tab opened the window list, Escape closed it"

call window.show '{"name": "t2"}' >/dev/null || fail "window.show t2"
until_ok 'test "$(panes)" = t2' || fail "t2 became no pane: $(panes)"
sleep 1
call pane.move '{"name": "t2", "ahead": -4000}' >/dev/null || fail "pane.move t2"
call window.show '{"name": "t1"}' >/dev/null || fail "window.show t1"
sleep 1.5
[ "$(state t2)" != shown ] || fail "t2 behind the camera is shown"
type_text "hello t1"
until_ok 'test "$(cat "$work/t1.typed" 2>/dev/null)" = "hello t1"' || fail "t1 got: $(cat "$work/t1.typed" 2>/dev/null)"
[ "$(owner "$t1")/$(owner "$t2")" = tiling/tiling ] || fail "the owners changed: $(owner "$t1") $(owner "$t2")"
no_lost "overlay show"
shot "$shots/construct-windows-overlay-flown.png"
ok "window.show took t1 as a pane and flew to it, and the text arrived; t2 waits behind the camera"

call construct.leave >/dev/null || fail "construct.leave"
ctl wait --gone --timeout 5 owner:construct >/dev/null || fail "construct nodes stay"
sleep 1
after=$(geometry)
[ "$after" = "$before" ] || fail "the windows changed: before
$before
after
$after"
until_ok 'test "$(focused "id:$(surface "$had")")" = True' || fail "the keyboard did not go back to $had"
type_text "back"
until_ok 'test "$(cat "$work/$had.typed" 2>/dev/null)" = back' || fail "$had got no text after the leave"
no_lost "overlay leave"
shot "$shots/construct-windows-left.png"
ok "construct.leave gave back t1 (flown in) and t2 (hidden) with their geometry, and the keyboard to $had"

# 2. Overlay lose cases.
foot_bg k1
foot_bg k2
until_ok 'test -n "$(surface k2)"' || fail "no k2"
call construct.load '{"name": "own"}' >/dev/null || fail "construct.load own"
until_ok 'test "$(panes)" = "o1 o2"' || fail "own: the panes are $(panes)"
no_lost "overlay own"
call pane.remove '{"name": "o1"}' >/dev/null || fail "pane.remove o1"
until_ok 'test "$(panes)" = o2' || fail "o1 did not close: $(panes)"
no_lost "overlay remove o1"
ok "overlay: pane.remove closed the own window o1"

call window.show '{"name": "k1"}' >/dev/null || fail "window.show k1"
sleep 1.5
kill9 k1
until_ok 'test "$(panes)" = o2' || fail "the pane of the killed k1 stays: $(panes)"
# The owner may give the keyboard to another of its windows; a press on
# the space takes it back.
sleep 1
echo '[{"pointer_move": {"position": [4, 4]}}, {"click": {}}]' | ctl inject || fail "inject click"
until_ok 'test "$(focused id:construct-look)" = True' || fail "the space has no keyboard after k1 died and a click"
no_lost "overlay kill flown"
call window.show '{"name": "k2"}' >/dev/null || fail "window.show k2"
call pane.move '{"name": "k2", "ahead": -4000}' >/dev/null || fail "pane.move k2"
sleep 1
kill9 k2
until_ok 'test "$(panes)" = o2' || fail "the pane of the killed k2 stays: $(panes)"
no_lost "overlay kill hidden"
ok "overlay: kill -9 of a taken window flown in and hidden removed its pane"

call window.show '{"name": "t1"}' >/dev/null || fail "window.show t1 again"
sleep 1.5
call pane.remove '{"name": "t1"}' >/dev/null || fail "pane.remove t1"
until_ok 'test "$(panes)" = o2' || fail "t1 stays a pane: $(panes)"
[ "$(owner "$t1")" = tiling ] || fail "t1 is not with tiling: $(owner "$t1")"
no_lost "overlay give back t1"
call window.show '{"name": "t2"}' >/dev/null || fail "window.show t2 again"
call construct.load '{"name": "empty"}' >/dev/null || fail "the second construct.load"
until_ok 'test -z "$(panes)"' || fail "the second load kept panes: $(panes)"
[ "$(owner "$t2")" = tiling ] || fail "t2 is not with tiling: $(owner "$t2")"
[ "$(owner "$(surface o2)")" = tiling ] || fail "the own window o2 is not with tiling: $(owner "$(surface o2)")"
no_lost "overlay second load"
ok "overlay: pane.remove gave t1 back; a second load gave t2 back and left the own o2 to tiling"

ctl call tiling window.give "{\"surface\": \"$t1\", \"to\": \"construct\"}" >/dev/null || fail "tiling window.give t1"
until_ok 'panes | grep -qw t1' || fail "the given t1 became no pane: $(panes)"
[ "$(owner "$t1")" = construct ] || fail "t1 is not with the overlay: $(owner "$t1")"
call construct.load '{"name": "empty"}' >/dev/null || fail "the load after the give"
sleep 1
panes | grep -qw t1 || fail "the load dropped the given t1: $(panes)"
[ "$(owner "$t1")" = construct ] || fail "the load moved t1 to $(owner "$t1")"
no_lost "overlay load after a give"
ok "overlay: a window given to it stays its pane over a load"

call window.show '{"name": "t1"}' >/dev/null || fail "window.show t1 before the kill"
call window.show '{"name": "t2"}' >/dev/null || fail "window.show t2 before the kill"
key Tab
sleep 1
kill -9 "$cpid"
wait "$cpid" 2>/dev/null
# The core keeps the nodes of a plugin with resume for 10 s.
until_ok 'test "$(ctl query owner:construct)" = "[]"' 80 || fail "construct nodes stay after the kill"
until_ok 'test "$(owner "$t1")/$(owner "$t2")" = tiling/tiling' || fail "after the kill: $(owner "$t1") $(owner "$t2")"
no_lost "overlay kill -9 construct"
ok "kill -9 of the overlay left every window shown and with tiling"
quit

# 3. Flavor.
session
construct flavor
ctl wait --timeout 10 id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$work/construct-flavor.log")"
call construct.load '{"name": "flav"}' >/dev/null || fail "construct.load flav"
until_ok 'test "$(panes)" = "f1 f2 f3 f4 f5 f6"' 80 || fail "flav: the panes are $(panes); windows: $(geometry)"
# The probe comes without the token of the file, as a free window; the
# view is full, so it waits in the window list (C7) until a choice.
until_ok 'test "$(state chora-probe)" = none' || fail "the probe is $(state chora-probe), not in the list"
call window.show '{"name": "chora-probe"}' >/dev/null || fail "window.show chora-probe"
sleep 1.5
key Alt+Escape
until_ok 'test "$(panes)" = "chora-probe f1 f2 f3 f4 f5 f6"' || fail "flav: the panes are $(panes)"
sleep 1
no_lost "flavor load"
call pane.move '{"name": "f1", "ahead": -6000}' >/dev/null || fail "pane.move f1"
sleep 0.5
[ "$(state f1)" = behind ] || fail "f1 far behind the camera is $(state f1)"
open_at f1
sleep 0.5
shot "$shots/construct-windows-flavor.png"
key Return
sleep 1.5
[ "$(state f1)" = shown ] || fail "the list did not fly to f1: $(state f1)"
key Alt+Escape
sleep 1
call pane.move '{"name": "f1", "ahead": -6000}' >/dev/null || fail "pane.move f1 again"
sleep 0.5
call window.show '{"name": "f1"}' >/dev/null || fail "window.show f1"
sleep 1.5
[ "$(state f1)" = shown ] || fail "window.show did not fly to f1: $(state f1)"
type_text "hello f1"
until_ok 'test "$(cat "$work/f1.typed" 2>/dev/null)" = "hello f1"' || fail "f1 got: $(cat "$work/f1.typed" 2>/dev/null)"
key Alt+Escape
sleep 1
no_lost "flavor show"
ok "flavor: f1 far behind the camera is in the list; Return and window.show flew to it, and text arrived"

call pane.remove '{"name": "f2"}' >/dev/null || fail "pane.remove f2"
until_ok '! panes | grep -qw f2' || fail "f2 did not close"
no_lost "flavor remove f2"
call pane.remove '{"name": "chora-probe"}' >/dev/null || fail "pane.remove chora-probe"
sleep 1
panes | grep -qw chora-probe || fail "the probe that refuses lost its pane"
no_lost "flavor remove probe"
kill9 f3
until_ok '! panes | grep -qw f3' || fail "the pane of the killed f3 stays"
no_lost "flavor kill pane"
call window.show '{"name": "f4"}' >/dev/null || fail "window.show f4"
sleep 1.5
kill9 f4
until_ok '! panes | grep -qw f4' || fail "the pane of the killed f4 stays"
until_ok 'test "$(focused id:construct-look)" = True' || fail "the space has no keyboard after f4 died"
no_lost "flavor kill flown"
call pane.move '{"name": "f5", "ahead": -6000}' >/dev/null || fail "pane.move f5"
sleep 0.5
kill9 f5
until_ok '! panes | grep -qw f5' || fail "the pane of the killed f5 stays"
no_lost "flavor kill hidden"
call construct.load '{"name": "empty"}' >/dev/null || fail "construct.load empty"
sleep 1
[ "$(panes)" = "chora-probe f1 f6" ] || fail "the flavor lost window panes on a load: $(panes)"
no_lost "flavor second load"
ok "flavor: pane.remove closed f2, the probe refused; kill -9 flown, hidden and shown; a second load kept the windows"

"$bin/chora-tiling" >"$work/tiling.log" 2>&1 &
pids+=($!)
sleep 2
# The list first: the window that tiling takes then takes the keyboard.
probe_id=$(surface chora-probe)
call pane.move '{"name": "chora-probe", "ahead": -6000}' >/dev/null || fail "pane.move probe"
sleep 0.5
open_at chora-probe
key g
sleep 0.5
shot "$shots/construct-windows-give.png"
key Return
until_ok 'test "$(owner "$probe_id")" = tiling' || fail "the give row did not give the probe to tiling: $(owner "$probe_id")"
no_lost "flavor give probe"
f1=$(surface f1)
given=$(call window.give "{\"surface\": \"$f1\", \"to\": \"tiling\"}") || fail "window.give f1"
[ "$given" = "\"$f1\"" ] || fail "window.give answered $given, not the surface id"
until_ok 'test "$(owner "$f1")" = tiling' || fail "f1 is not with tiling: $(owner "$f1")"
panes | grep -qw f1 && fail "f1 stays a pane"
no_lost "flavor give f1"
ok "the give rows of the list and window.give handed chora-probe and f1 to tiling"

f6=$(surface f6)
kill -9 "$cpid"
wait "$cpid" 2>/dev/null
# After the grace time of resume (10 s), the core frees the windows.
until_ok 'test "$(owner "$f6")" = tiling' 80 || fail "f6 is not with tiling after the kill: $(owner "$f6")"
no_lost "flavor kill -9 construct"
ok "kill -9 of the flavor freed f6, and tiling took it"
quit

# 4. Over chora-xp and chora-mc: one adoption and leave round each.
round() {
    local wm=$1 id
    sleep 1
    before=$(geometry)
    construct "over-$wm" --overlay
    until_ok 'call construct.list >/dev/null 2>&1' || fail "$wm: the overlay does not answer"
    call construct.load '{"name": "empty"}' >/dev/null || fail "$wm: construct.load"
    id=$(call window.list | python3 -I -c "import json, sys; print(next(w['surface'] for w in json.load(sys.stdin)['windows'] if w['owner'] == '$wm'))") ||
        fail "$wm: window.list has no window of $wm"
    call window.show "{\"name\": \"$id\"}" >/dev/null || fail "$wm: window.show"
    sleep 1.5
    [ -n "$(panes)" ] || fail "$wm: no pane"
    no_lost "$wm show"
    call construct.leave >/dev/null || fail "$wm: construct.leave"
    ctl wait --gone --timeout 5 owner:construct >/dev/null || fail "$wm: construct nodes stay"
    sleep 1
    [ "$(geometry)" = "$before" ] || fail "$wm: the windows changed: before
$before
after
$(geometry)"
    no_lost "$wm leave"
    ok "over $wm: window.show took its window as a pane, and construct.leave gave it back as it was"
    quit
}
session "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
foot_bg x1
until_ok 'test "$(owner "$(surface x1)")" = xp' || fail "xp took no window"
round xp
session "$bin/chora-mc"
ctl wait --timeout 10 'owner:mc meta:pane=mission' >/dev/null || fail "mc shows no page"
ctl call mc scene.show '{"name": "terminal"}' >/dev/null || fail "scene.show terminal"
until_ok 'ctl query "kind:surface owner:mc" | grep -q app_id' || fail "mc took no window"
round mc

# 5. Drops with Super (D55): the flavor on the first output, tiling on a
# second.
cat >"$work/chora/constructs/drop.yaml" <<EOF
radius: 5000
panes:
- kind: program
  command: sh $work/foot.sh d1
  at: [-700, 1800]
  size: [480, 300]
- kind: program
  command: sh $work/foot.sh d2
  at: [-100, 1800]
  size: [480, 300]
- kind: program
  command: sh $work/foot.sh d3
  at: [-700, 3200]
  size: [480, 300]
- kind: plugin
  plugin: mc
  scene: mission
  at: [900, 2600]
EOF
session
construct drop
ctl wait --timeout 10 id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$work/construct-drop.log")"
ctl add-output 1280x800+1280+0 >/dev/null || fail "add-output"
mapfile -t outs < <(ctl outputs | python3 -I -c 'import json, sys; print(*[o["name"] for o in sorted(json.load(sys.stdin), key=lambda o: o["position"][0])], sep="\n")')
"$bin/chora-mc" --output "${outs[0]}" >"$work/mc.log" 2>&1 &
pids+=($!)
"$bin/chora-tiling" --output "${outs[1]}" >"$work/tiling.log" 2>&1 &
pids+=($!)
ctl wait --timeout 10 'owner:mc meta:pane=mission' >/dev/null || fail "mc shows no page"
ctl wait --timeout 10 owner:tiling >/dev/null || fail "tiling does not start"
call construct.load '{"name": "drop"}' >/dev/null || fail "construct.load drop"
until_ok 'test "$(panes)" = "d1 d2 d3 mission"' 80 || fail "drop: the panes are $(panes)"
sleep 1
no_lost "drop load"
# The centre of the pane node $1 on the first output.
centre() { ctl query "id:$1" | python3 -I -c 'import json, sys; r = list(json.load(sys.stdin)[0]["rects"].values())[0]; print(round(r[0] + r[2] / 2), round(r[1] + r[3] / 2), sep=", ")'; }
# super_drag FROM OUTPUT TO: a drag with Super and the left button from
# FROM on the first output to TO on OUTPUT.
super_drag() {
    echo "[{\"pointer_move\": {\"position\": [$1], \"output\": \"${outs[0]}\"}},
  {\"key\": {\"key\": \"Super_L\", \"state\": \"pressed\"}},
  {\"button\": {\"state\": \"pressed\", \"hold\": true}},
  {\"pointer_move\": {\"position\": [$3], \"output\": \"$2\"}},
  {\"button\": {\"state\": \"released\"}},
  {\"key\": {\"key\": \"Super_L\", \"state\": \"released\"}}]" | ctl inject >/dev/null || fail "inject the drag"
}
# shown ID OWNER: the window shows, and OWNER owns it.
shown() {
    ctl query "id:$1" | python3 -I -c "import json, sys; n = json.load(sys.stdin); sys.exit(not (n and n[0].get('owner') == '$2' and n[0]['visible'] and n[0]['opacity'] > 0 and n[0]['rects']))"
}
at_of() { call construct.list | python3 -I -c "import json, sys; print(next(p['at'] for p in json.load(sys.stdin)['panes'] if p['name'] == '$1'))"; }
drop_shot() { for o in "${outs[@]}"; do shot "$shots/construct-windows-drop-$1-$o.png" --output "$o"; done; }
drop_shot 0-load

d1=$(surface d1)
super_drag "$(centre "construct-pane-$d1")" "${outs[1]}" "640, 400"
until_ok 'shown "$d1" tiling' || fail "d1 is not shown with tiling: $(owner "$d1")"
panes | grep -qw d1 && fail "d1 stays a pane"
no_lost "drop on tiling"
drop_shot 1-tiling
ok "a Super drag of the pane d1 to the tiling output: tiling owns d1, and it shows"

d2=$(surface d2)
super_drag "$(centre "construct-pane-$d2")" "${outs[0]}" "$(centre construct-plugin-mission)"
until_ok 'shown "$d2" mc' || fail "d2 is not shown with mc: $(owner "$d2")"
panes | grep -qw d2 && fail "d2 stays a pane"
no_lost "drop on mc"
drop_shot 2-mc
ok "a Super drag of the pane d2 onto the mc pane: mc owns d2"

d3=$(surface d3)
before=$(at_of d3)
from=$(centre "construct-pane-$d3")
super_drag "$from" "${outs[0]}" "$((${from%,*} - 150)), ${from#*, }"
sleep 1
after=$(at_of d3)
[ "$after" != "$before" ] || fail "the drop on empty space did not move d3: $after"
[ "$(owner "$d3")" = construct ] || fail "the drop on empty space gave d3 to $(owner "$d3")"
no_lost "drop on space"
drop_shot 3-space
ok "a Super drag of d3 onto empty space moved the pane ($before -> $after) and gave nothing"

if call window.give "{\"surface\": \"$d3\", \"to\": \"nobody\"}" >/dev/null 2>&1; then
    fail "a give to nobody was accepted"
fi
panes | grep -qw d3 || fail "the refused give removed the pane d3"
[ "$(owner "$d3")" = construct ] || fail "the refused give moved d3 to $(owner "$d3")"
no_lost "drop refused"
drop_shot 4-refused
ok "a refused give left the pane d3 with the construct"
quit
