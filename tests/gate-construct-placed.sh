#!/bin/bash
# Gate of the saved pose (B1): a pane that was moved is saved as placed.
# After a restart the cube stands back and the pane is still drawn in
# front of it; a pane nobody placed gives way to the cube.
#
# Usage: tests/gate-construct-placed.sh   BIN: directory with the binaries
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
surface_of_pane() {
    ctl query "kind:surface owner:construct" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
print(nodes[0]["id"] if nodes else "")'
}
pane_drawn() {
    ctl query "id:construct-pane-$1" | python3 -I -c '
import json, sys
nodes = json.load(sys.stdin)
print(bool(nodes and nodes[0].get("visible") and nodes[0]["rects"].get("HEADLESS-1")))'
}
start_cube() {
    session_start 1280x800 -- "$bin/chora-construct"
    grant input.inject screen.capture
    ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
}
end_cube() {
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
}
load_and_step_back() {
    call construct.load "{\"name\": \"$1\"}" >/dev/null || fail "construct.load $1"
    until_ok 'test -n "$(surface_of_pane)"' 80 || fail "the program pane of $1 has no window"
    call cube.open '{}' >/dev/null || fail "cube.open"
    sleep 1
    echo '[{"shortcut": {"keys": "Alt+Escape"}}]' | ctl inject >/dev/null || fail "step back"
    sleep 3
}

mkdir -p "$work/chora/constructs"
cat >"$work/chora/constructs/placed.yaml" <<YAML
camera: {x: 0, z: 0, yaw: 0, pitch: 0}
radius: 5000
panes:
- kind: program
  command: $bin/chora-probe window 40 120 200 --server-deco --report
  at: [0, 2000]
  size: [800, 500]
YAML

start_cube
load_and_step_back placed
surface=$(surface_of_pane)
[ "$(pane_drawn "$surface")" != True ] || fail "a pane nobody placed is drawn in front of the stood-back cube"
ok "a pane nobody placed gives way to the cube that stands back"
name=$(call construct.list | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["panes"][0]["name"])')
call pane.move "{\"name\": \"$name\", \"right\": 10}" >/dev/null || fail "pane.move $name"
call construct.save '{"name": "placed-saved"}' >/dev/null || fail "construct.save"
[ -s "$work/chora/constructs/placed-saved.yaml" ] || fail "no saved file"
end_cube

start_cube
load_and_step_back placed-saved
surface=$(surface_of_pane)
until_ok '[ "$(pane_drawn "$surface")" = True ]' 20 ||
    fail "a placed pane saved and loaded is hidden again when the cube stands back: $(ctl query "id:construct-pane-$surface" | tr -d "\n " | cut -c1-600)"
ok "a placed pane stays drawn after a save and a restart"
end_cube
