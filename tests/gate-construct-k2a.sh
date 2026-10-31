#!/bin/bash
# Gate of step K2a of the construct track: construct files, plugin panes
# and the overlay.
#
# 1. chora-mc runs, and chora-construct runs beside it as the flavor. The
#    construct k2a with a foot pane and a pane of the mc scene mission
#    loads, and both panes show within 10 s.
# 2. Panes move and resize by method and by Ctrl with a drag and the
#    wheel; the construct saves; the session quits. A new session starts
#    the flavor into the saved construct, with the same poses.
# 3. chora-tiling runs with a window; chora-construct --overlay loads a
#    construct over it. The windows stay with tiling, the cameras stay.
#    A fly-in gives the keyboard to the pane's window through its owner
#    (D31), and typed text arrives. construct.leave gives the output
#    back: the space is gone, and the cameras are as before.
#
# Usage: tests/gate-construct-k2a.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the captures as construct-k2a-*.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-mc chora-tiling
need foot
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work}
mkdir -p "$work/chora/constructs" "$shots"
# chora-construct starts its programs with `choractl run`.
export PATH=$bin:$PATH
# A debug core on the software renderer is slow for the fog and the
# panes; the watchdog would take their shaders away (as K1).
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
foot=$(command -v foot)

shot() { ctl capture "$@" >/dev/null || fail "capture $*"; }
call() { ctl call construct "$@"; }
# The JSON of the first node of a selector, as python sees it.
node() { ctl query "$1" | python3 -I -c "import json, sys; n = json.load(sys.stdin); print($2 if n else '')"; }
# The panes of construct.list, sorted, with poses rounded to 0.01.
poses() {
    call construct.list | python3 -I -c '
import json, sys
a = json.load(sys.stdin)
for p in sorted(a["panes"], key=lambda p: p["name"]):
    print(p["name"], p["kind"], [round(v, 2) for v in p["at"] + p["size"]])'
}
# construct NAME [ARGS]: chora-construct in the background, its log in
# $work/construct-NAME.log, its pid in $cpid.
construct() {
    local name=$1
    shift
    "$bin/chora-construct" "$@" >"$work/construct-$name.log" 2>&1 &
    cpid=$!
    pids+=("$cpid")
}
# sh $work/foot.sh NAME: a foot that writes its pid to $work/NAME.pid
# and a typed line to $work/NAME.typed. The construct starts it with
# `choractl run`, outside the session, so the gate ends it by its pid.
cat >"$work/foot.sh" <<EOF
echo \$\$ >"$work/\$1.pid"
exec "$foot" -f "DejaVu Sans Mono:size=10" sh -c 'echo construct K2a; read l; printf %s "\$l" >"$work/\$0.typed"; sleep 600' "\$1"
EOF
end_foot() {
    [ -s "$work/$1.pid" ] && kill "$(cat "$work/$1.pid")" 2>/dev/null
    rm -f "$work/$1.pid"
}
# session STARTUP: a headless session, its pid in $spid.
session() {
    session_start 1280x800 -- "$@"
    spid=${pids[-1]}
    # shellcheck disable=SC2154 # the trap sets s itself
    trap 's=$?; end_foot k2a; end_foot over; end_foot tiling; gate_cleanup $s' EXIT
    grant screen.capture input.inject
}
# quit: ends the construct, its foot and the session.
quit() {
    end_foot k2a
    kill "$cpid" "$spid"
    wait "$cpid" "$spid" 2>/dev/null
}

cat >"$work/chora/constructs/k2a.yaml" <<EOF
camera: {x: 0, z: 0, yaw: 0, pitch: 0}
radius: 5000
panes:
- kind: program
  command: sh $work/foot.sh k2a
  at: [-650, 2000]
  size: [1000, 640]
- kind: plugin
  plugin: mc
  scene: mission
  at: [700, 2200]
EOF

# 1. Load beside mc.
session "$bin/chora-mc"
ctl wait --timeout 10 'owner:mc meta:pane=mission' >/dev/null || fail "mc tags no page"
construct flavor
ctl wait --timeout 10 id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$work/construct-flavor.log")"
start=$(date +%s.%N)
call construct.load '{"name": "k2a"}' >/dev/null || fail "construct.load"
ctl wait --timeout 10 'kind:surface owner:construct' >/dev/null || fail "foot became no pane"
surface=$(node 'kind:surface owner:construct' 'n[0]["id"]')
for pane in "construct-pane-$surface" construct-plugin-mission; do
    ctl wait --timeout 10 "id:$pane" >/dev/null || fail "no pane $pane"
done
took=$(python3 -I -c "import time; print(round(time.time() - $start, 1))")
sleep 1
for pane in "construct-pane-$surface" construct-plugin-mission; do
    [ -n "$(node "id:$pane" 'n[0]["rects"]')" ] || fail "the pane $pane does not show"
done
python3 -I -c "import sys; sys.exit(not $took < 10)" || fail "the panes took $took s"
shot "$shots/construct-k2a-load.png"
ok "construct.load showed a foot pane and the mc scene mission within $took s"

# 2. Change the poses, save, quit, start again.
call pane.move '{"name": "foot", "right": -100, "ahead": 300}' >/dev/null || fail "pane.move"
call pane.resize '{"name": "foot", "scale": 1.2}' >/dev/null || fail "pane.resize"
centre=$(node id:construct-plugin-mission '"%d, %d" % (lambda r: (r[0] + r[2] / 2, r[1] + r[3] / 2))(list(n[0]["rects"].values())[0])')
before=$(poses | grep '^mission')
echo "[{\"key\": {\"key\": \"Control_L\", \"state\": \"pressed\"}},
  {\"pointer_move\": {\"position\": [$centre]}}, {\"button\": {\"state\": \"pressed\"}},
  {\"pointer_move\": {\"position\": [${centre%,*}, 300]}}, {\"button\": {\"state\": \"released\"}},
  {\"scroll\": {\"dy\": -40}}, {\"key\": {\"key\": \"Control_L\", \"state\": \"released\"}}]" |
    ctl inject || fail "inject the drag"
sleep 1
after=$(poses | grep '^mission')
[ "$after" != "$before" ] || fail "a drag and the wheel with Ctrl changed nothing: $after"
[ "$(node 'kind:surface owner:construct' 'n[0]["opacity"]')" = 0.0 ] || fail "the drag flew into a pane"
ok "Ctrl with a drag and the wheel moved the mc pane: $before -> $after"
sleep 1
saved=$(poses)
shot "$shots/construct-k2a-moved.png"
call construct.save '{"name": "k2a-saved"}' >/dev/null || fail "construct.save"
[ -s "$work/chora/constructs/k2a-saved.yaml" ] || fail "no saved file"
quit
printf 'start: k2a-saved\n' >"$work/chora/chora-construct.yaml"
session "$bin/chora-mc"
construct restart
ctl wait --timeout 10 'kind:surface owner:construct' >/dev/null || fail "the restart started no foot"
ctl wait --timeout 10 id:construct-plugin-mission >/dev/null || fail "the restart shows no mc pane"
sleep 2
again=$(poses)
[ "$again" = "$saved" ] || fail "the poses changed: saved
$saved
after the restart
$again"
shot "$shots/construct-k2a-restart.png"
ok "after save, quit and a new start, the poses are the same: $(echo "$saved" | tr '\n' ';')"
quit
rm "$work/chora/chora-construct.yaml"

# 3. Overlay over tiling.
cat >"$work/chora/constructs/over.yaml" <<EOF
panes:
- kind: program
  command: sh $work/foot.sh over
  at: [0, 2000]
EOF
session "$bin/chora-tiling"
sh "$work/foot.sh" tiling &
ctl wait --timeout 10 'kind:surface owner:tiling' >/dev/null || fail "tiling took no window"
sleep 1
cameras=$(ctl outputs)
shot "$shots/construct-k2a-tiling.png"
construct overlay --overlay
sleep 1
[ -z "$(ctl query 'owner:construct' | python3 -I -c 'import json, sys; print(json.load(sys.stdin) or "")')" ] ||
    fail "the overlay shows before construct.load"
call construct.load '{"name": "over"}' >/dev/null || fail "construct.load over tiling: $(tail -n 3 "$work/construct-overlay.log")"
ctl wait --timeout 10 id:construct-look >/dev/null || fail "no space over tiling"
pane=
for _ in $(seq 100); do
    pane=$(call construct.list | python3 -I -c 'import json, sys; p = json.load(sys.stdin)["panes"]; print(p[0]["name"] if p else "")')
    [ -n "$pane" ] && break
    sleep 0.1
done
[ -n "$pane" ] || fail "the foot of the overlay became no pane"
sleep 1
owners=$(ctl query kind:surface | python3 -I -c 'import json, sys; print(sorted({n["owner"] for n in json.load(sys.stdin)}))')
[ "$owners" = "['tiling']" ] || fail "the windows belong to $owners, not to tiling alone"
[ "$(ctl outputs)" = "$cameras" ] || fail "the overlay changed the outputs: $(ctl outputs)"
shot "$shots/construct-k2a-overlay.png"
ok "construct.load over tiling shows the pane $pane; tiling keeps the windows and the cameras"

pane_rects() {
    ctl query owner:construct | python3 -I -c '
import json, sys
print([n["rects"] for n in json.load(sys.stdin) if n["id"].startswith("construct-pane-")])'
}
walk=$(pane_rects)
call pane.fly "{\"name\": \"$pane\"}" >/dev/null || fail "pane.fly"
sleep 1.5
# shellcheck disable=SC2028 # \n is the JSON escape, for the core
echo '[{"text": {"text": "hello overlay\n"}}]' | ctl inject || fail "inject text"
for _ in $(seq 30); do
    [ -s "$work/over.typed" ] && break
    sleep 0.1
done
[ "$(cat "$work/over.typed" 2>/dev/null)" = "hello overlay" ] ||
    fail "the foot of the overlay got: $(cat "$work/over.typed" 2>/dev/null)"
[ ! -s "$work/tiling.typed" ] || fail "the other tiling window got the text"
sleep 0.5
shot "$shots/construct-k2a-flown.png"
ok "after pane.fly, tiling gave the keyboard to the window of the pane, and the text arrived"

# A click into the pane that the camera is in keeps the way back.
echo '[{"pointer_move": {"position": [640, 400]}}, {"click": {}}]' | ctl inject || fail "inject click"
sleep 1.5
echo '[{"shortcut": {"keys": "Alt+Escape"}}]' | ctl inject || fail "inject Alt+Escape"
sleep 1
[ "$(node id:construct-look 'n[0]["focused"]')" = True ] || fail "the space has no keyboard after the fly-out"
[ "$(pane_rects)" = "$walk" ] || fail "after a click inside, the fly-out went to $(pane_rects), not back to $walk"
ok "after a click inside the pane, Alt+Escape flew back to the pose before"
call construct.leave >/dev/null || fail "construct.leave"
ctl wait --gone --timeout 5 owner:construct >/dev/null || fail "construct nodes stay: $(ctl query owner:construct)"
[ "$(ctl outputs)" = "$cameras" ] || fail "the cameras did not come back: $(ctl outputs)"
sleep 0.5
shot "$shots/construct-k2a-left.png"
ok "construct.leave gave the output back; the cameras are as before"
