#!/bin/bash
# W10 xp day: open/minimize/restore/maximize/close bursts, cube turns
# while windows move, dialogs open during restarts, settings.set while
# the start menu is open, broken values (missing files, huge strings,
# unicode), harmless session commands, restart spam and hotplug.
# Notifications need a session bus; the sandbox has no dbus-daemon, so
# the notify-send flood runs only where `dbus-run-session` exists.
WF=w10; STOP_ON_DEATH=1; . "$(dirname "$0")/lib.sh"
config_home
xp_settings
mark=/tmp/bt-$WF-mark
rm -f "$mark".*
session_start "1280x800" tests/battle/keep.sh "/tmp/bt-$WF-xp.pid" "$bin/chora-xp" --theme flavors/xp/theme --config "/tmp/bt-$WF-xp.yaml" || exit 1
ready xp turn_to '{"desktop": 0}' || finding "W10: xp silent at start" wrong "start xp"
rss_note start
c0=$(rss chora); x0=$(rss chora-xp)
xset() { ctl call xp settings.set "$1" >/dev/null 2>&1; }
# harmless session commands, never systemctl
xset "{\"session\": {\"turn_off\": \"touch $mark.turn_off\", \"restart\": \"touch $mark.restart\", \"stand_by\": \"touch $mark.stand_by\", \"log_off\": \"touch $mark.log_off\"}}" \
  || finding "W10: settings.set of the session commands refused" wrong "settings.set session"
# center <node id> -> "X Y" of its rect on HEADLESS-1
center() { ctl query "id:$1" | python3 -c 'import json,sys
x=json.load(sys.stdin); r=x[0]["rects"].get("HEADLESS-1") if x else None
print(f"{int(r[0]+r[2]/2)} {int(r[1]+r[3]/2)}" if r else "")'; }
click() {  # click <x> <y> [count]
  local ev="[{\"pointer_move\": {\"position\": [$1, $2], \"output\": \"HEADLESS-1\"}}"
  for _ in $(seq "${3:-1}"); do ev="$ev, {\"button\": {\"state\": \"pressed\"}}, {\"button\": {\"state\": \"released\"}}"; done
  inject "$ev]" >/dev/null
}
surfaces() { ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(" ".join(n["id"] for n in json.load(sys.stdin)))'; }
# faces of a cube turn that never ended
faces() { ctl query "kind:group" | grep -c '"id": "xp-face-'; }

# 1. bursts: 8 windows, then minimize/restore/activate without waiting
pids=()
for i in $(seq 8); do pids+=("$(open $((i*30)) 60 $((250-i*30)) --server-deco)"); done
wait_count "app_id:chora-probe kind:surface" 8 || finding "W10: $(count_windows) of 8 windows" wrong "8 runs on xp"
ids=$(surfaces)
for round in 1 2 3; do
  for s in $ids; do ctl call xp minimize "{\"surface\": \"$s\"}" >/dev/null 2>&1 & done
  wait_bg
  for s in $ids; do ctl call xp restore "{\"surface\": \"$s\"}" >/dev/null 2>&1 & done
  wait_bg
  for s in $ids; do ctl call xp activate "{\"surface\": \"$s\"}" >/dev/null 2>&1; done
done
sleep 3
hidden=0
for s in $ids; do
  v=$(ctl query "id:xp-win-$s" | python3 -c 'import json,sys; x=json.load(sys.stdin); print(x[0].get("visible") if x else "gone")')
  [ "$v" = True ] || hidden=$((hidden+1))
done
[ $hidden = 0 ] || finding "W10: $hidden of 8 windows stay hidden after minimize/restore bursts" wrong "3x parallel minimize, restore, activate of 8 windows"
# maximize two windows with their max button, twice (maximize, restore)
for s in $(echo $ids | cut -d' ' -f1-2); do
  for _ in 1 2; do
    xy=$(center "xp-max-$s"); [ -n "$xy" ] && click $xy
    sleep 0.5
  done
done
health "minimize/restore/maximize bursts"

# 2. cube turns while windows move and restore
for i in $(seq 12); do
  ctl call xp turn_to "{\"desktop\": $((i % 4))}" >/dev/null 2>&1 &
  s=$(echo $ids | cut -d' ' -f$((i % 8 + 1)))
  xy=$(center "xp-title-$s")
  if [ -n "$xy" ]; then
    set -- $xy
    inject "[{\"pointer_move\": {\"position\": [$1, $2], \"output\": \"HEADLESS-1\"}}, {\"button\": {\"state\": \"pressed\"}}, {\"pointer_move\": {\"position\": [$(($1+40)), $(($2+30))], \"output\": \"HEADLESS-1\"}}, {\"button\": {\"state\": \"released\"}}]" >/dev/null
  fi
  ctl call xp restore "{\"surface\": \"$s\"}" >/dev/null 2>&1
done
wait_bg
ctl call xp turn_to '{"desktop": 0}' >/dev/null 2>&1
ok=0
for _ in $(seq 50); do [ "$(faces)" = 0 ] && { ok=1; break; }; sleep 0.2; done
[ $ok = 1 ] || finding "W10: cube faces stay after the turns ($(faces) xp-face nodes)" wrong "12 turn_to during drags and restores"
health "cube turns during moves"

# 3. settings.set while the start menu is open, valid and broken values
ctl call xp start_menu '{"open": true}' >/dev/null 2>&1
long=$(head -c 600000 /dev/zero | tr '\0' 'x')
bad=("{\"wallpaper\": {\"picture\": {\"file\": \"/nonexistent/a.png\"}}}"
  "{\"wallpaper\": {\"picture\": {\"file\": \"/etc/passwd\"}}}"
  "{\"wallpaper\": {\"picture\": {\"file\": \"relative.png\"}}}"
  "{\"wallpaper\": {\"color\": [999, -1, 0]}}"
  "{\"wallpaper\": {\"position\": \"nope\"}}"
  "{\"taskbar\": {\"auto_hide\": true, \"on_top\": false}}"
  "{\"taskbar\": {\"auto_hide\": false, \"on_top\": true, \"clock\": false}}"
  "{\"terminal\": \"ターミナル 🚀 \\\"quoted\\\" \$(id)\"}"
  "{\"terminal\": \"   \"}"
  "{\"clock_format\": \"$(printf '%%A %.0s' $(seq 1500))\"}"
  "{\"programs\": [{\"title\": \"$(printf 'Ünï %.0s' $(seq 300))\", \"command\": \"true\", \"icon\": \"nope\"}]}"
  "{\"programs\": [{\"title\": \"\", \"command\": \"true\", \"icon\": \"x\"}]}")
for b in "${bad[@]}"; do
  out=$(timeout 15 "$bin/choractl" --name agent call xp settings.set "$b" 2>&1); rc=$?
  step "settings.set ${b:0:70}: rc=$rc ${out:0:90}"
  ctl call xp turn_to '{"desktop": 0}' >/dev/null 2>&1 || { finding "W10: xp silent after settings.set ${b:0:60}" crash "settings.set ${b:0:60} with the menu open"; ready xp turn_to '{"desktop": 0}'; }
done
# an endless wallpaper file: guarded, because chora-xp reads it whole
# (it took 104 GB within seconds once); kill xp past 1 GB
ctl call xp settings.set '{"wallpaper": {"picture": {"file": "/dev/zero"}}}' >/dev/null 2>&1 &
caller=$!
peak=0
for _ in $(seq 150); do
  r=$(rss chora-xp); [ -n "$r" ] && [ "$r" -gt "$peak" ] && peak=$r
  if [ "${r:-0}" -gt 1000000 ]; then
    for p in $(pgrep -s "$(pgrep -P "$HLPID" -x chora)" -x chora-xp); do kill -9 "$p"; done
    finding "W10: wallpaper /dev/zero makes chora-xp read without end (RSS past 1 GB)" crash "settings.set wallpaper picture file /dev/zero"
    break
  fi
  kill -0 $caller 2>/dev/null || break
  sleep 0.1
done
wait $caller
step "wallpaper /dev/zero: peak xp RSS $peak kB"
ready xp turn_to '{"desktop": 0}' || finding "W10: xp did not come back after /dev/zero" crash "wallpaper /dev/zero"
# keep.sh started xp again from the file, which may hold /dev/zero now
grep -q /dev/zero "/tmp/bt-$WF-xp.yaml" && finding "W10: the settings file keeps the wallpaper /dev/zero" crash "settings.set wallpaper /dev/zero"
xset '{"wallpaper": {"picture": "default"}}'
# a 600 kB terminal twice: settings.get must still answer
# nine strings of 120 kB (one argument holds at most 128 kB): together
# the settings pass the 1 MiB line limit of the core
long=$(head -c 120000 /dev/zero | tr '\0' 'x')
for k in terminal launcher file_manager; do xset "{\"$k\": \"$long\"}"; step "120 kB $k: rc=$?"; done
for k in turn_off restart stand_by log_off; do xset "{\"session\": {\"$k\": \"$long\"}}"; step "120 kB session.$k: rc=$?"; done
xset "{\"programs\": [{\"title\": \"t\", \"command\": \"$long\", \"icon\": \"x\"}]}"; step "120 kB program: rc=$?"
xset "{\"clock_format\": \"$long\"}"; step "120 kB clock_format: rc=$?"
ctl call xp settings.get >/dev/null 2>&1 || finding "W10: settings.get fails once the settings pass 1 MiB" wrong "settings.set of 9 strings of 120 kB"
grep -q "the request is longer than" "$CHORA_LOG" && finding "W10: xp disconnected by its own answer past 1 MiB (settings of 9x120 kB)" crash "settings.set of 9 strings of 120 kB each"
ready xp turn_to '{"desktop": 0}' || finding "W10: xp gone after 1 MiB of settings" crash "settings.set of 9 strings of 120 kB"
xset '{"session": {"turn_off": "true", "restart": "true", "stand_by": "true", "log_off": "true"}}'
xset '{"file_manager": "thunar"}'
xset '{"terminal": "foot", "launcher": "fuzzel", "clock_format": "%-I:%M %p", "programs": [], "wallpaper": {"picture": "default", "position": "fill", "color": [0, 78, 152]}}' \
  || finding "W10: settings.set back to sane values refused" wrong "settings.set defaults"
ctl call xp start_menu '{"open": false}' >/dev/null 2>&1
health "settings with the menu open"

# 4. dialogs open during restarts; the Turn Off dialog runs a harmless command
for i in $(seq 4); do
  ctl call xp start_menu '{"open": true}' >/dev/null 2>&1
  inject '[{"key": {"key": "u", "state": "pressed"}}, {"key": {"key": "u", "state": "released"}}]' >/dev/null
  sleep 0.5
  step "dialogs open: $(ctl query 'id:xp-dialogs' | grep -c '"children": \[\]')"
  if [ $((i % 2)) = 0 ]; then
    ctl call xp restart >/dev/null 2>&1
  else
    restart_plugin xp || finding "W10: restart of xp failed" ugly "restart xp"
  fi
  ready xp turn_to '{"desktop": 0}' || finding "W10: xp silent after a restart with a dialog open ($i)" wrong "start menu, u, restart"
  health "restart $i with a dialog open"
done
# restart spam: 10 restarts in a row without waiting
for i in $(seq 10); do ctl call xp restart >/dev/null 2>&1; done
ready xp turn_to '{"desktop": 0}' || finding "W10: xp silent after 10 restarts in a row" wrong "10x call xp restart"
sleep 2
[ "$(count_windows)" = 8 ] || finding "W10: $(count_windows) windows after restart spam, expected 8" wrong "10x call xp restart"
nodes=$(ctl query "owner:xp" | grep -c '"id"')
step "xp nodes after restarts: $nodes"
# Turn Off dialog: keyboard u, then the dialog's default button with Return
ctl call xp start_menu '{"open": true}' >/dev/null 2>&1
inject '[{"key": {"key": "u", "state": "pressed"}}, {"key": {"key": "u", "state": "released"}}]' >/dev/null
sleep 0.5
inject '[{"key": {"key": "Return", "state": "pressed"}}, {"key": {"key": "Return", "state": "released"}}]' >/dev/null
sleep 2
step "session marks: $(ls "$mark".* 2>/dev/null | tr '\n' ' ')"
health "Turn Off dialog"

# 5. hotplug: a second output comes and goes 10 times, windows stay
for i in $(seq 10); do
  name=$(admin add-output "800x600+1280+0" | python3 -c 'import json,sys; print(json.load(sys.stdin)["name"])' 2>/dev/null)
  [ -n "$name" ] || { finding "W10: add-output failed" wrong "add-output"; break; }
  ctl wait "id:xp-bar-$name" --timeout 5 >/dev/null || finding "W10: no taskbar on $name" wrong "add-output, wait xp-bar-$name"
  [ $((i % 3)) = 0 ] && ctl call xp start_menu '{"open": true}' >/dev/null 2>&1
  admin remove-output "$name" >/dev/null 2>&1
  health "hotplug $i"
done
ctl call xp start_menu '{"open": false}' >/dev/null 2>&1
sleep 1
[ "$(count_windows)" = 8 ] || finding "W10: $(count_windows) windows after hotplug, expected 8" wrong "10x add/remove output"
off=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(sum(1 for n in json.load(sys.stdin) if not n["rects"]))')
[ "$off" -gt 0 ] && finding "W10: $off windows show nowhere after hotplug" wrong "10x add/remove output"
for p in "${pids[@]:0:4}"; do kill "$p"; done
ctl wait "app_id:chora-probe kind:surface" --timeout 3 >/dev/null
alive_frames after-hotplug ctl call xp start_menu '{"open": true}'

rss_note end
rss_growth "$c0" "$(rss chora)" chora
rss_growth "$x0" "$(rss chora-xp)" chora-xp
health "end of W10"
log_scan
session_stop
