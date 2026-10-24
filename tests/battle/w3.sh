#!/bin/bash
# W3: two window managers on two outputs, drag windows from XP (output 1)
# across the edge into the tiling output, restart each WM.
WF=w3; . "$(dirname "$0")/lib.sh"
session_start "1280x800,1280x800+1280+0" sh -c "$bin/chora-xp --output HEADLESS-1 --theme flavors/xp/theme --no-sounds & $bin/chora-tiling --output HEADLESS-2 & wait" || exit 1

for i in $(seq 5); do open $((i*30)) 120 $((255-i*30)) --server-deco; done
wait_count "app_id:chora-probe kind:surface" 5 \
  || finding "W3: $(count_windows) windows instead of 5" wrong "spawn 5 windows on two WMs"
health "5 windows on two WMs"
owners() { ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys,collections
c=collections.Counter(n["owner"] for n in json.load(sys.stdin))
print(dict(c))'; }
step "owners: $(owners)"

# drag three windows over the right edge: press the title bar, walk the
# pointer to x>1280 on the second output, release. The grab point comes
# from the rects of T2, not from composing transforms by hand.
for n in 1 2 3; do
  s=$(ctl query "owner:xp app_id:chora-probe kind:surface" | python3 -c "import json,sys; xs=json.load(sys.stdin); print(xs[0]['id'] if xs else '')")
  [ -n "$s" ] || { finding "W3: no xp window left to grab" wrong "query owner:xp"; break; }
  unset X Y
  eval "$(ctl query "id:xp-title-$s" | python3 -c 'import json,sys
x=json.load(sys.stdin)
r=(x[0]["rects"].get("HEADLESS-1") if x else None)
print(f"X={int(r[0]+r[2]/2)} Y={int(r[1]+r[3]/2)}" if r else "")')"
  [ -n "${X:-}" ] || { finding "W3: no rect for xp-title-$s to grab" wrong "query id:xp-title-$s"; break; }
  # one request holds the whole drag: buttons left down go up at the end
  # of their own request, so press and release must live in the same list
  events="[{\"pointer_move\": {\"position\": [$X, $Y], \"output\": \"HEADLESS-1\"}}, {\"button\": {\"state\": \"pressed\"}}"
  for x in 600 900 1200 1400 1700 2000; do
    out=$([ $x -lt 1280 ] && echo HEADLESS-1 || echo HEADLESS-2)
    lx=$([ $x -lt 1280 ] && echo $x || echo $((x-1280)))
    events="$events, {\"pointer_move\": {\"position\": [$lx, 400], \"output\": \"$out\"}}"
  done
  events="$events, {\"button\": {\"state\": \"released\"}}]"
  inject "$events" >/dev/null
  # the drag is fire-and-forget: wait until tiling owns the window
  # before the next query, or the title node can be gone mid-handoff
  wait_sel "owner:tiling id:$s kind:surface" \
    || finding "W3: $s stayed with xp after the drag" wrong "wait owner:tiling id:$s"
  health "drag $n ($s)"
done
step "owners after drags: $(owners)"

ctl restart xp >/dev/null 2>&1 || finding "W3: choractl restart xp failed" ugly "restart xp (sh -c startup)"
ready xp turn_to '{"desktop": 0}' || finding "W3: xp did not answer after restart" wrong "restart xp"
health "restart xp"
ctl restart tiling >/dev/null 2>&1 || finding "W3: choractl restart tiling failed" ugly "restart tiling"
ready tiling get_layout '{}' || finding "W3: tiling did not answer after restart" wrong "restart tiling"
health "restart tiling"
[ "$(count_windows)" -ge 5 ] || finding "W3: only $(count_windows) windows after the restarts" wrong "5 windows, restart both WMs"
ctl capture "$repo/tests/battle/w3-final.png" --output HEADLESS-2 >> "$RUN" 2>&1
session_stop
