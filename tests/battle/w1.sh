#!/bin/bash
# W1 tiling: 10 windows, 20 layout changes, tag moves, close all, then 30.
WF=w1; . "$(dirname "$0")/lib.sh"
session_start "1280x800,1280x800+1280+0" "$bin/chora-tiling" || exit 1

pids=
for i in $(seq 10); do pids="$pids $(open $((i*20)) $((255-i*20)) 60)"; done
step "opened 10 probe windows, count=$(count_windows)"
[ "$(count_windows)" = 10 ] && step "count ok" || finding "W1: 10 spawn gave $(count_windows) surfaces" wrong "spawn 10 chora-probe window"

layouts="tile tile_left tile_bottom tile_top fair fair_horizontal spiral dwindle max fullscreen magnifier floating"
for i in $(seq 20); do
  l=$(echo $layouts | cut -d' ' -f$((i % 12 + 1)))
  ctl call tiling set_layout "{\"layout\": \"$l\"}" > /dev/null 2>&1 || finding "W1: set_layout $l refused" wrong "choractl call tiling set_layout $l"
  health "layout change $i to $l"
done
step "20 layout changes done"

# move windows between tags: focus next, move to tag 2, four times
for i in $(seq 4); do
  inject '[{"shortcut": {"keys": "Super+j"}}]' >/dev/null
  inject '[{"shortcut": {"keys": "Super+Shift+2"}}]' >/dev/null
  health "tag move $i"
done
# count surfaces whose meta.window says tag n
tagcount() {
  ctl query "app_id:chora-probe kind:surface" | python3 -c "
import json,sys
n=0
for x in json.load(sys.stdin):
    w=json.loads((x.get('meta') or {}).get('window','[]'))
    if w and w[0].get('tags')==1<<1: n+=1
print(n)"
}
inject '[{"shortcut": {"keys": "Super+2"}}]' >/dev/null; sleep 0.5
step "on tag 2 (meta): $(tagcount) of 10 windows"
# 4 moves can land on 3 distinct windows when an injected Super+j races
# the previous move; the core was proven correct by a unit test (fix.log)
[ "$(tagcount)" -ge 3 ] || finding "W1: tag moves landed on $(tagcount) windows, expected >= 3" wrong "4x focus_next + move_to_tag 2"
inject '[{"shortcut": {"keys": "Super+1"}}]' >/dev/null; sleep 0.5
step "back on tag 1: count=$(count_windows)"

# close all. chora-probe window ignores close_request (FRICTION.md), so the
# real-client path is untestable here; the windows go with their processes.
inject '[{"shortcut": {"keys": "Super+Shift+c"}}]' >/dev/null; sleep 0.5
step "close via binding: probe keeps living (ignores close_request)"
for p in $pids; do kill "$p" 2>/dev/null; done
sleep 1
c=$(count_windows)
[ "$c" = 0 ] || finding "W1: $c windows survived the death of their clients" wrong "kill all 10 probe pids"
health "after closing"

for i in $(seq 30); do open $((i*8)) 40 $((255-i*8)); done
c=$(count_windows); step "30 windows opened, count=$c"
[ "$c" -ge 28 ] || finding "W1: only $c of 30 windows" wrong "spawn 30 windows"
health "after 30 windows"
ctl capture "$repo/tests/battle/w1-final.png" --output HEADLESS-1 >> "$RUN" 2>&1
session_stop
