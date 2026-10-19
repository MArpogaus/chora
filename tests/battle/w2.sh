#!/bin/bash
# W2 xp: 5 windows, start menu, 20 Alt+Tab, minimize/restore, XP Restart x5,
# choractl restart xp x5; windows must keep their place.
WF=w2; . "$(dirname "$0")/lib.sh"
session_start "1280x800" "$bin/chora-xp" --theme flavors/xp/theme --no-sounds || exit 1

for i in $(seq 5); do open $((i*40)) $((200-i*30)) 90 --server-deco; done
[ "$(count_windows)" = 5 ] || finding "W2: $(count_windows) windows instead of 5" wrong "spawn 5 probe windows on xp"

# start menu: open and close with the button, Ctrl+Escape checked too
menu() { ctl query "id:xp-menu" | python3 -c 'import json,sys; x=json.load(sys.stdin); print(x[0].get("visible") if x else "gone")'; }
for i in $(seq 3); do
  if eval $(python3 tests/battle/pick.py "id:xp-start-HEADLESS-1" center) && [ -n "${X:-}" ]; then
    inject "[{\"pointer_move\": {\"position\": [$X, $Y]}}, {\"click\": {}}]" >/dev/null; sleep 0.8
    [ "$(menu)" = True ] || finding "W2: start button click did not open the menu" wrong "click xp-start"
    inject "[{\"pointer_move\": {\"position\": [$X, $Y]}}, {\"click\": {}}]" >/dev/null; sleep 0.8
    [ "$(menu)" = False ] || finding "W2: second start click did not close the menu" wrong "click xp-start again"
  else
    finding "W2: no xp-start-HEADLESS-1 node" wrong "query id:xp-start-HEADLESS-1"
  fi
done
inject '[{"shortcut": {"keys": "Ctrl+Escape"}}]' >/dev/null; sleep 0.8
[ "$(menu)" = True ] || finding "W2: Ctrl+Escape did not open the menu" wrong "Ctrl+Escape"
inject '[{"key": {"key": "Escape", "state": "pressed"}}, {"key": {"key": "Escape", "state": "released"}}]' >/dev/null; sleep 0.8
[ "$(menu)" = False ] || finding "W2: Escape did not close the menu" wrong "open menu, Escape"
health "start menu open/close"

for i in $(seq 20); do inject '[{"shortcut": {"keys": "Alt+Tab"}}]' >/dev/null; done
sleep 1; health "20 Alt+Tab"

# minimize with the title bar button, restore with the taskbar button
s=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(json.load(sys.stdin)[0]["id"])')
v() { ctl query "id:xp-win-$s" | python3 -c 'import json,sys; x=json.load(sys.stdin); print(x[0].get("visible") if x else "gone")'; }
[ "$(v)" = True ] || finding "W2: window group xp-win-$s not visible before minimize" wrong "open window $s"
if eval $(python3 tests/battle/pick.py "id:xp-min-$s" center) && [ -n "${X:-}" ]; then
  inject "[{\"pointer_move\": {\"position\": [$X, $Y]}}, {\"click\": {}}]" >/dev/null
  sleep 1.5
  [ "$(v)" = False ] || finding "W2: xp-min-$s click did not minimize" wrong "click min button at $X,$Y"
  health "after minimize"
else
  finding "W2: no xp-min-$s button (server deco missing?)" wrong "open with --server-deco"
fi
if eval $(python3 tests/battle/pick.py "id:xp-task-$s-HEADLESS-1" center) && [ -n "${X:-}" ]; then
  # the first click on the button of the active window does nothing (XP does
  # it the same way), so click until the state flips, at most three times
  ok=0
  for _ in 1 2 3; do
    inject "[{\"pointer_move\": {\"position\": [$X, $Y]}}, {\"click\": {}}]" >/dev/null
    sleep 1.5; [ "$(v)" = True ] && { ok=1; break; }
  done
  [ $ok = 1 ] || finding "W2: taskbar clicks did not restore $s" wrong "up to 3 clicks on xp-task-$s"
  health "after restore"
else
  finding "W2: taskbar button of $s not found" wrong "query id:xp-task-$s-HEADLESS-1"
fi

# remember places
before=$(python3 tests/battle/positions.py)

# XP Restart from the start menu five times; fall back to choractl restart
for i in $(seq 5); do
  inject '[{"shortcut": {"keys": "Ctrl+Escape"}}]' >/dev/null; sleep 0.5
  # the Restart icon of the shutdown dialog: click it when present
  if python3 tests/battle/pick.py "kind:image meta:choice=Restart" center > /tmp/rs 2>/dev/null; then
    eval $(cat /tmp/rs); inject "[{\"pointer_move\": {\"position\": [$X, $Y]}}, {\"click\": {}}]" >/dev/null
    sleep 6
  else
    step "no meta:choice=Restart node (friction); using choractl restart xp"
    ctl restart xp >/dev/null; sleep 4
  fi
  health "XP Restart $i"
done

for i in $(seq 5); do ctl restart xp >/dev/null; sleep 3; health "choractl restart xp $i"; done

after=$(python3 tests/battle/positions.py)
if [ "$before" != "$after" ]; then
  { echo "before: $before"; echo "after:  $after"; } >> "$RUN"
  finding "W2: windows lost their place across restarts" wrong "compare pos before/after 10 restarts"
fi
ctl capture "$repo/tests/battle/w2-final.png" --output HEADLESS-1 >> "$RUN" 2>&1
session_stop
