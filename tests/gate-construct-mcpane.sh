#!/bin/bash
# Gate of the mc plugin pane in the construct flavor.
#
# 1. mc runs, and the construct shows the mc scene agenda as a plugin
#    pane. A list in the left margin of mc (region.put notes left) shows
#    in the pane: the node that mc tags for panes holds the regions, and
#    the capture is kept to look at.
# 2. A fly-in into the pane gives the keyboard to mc (pane.focus): the
#    page of mc is the focused node.
#
# Usage: tests/gate-construct-mcpane.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the captures as construct-mcpane-*.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-construct chora-mc
temp_config
work=$XDG_CONFIG_HOME
shots=${SHOTS:-$work}
mkdir -p "$work/chora/constructs" "$shots"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}

call() { ctl call construct "$@"; }
focused() { ctl query "$1" | python3 -I -c 'import json, sys; n = json.load(sys.stdin); print(n[0].get("focused") if n else "")'; }

cat >"$work/chora/constructs/mcpane.yaml" <<EOF
camera: {x: 0, z: 0, yaw: 0, pitch: 0}
radius: 5000
panes:
- kind: plugin
  plugin: mc
  scene: agenda
  at: [0, 1400]
EOF

session_start 1280x720 -- "$bin/chora-mc"
spid=${pids[-1]}
trap 's=$?; gate_cleanup $s' EXIT
grant screen.capture input.inject
ctl wait --timeout 10 'owner:mc meta:pane=agenda' >/dev/null || fail "mc tags no pane"
"$bin/chora-construct" >"$work/construct.log" 2>&1 &
pids+=("$!")
ctl wait --timeout 10 id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$work/construct.log")"
call construct.load '{"name": "mcpane"}' >/dev/null || fail "construct.load"
ctl wait --timeout 10 id:construct-plugin-agenda >/dev/null || fail "no mc pane"

# 1. The left margin shows in the pane.
ctl call mc region.put '{"content": "notes", "region": "left"}' >/dev/null || fail "region.put left"
ctl call mc content.set '{"content": "notes", "title": "Notes", "lines": ["left margin", "second line"]}' >/dev/null || fail "content.set"
ctl call mc region.toggle '{"region": "left", "shown": true}' >/dev/null || fail "region.toggle left"
sleep 2
ctl capture "$shots/construct-mcpane-left.png" >/dev/null || fail "capture"
ctl query 'id:mc' | python3 -I -c '
import json, sys
n = json.load(sys.stdin)[0]
sys.exit(0 if n["size"] and "mc-regions" in n["children"] else 1)' || fail "the tagged node of mc holds no regions"
ok "the node of mc that the pane shows holds the regions; see construct-mcpane-left.png"

# 2. Fly in: mc takes the keyboard for its page.
call pane.fly '{"name": "agenda"}' >/dev/null || fail "pane.fly"
sleep 2
[ "$(focused id:mc-s-agenda-page)" = True ] || fail "mc has no keyboard in its pane; the space has it: $(focused id:construct-look)"
ok "flown into the mc pane, mc has the keyboard"
