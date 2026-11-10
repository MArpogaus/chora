#!/bin/bash
# Gate of step K5 of the construct track: direct input on the 3D base
# (design/3d.md, section 5). Four probe windows stand as program panes of
# a construct file, three of them turned:
#
# 1. A click on a tilted pane reaches its window at the point that shows
#    under the pointer, and the view does not move.
# 2. Typed keys go to the pane that was clicked last.
# 3. Where two panes overlap on the output, the nearer one gets the click.
# 4. Navigate mode keeps its grab: a drag with the modifier over a pane
#    reaches no window and moves the pane; after a click on the empty
#    space the walk keys move the camera and reach no window.
#
# Each check reports on its own; the gate fails when one of them fails.
# panes_spec gives the name, centre, yaw, size and colour of each pane;
# pane c stands in front of d. `screen NAME U V` projects a point of a
# pane for the camera at the origin, `behind NAME X Y` tells whether a
# pane lies under a point of the output, and `pressed_at NAME U V`
# whether the last press of a pane lies within 1.5 pixels of (U, V).
#
# Usage: tests/gate-construct-3d.sh   BIN: directory with the binaries
#   SHOT  a PNG path that keeps a capture of the four panes
# shellcheck disable=SC2016,SC2034 # check and until_ok evaluate their code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-probe
temp_config
work=$XDG_CONFIG_HOME
mkdir -p "$work/chora/constructs"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}

panes_spec='a:-700,2000:0.7:480,360:200,60,60
b:700,2000:-0.7:480,360:60,160,60
c:0,1400:0.25:400,300:60,60,200
d:60,2600:-0.2:640,400:200,200,60'
{
    echo "camera: {x: 0, z: 0, yaw: 0, pitch: 0}"
    echo "radius: 6000"
    echo "panes:"
    while IFS=: read -r name at yaw size rgb; do
        echo "  - kind: program"
        echo "    command: exec chora-probe window ${rgb//,/ } --report >$work/$name.log"
        echo "    at: [$at]"
        echo "    yaw: $yaw"
        echo "    size: [$size]"
    done <<<"$panes_spec"
} >"$work/chora/constructs/k5.yaml"

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture

failed=0
check() {
    if eval "$2"; then
        ok "$1"
    else
        echo "FAIL: $1" >&2
        failed=1
    fi
}
until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
call() { ctl call construct "$@"; }
inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
pane_count() { call construct.list | python3 -I -c 'import json, sys; print(len(json.load(sys.stdin)["panes"]))'; }
screen() {
    python3 -I - "$panes_spec" "$@" <<'EOF'
import math, sys
spec, name, u, v = sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4])
focal, cx, cy = 1280 * 0.65, 640, 400
for line in spec.splitlines():
    n, at, yaw, size, _ = line.split(":")
    if n != name:
        continue
    ax, az = map(float, at.split(","))
    w, h = map(float, size.split(","))
    yaw = float(yaw)
    rx, rz = math.cos(yaw), -math.sin(yaw)
    x, y, z = ax + rx * (u - w / 2), h / 2 - v, az + rz * (u - w / 2)
    print(f"{cx + focal * x / z:.2f}, {cy - focal * y / z:.2f}")
EOF
}
behind() {
    python3 -I - "$panes_spec" "$@" <<'EOF'
import math, sys
spec, name, sx, sy = sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4])
focal, cx, cy = 1280 * 0.65, 640, 400
for line in spec.splitlines():
    n, at, yaw, size, _ = line.split(":")
    if n != name:
        continue
    ax, az = map(float, at.split(","))
    w, h = map(float, size.split(","))
    yaw = float(yaw)
    d = ((sx - cx) / focal, -(sy - cy) / focal, 1.0)
    nx, nz = math.sin(yaw), math.cos(yaw)
    t = (ax * nx + az * nz) / (d[0] * nx + d[2] * nz)
    px, py, pz = d[0] * t - ax, d[1] * t, d[2] * t - az
    u = px * math.cos(yaw) - pz * math.sin(yaw) + w / 2
    v = h / 2 - py
    sys.exit(0 if 0 <= u <= w and 0 <= v <= h else 1)
EOF
}
click_at() { inject "[{\"pointer_move\": {\"position\": [$1]}}, {\"click\": {}}]"; }
presses() { grep -c '^press' "$work/$1.log"; }
keys() { grep -c '^key' "$work/$1.log"; }
pressed_at() {
    grep '^press' "$work/$1.log" | tail -n 1 |
        awk -v u="$2" -v v="$3" '{ dx = $2 - u; dy = $3 - v; near = dx * dx + dy * dy <= 2.25 } END { exit !near }'
}
camera() {
    call construct.save '{"name": "k5-now"}' >/dev/null || fail "construct.save"
    sed -n 's/^camera: *//p; /^camera:$/,/^[a-z]/p' "$work/chora/constructs/k5-now.yaml" | tr -d ' \n'
}
pane_at() { call construct.list | python3 -I -c "import json, sys; print(next(p['at'] for p in json.load(sys.stdin)['panes'] if p['name'] == '$1'))"; }
name_of() { call construct.list | python3 -I -c "import json, sys; print(next(p['name'] for p in json.load(sys.stdin)['panes'] if [round(x) for x in p['at']] == [$1]))"; }

ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
call construct.load '{"name": "k5"}' >/dev/null || fail "construct.load k5"
until_ok 'test "$(pane_count)" = 4' 80 || fail "the four probe panes do not come: $(call construct.list)"
for name in a b c d; do
    until_ok "test -e $work/$name.log" || fail "no log of pane $name"
done
sleep 2
[ -n "${SHOT:-}" ] && ctl capture "$SHOT" >/dev/null
view() { ctl query owner:construct | python3 -I -c 'import json, sys; print(sorted(str(n.get("rects")) for n in json.load(sys.stdin) if n["id"].startswith("construct-pane-") and not n["id"].endswith("-mirror")))'; }
view=$(view)

click_at "$(screen a 100 80)"
check "a click on the tilted pane a reaches its window at (100, 80)" \
    'until_ok "pressed_at a 100 80" 8'
sleep 0.6
check "the click does not move the view" 'test "$(view)" = "$view"'

inject '[{"text": {"text": "a"}}]'
check "typing goes to the clicked pane a" 'until_ok "test \$(keys a) -ge 1" 8 && test "$(keys b)" = 0'
click_at "$(screen b 380 300)"
check "a click on the tilted pane b reaches its window at (380, 300)" \
    'until_ok "pressed_at b 380 300" 8'
a_keys=$(keys a)
inject '[{"text": {"text": "b"}}]'
check "typing goes to the clicked pane b, not to a" \
    'until_ok "test \$(keys b) -ge 1" 8 && test "$(keys a)" = "$a_keys"'

point=$(screen c 300 150)
# shellcheck disable=SC2086 # X and Y as two arguments
behind d ${point/, / } || fail "the test layout: pane d is not behind the point $point of c"
click_at "$point"
check "where c and d overlap, the nearer pane c gets the click" \
    'until_ok "pressed_at c 300 150" 8 && test "$(presses d)" = 0'

a_presses=$(presses a)
a_name=$(name_of "-700, 2000")
before=$(pane_at "$a_name")
from=$(screen a 240 180)
inject "[{\"pointer_move\": {\"position\": [$from]}},
  {\"key\": {\"key\": \"Super_L\", \"state\": \"pressed\"}},
  {\"button\": {\"state\": \"pressed\", \"hold\": true}},
  {\"pointer_move\": {\"position\": [${from%%,*}, 300]}},
  {\"pointer_move\": {\"position\": [${from%%,*}, 250]}},
  {\"button\": {\"state\": \"released\"}},
  {\"key\": {\"key\": \"Super_L\", \"state\": \"released\"}}]"
sleep 1
check "a drag with the modifier over pane a reaches no window" 'test "$(presses a)" = "$a_presses"'
check "the drag with the modifier moves pane a" 'test "$(pane_at "$a_name")" != "$before"'
click_at "640, 60"
sleep 0.5
view=$(camera)
all_keys=$(cat "$work"/?.log | grep -c '^key')
inject '[{"shortcut": {"keys": "w"}}]'
sleep 1
check "after a click on the empty space, w walks" 'test "$(camera)" != "$view"'
check "the walk key reaches no window" 'test "$(cat "$work"/?.log | grep -c "^key")" = "$all_keys"'

[ "$failed" = 0 ] || fail "direct input on the 3D base"
ok "direct input on the 3D base"
