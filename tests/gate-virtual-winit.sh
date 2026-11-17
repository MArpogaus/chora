#!/bin/bash
# Gate of virtual outputs on the winit backend (D69, exit criterion of
# step 2): a chora that runs nested in a headless session opens a winit
# window. Inside it tests/pane-host.py adds a virtual output and shows it
# in a forwarding pane, and chora-xp manages that output.
#
# 1. `output.add` with `virtual` answers on the winit backend, the output
#    is no screen of the nested session, and xp draws its taskbar on it.
# 2. A probe window opens under the pane: xp attaches it on the virtual
#    output.
# 3. A click through the pane reaches the window at the mapped point.
# 4. With nothing moving, `render.stats` of the nested session counts no
#    pass and no damaged frame for any output in 2 s.
#
# Usage: tests/gate-virtual-winit.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling chora-xp chora-probe
need python3
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

session_start 1280x800 -- "$bin/chora-tiling"
XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$GATE_ENV")
export XDG_RUNTIME_DIR
outer=$CHORA_SOCKET
ctl wait --timeout 10 owner:tiling >/dev/null || fail "no window manager in the outer session"

inner=$XDG_RUNTIME_DIR/chora/wayland-2.sock
ctl run -- chora -- sleep 900 >"$work/run.json" || fail "choractl run of the nested chora"
run_pid "$(python3 -I -c 'import json, sys; print(json.load(sys.stdin)["pid"])' <"$work/run.json")"
until_ok 'test -S "$inner"' 80 || fail "the nested chora opened no IPC socket"
outer_shows() { CHORA_SOCKET=$outer ctl query kind:surface | grep -q '"chora'; }
until_ok outer_shows 80 || fail "the nested chora shows no window in the outer session"
export CHORA_SOCKET=$inner
export WAYLAND_DISPLAY=$XDG_RUNTIME_DIR/wayland-2
grant input.inject screen.capture
screen=$(ctl outputs | python3 -I -c 'import json, sys; print(json.load(sys.stdin)[0]["name"])')
[ -n "$screen" ] || fail "the nested session has no screen"
size=$(ctl outputs | python3 -I -c 'import json, sys; o = json.load(sys.stdin)[0]; print(o["size"][0], o["size"][1])')
ok "a nested chora runs on the winit backend with the screen $screen of $size"

python3 -I "$repo/tests/pane-host.py" "$screen" 600x400+20+20 >"$work/pane-host.log" 2>&1 &
pids+=($!)
until_ok 'grep -qx ready "$work/pane-host.log"' 40 || fail "no pane host: $(tail -n 3 "$work/pane-host.log")"
face=$(awk 'NF == 5 {print $1; exit}' "$work/pane-host.log")
listed=$(ctl outputs | python3 -I -c 'import json, sys; print(" ".join(o["name"] + ("*" if o.get("virtual") else "") for o in json.load(sys.stdin)))')
[ "$listed" = "$screen $face*" ] || fail "the outputs of the nested session: $listed"
"$bin/chora-xp" --theme "$repo/flavors/xp/theme" --output "$face" >"$work/xp.log" 2>&1 &
pids+=($!)
ctl wait "id:xp-bar-$face" --timeout 20 >/dev/null || fail "xp does not start on $face: $(tail -n 3 "$work/xp.log")"
ok "on winit, output.add gave the virtual output $face, and xp has its taskbar on it"

inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
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
inject '[{"pointer_move": {"position": [320, 220]}}]'
"$bin/chora-probe" window 40 120 200 --report >"$work/probe.log" 2>&1 &
pids+=($!)
until_ok 'probe | grep -q " xp $face "' 80 || fail "the probe window came to no xp on $face: $(probe)"
read -r _ _ _ x y <<<"$(probe)"
ok "xp attached the probe window on $face at ($x, $y)"

inject "[{\"pointer_move\": {\"position\": [$((20 + x + 3)), $((20 + y + 2))]}}, {\"click\": {}}]"
until_ok "grep -q '^press' $work/probe.log" 20 || fail "a click through the pane did not reach the probe window"
grep '^press' "$work/probe.log" | tail -n 1 | awk '{ dx = $2 - 3; dy = $3 - 2; exit !(dx * dx + dy * dy <= 1) }' ||
    fail "the click at (3, 2) of the window came at $(grep '^press' "$work/probe.log" | tail -n 1)"
ok "a click through the pane reached the window at (3, 2)"

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
sleep 3
still || fail "a still virtual output draws on winit: $(drawn)"
ok "on winit a still virtual output and its pane draw nothing: $(drawn)"
ok "gate-virtual-winit: virtual outputs work on the winit backend"
