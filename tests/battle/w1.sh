#!/bin/bash
# W1 tiling: 10 windows, 20 layout changes, tag moves, close all, then 30.
WF=w1; . "$(dirname "$0")/lib.sh"
session_start "1280x800,1280x800+1280+0" "$bin/chora-tiling" || exit 1

pids=
for i in $(seq 10); do pids="$pids $(open $((i*20)) $((255-i*20)) 60)"; done
wait_count "app_id:chora-probe kind:surface" 10 \
  || finding "W1: 10 spawn gave $(count_windows) surfaces" wrong "spawn 10 chora-probe window"
step "opened 10 probe windows, count=$(count_windows)"

layouts="tile tile_left tile_bottom tile_top fair fair_horizontal spiral dwindle max fullscreen magnifier floating"
for i in $(seq 20); do
  l=$(echo $layouts | cut -d' ' -f$((i % 12 + 1)))
  ctl call tiling set_layout "{\"layout\": \"$l\"}" > /dev/null 2>&1 || finding "W1: set_layout $l refused" wrong "choractl call tiling set_layout $l"
  health "layout change $i to $l"
done
step "20 layout changes done"

# the surface id of the probe window with the keyboard focus, from T1
focused_probe() { ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys
print(next((n["id"] for n in json.load(sys.stdin) if n.get("focused")), ""))'; }

# move windows between tags with the methods of T10: focus next, move to 2
prev=
for i in $(seq 4); do
  ctl call tiling focus_next > /dev/null 2>&1 || finding "W1: focus_next refused" wrong "call tiling focus_next"
  f=
  for _ in $(seq 50); do
    f=$(focused_probe)
    [ -n "$f" ] && [ "$f" != "$prev" ] && break
    sleep 0.2
  done
  { [ -n "$f" ] && [ "$f" != "$prev" ]; } \
    || { finding "W1: focus_next left the focus on ${f:-nothing}" wrong "call tiling focus_next"; break; }
  ctl call tiling move_to_tag "{\"surface\": \"$f\", \"tag\": 2}" > /dev/null 2>&1 \
    || finding "W1: move_to_tag $f refused" wrong "call tiling move_to_tag $f 2"
  prev=$f
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
# count probe windows that show on some output, from the rects of T2
shown_count() { ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys
print(sum(1 for n in json.load(sys.stdin) if n["rects"]))'; }

for _ in $(seq 50); do [ "$(tagcount)" -ge 4 ] && break; sleep 0.2; done
# the method calls are synchronous, so all 4 moves land on 4 windows
[ "$(tagcount)" = 4 ] || finding "W1: tag moves landed on $(tagcount) windows, expected 4" wrong "4x focus_next + move_to_tag 2"

ctl call tiling view_tag '{"tag": 2}' > /dev/null 2>&1 || finding "W1: view_tag 2 refused" wrong "call tiling view_tag 2"
ok=0
for _ in $(seq 50); do [ "$(shown_count)" = 4 ] && { ok=1; break; }; sleep 0.2; done
[ $ok = 1 ] || finding "W1: view_tag 2 shows $(shown_count) windows, expected 4" wrong "view tag 2 after 4 moves"
step "on tag 2: 4 windows with rects (meta: $(tagcount))"

ctl call tiling view_tag '{"tag": 1}' > /dev/null 2>&1 || finding "W1: view_tag 1 refused" wrong "call tiling view_tag 1"
ok=0
for _ in $(seq 50); do [ "$(shown_count)" = 6 ] && { ok=1; break; }; sleep 0.2; done
[ $ok = 1 ] || finding "W1: view_tag 1 shows $(shown_count) windows, expected 6" wrong "view tag 1 again"
step "back on tag 1: count=$(count_windows)"

# close all. chora-probe window ignores close_request without its
# --exit-on-close flag (FRICTION.md), so the real-client path is untestable
# here; the windows go with their processes.
inject '[{"shortcut": {"keys": "Super+Shift+c"}}]' >/dev/null
step "close via binding: probe keeps living (ignores close_request)"
for p in $pids; do kill "$p" 2>/dev/null; done
ctl wait "app_id:chora-probe kind:surface" --gone --timeout 10 >/dev/null \
  || finding "W1: $(count_windows) windows survived the death of their clients" wrong "kill all 10 probe pids"
health "after closing"

for i in $(seq 30); do open $((i*8)) 40 $((255-i*8)); done
wait_count "app_id:chora-probe kind:surface" 28 \
  || finding "W1: only $(count_windows) of 30 windows" wrong "spawn 30 windows"
c=$(count_windows); step "30 windows opened, count=$c"
health "after 30 windows"
ctl capture "$repo/tests/battle/w1-final.png" --output HEADLESS-1 >> "$RUN" 2>&1
session_stop
