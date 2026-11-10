#!/bin/bash
# Gate of the cube of the construct (D69 step 9, D76): the construct adds
# four virtual outputs, starts one chora-xp on them and shows each as a
# forwarding face of a cube in its 3D context.
#
# 1. cube.open: four virtual outputs come, each with an xp taskbar.
# 2. A probe window opens on the front face, which shows its output at
#    1:1, and a click on the face reaches the window.
# 3. cube.turn turns the next face to the front; the face before looks
#    away and hides.
# 4. When xp ends, the construct removes the outputs and the cube.
#
# Usage: tests/gate-construct-cube.sh   BIN: directory with the binaries
#   SHOT  a PNG path that keeps a capture of the cube during a turn
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
visible() { ctl query "id:$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(n[0]["visible"] if n else "gone")'; }
probe_rect() {
    ctl query kind:surface | python3 -I -c '
import json, sys
for n in json.load(sys.stdin):
    if n["surface"].get("app_id") == "chora-probe" and n.get("owner") == "xp":
        for out, r in n["rects"].items():
            print(out, round(r[0] + r[2] / 2), round(r[1] + r[3] / 2))
            break'
}

session_start 1280x800 -- "$bin/chora-construct"
grant input.inject screen.capture
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"

opened=$(call cube.open '{}') || fail "cube.open: $opened"
until_ok 'test "$(virtuals | wc -w)" = 4' || fail "no four virtual outputs: $(virtuals)"
until_ok 'test "$(bars)" = 4' 80 || fail "xp shows $(bars) taskbars on $(virtuals)"
ok "the cube shows four xp taskbars on $(virtuals)"

inject '[{"pointer_move": {"position": [640, 400]}}]'
"$bin/chora-probe" window 40 120 200 --report >"$work/probe.log" 2>&1 &
pids+=($!)
until_ok 'test -n "$(probe_rect)"' 80 || fail "the probe window came to no xp face"
read -r out x y <<<"$(probe_rect)"
front=$(echo "$opened" | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["outputs"][0])')
[ "$out" = "$front" ] || fail "the probe window went to $out, not to the front face $front"
inject "[{\"pointer_move\": {\"position\": [$x, $y]}}, {\"click\": {}}]"
until_ok "grep -q '^press' $work/probe.log" 20 || fail "a click on the front face did not reach the probe window"
ok "a click on the front face reached the xp window at ($x, $y) of $out"

call cube.turn '{"by": 1}' >/dev/null || fail "cube.turn"
sleep 0.15
[ -n "${SHOT:-}" ] && ctl capture "$SHOT" >/dev/null
until_ok 'test "$(visible construct-cube-face-1)" = True && test "$(visible construct-cube-face-0)" = False' ||
    fail "the cube did not turn: face 0 $(visible construct-cube-face-0), face 1 $(visible construct-cube-face-1)"
ok "the cube turned face 1 to the front, and face 0 hides"

xp_pid=$(python3 -I - "$CHORA_SOCKET" <<'EOF'
import os, sys
for pid in filter(str.isdigit, os.listdir("/proc")):
    try:
        cmd = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
        env = open(f"/proc/{pid}/environ", "rb").read().split(b"\0")
    except OSError:
        continue
    if cmd[0].endswith(b"chora-xp") and b"CHORA_SOCKET=" + sys.argv[1].encode() in env:
        print(pid)
EOF
)
[ -n "$xp_pid" ] || fail "no xp process of this session"
run_pid "$xp_pid"
kill "$xp_pid"
until_ok 'test -z "$(virtuals)"' 40 || fail "the virtual outputs stay after xp ended: $(virtuals)"
until_ok 'test "$(visible construct-cube-face-0)" = gone' || fail "the cube stays after xp ended"
ok "xp ended: the construct removed the outputs and the cube"
