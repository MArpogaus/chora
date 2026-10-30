#!/bin/bash
# W2 xp: 5 windows, start menu, 20 Alt+Tab, minimize/restore/activate,
# XP Restart x5, five outside restarts; windows must keep their place.
WF=w2; . "$(dirname "$0")/lib.sh"
xp_settings
# keep.sh starts xp again after each restart
session_start "1280x800" tests/battle/keep.sh "/tmp/bt-$WF-xp.pid" "$bin/chora-xp" --theme flavors/xp/theme --config "/tmp/bt-$WF-xp.yaml" || exit 1

for i in $(seq 5); do open $((i*40)) $((200-i*30)) 90 --server-deco; done
wait_count "app_id:chora-probe kind:surface" 5 \
  || finding "W2: $(count_windows) windows instead of 5" wrong "run 5 probe windows on xp"

# the start menu of T11, in place of the clicks on the start button
menu() {  # menu <True|False>  -> wait for that visible state of xp-menu
  for _ in $(seq 50); do
    got=$(ctl query "id:xp-menu" | python3 -c 'import json,sys; x=json.load(sys.stdin); print(x[0].get("visible") if x else "gone")')
    [ "$got" = "$1" ] && return 0
    sleep 0.2
  done
  return 1
}
for i in $(seq 3); do
  ctl call xp start_menu '{"open": true}' > /dev/null 2>&1 \
    || finding "W2: xp start_menu open refused" wrong "call xp start_menu true"
  menu True || finding "W2: xp start_menu(true) did not open the menu" wrong "call xp start_menu true"
  ctl call xp start_menu '{"open": false}' > /dev/null 2>&1 \
    || finding "W2: xp start_menu close refused" wrong "call xp start_menu false"
  menu False || finding "W2: xp start_menu(false) did not close the menu" wrong "call xp start_menu false"
done
inject '[{"shortcut": {"keys": "Ctrl+Escape"}}]' >/dev/null
menu True || finding "W2: Ctrl+Escape did not open the menu" wrong "Ctrl+Escape"
inject '[{"key": {"key": "Escape", "state": "pressed"}}, {"key": {"key": "Escape", "state": "released"}}]' >/dev/null
menu False || finding "W2: Escape did not close the menu" wrong "open menu, Escape"
health "start menu open/close"

for i in $(seq 20); do inject '[{"shortcut": {"keys": "Alt+Tab"}}]' >/dev/null; done
health "20 Alt+Tab"

# minimize, restore and activate with the methods of T11, in place of the
# clicks on the title bar and the taskbar
s=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(json.load(sys.stdin)[0]["id"])')
v() {  # v <surface> <True|False>  -> wait for that visible state of its group
  for _ in $(seq 50); do
    got=$(ctl query "id:xp-win-$1" | python3 -c 'import json,sys; x=json.load(sys.stdin); print(x[0].get("visible") if x else "gone")')
    [ "$got" = "$2" ] && return 0
    sleep 0.2
  done
  return 1
}
v "$s" True || finding "W2: window group xp-win-$s not visible before minimize" wrong "open window $s"
ctl call xp minimize "{\"surface\": \"$s\"}" > /dev/null 2>&1 \
  || finding "W2: xp minimize $s refused" wrong "call xp minimize $s"
v "$s" False || finding "W2: xp minimize did not minimize $s" wrong "call xp minimize $s"
health "after minimize"
ctl call xp restore "{\"surface\": \"$s\"}" > /dev/null 2>&1 \
  || finding "W2: xp restore $s refused" wrong "call xp restore $s"
v "$s" True || finding "W2: xp restore did not restore $s" wrong "call xp restore $s"
health "after restore"
ctl call xp activate "{\"surface\": \"$s\"}" > /dev/null 2>&1 \
  || finding "W2: xp activate $s refused" wrong "call xp activate $s"
ok=0
for _ in $(seq 50); do
  got=$(ctl query "id:$s" | python3 -c 'import json,sys; x=json.load(sys.stdin); print(bool(x and x[0].get("focused")))')
  [ "$got" = True ] && { ok=1; break; }
  sleep 0.2
done
[ $ok = 1 ] || finding "W2: xp activate did not focus $s" wrong "call xp activate $s"

# remember places from the rects of T2
places() { ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys
for n in sorted(json.load(sys.stdin), key=lambda n: n["id"]):
    r = n["rects"].get("HEADLESS-1")
    print(n["id"], f"{int(r[0])},{int(r[1])}" if r else "-")'; }
# the xp lamp flies the mesh of a restored window for 400 ms. While it
# flies, the clipped query of 77550e5 has no rect or one on the way, so
# take the baseline only when two reads 0.5 s apart agree on every place
before=$(places)
for _ in $(seq 30); do
    sleep 0.5
    now=$(places)
    [ "$now" = "$before" ] && ! grep -q ' -$' <<<"$now" && break
    before=$now
done

# the Restart method of T11 five times, then five restarts from the outside
for i in $(seq 5); do
  # xp runs itself again in the same process; Ok comes back before the
  # exec, so only a refused or unknown method fails here
  ctl call xp restart > /dev/null 2>&1 || finding "W2: xp restart refused in round $i" wrong "call xp restart"
  ready xp turn_to '{"desktop": 0}' || finding "W2: xp did not answer after Restart $i" wrong "call xp restart"
  health "XP Restart $i"
done

# five restarts from the outside, as `choractl restart xp` on a login
for i in $(seq 5); do
  restart_plugin xp || finding "W2: the restart of xp failed" ugly "restart xp"
  ready xp turn_to '{"desktop": 0}' || finding "W2: xp did not answer after restart $i" wrong "restart xp"
  health "restart xp $i"
done

after=$(places)
if [ "$before" != "$after" ]; then
  { echo "before: $before"; echo "after:  $after"; } >> "$RUN"
  finding "W2: windows lost their place across restarts" wrong "compare places before/after 10 restarts"
fi
ctl capture "$repo/tests/battle/w2-final.png" --output HEADLESS-1 >> "$RUN" 2>&1
session_stop
