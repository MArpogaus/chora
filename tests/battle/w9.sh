#!/bin/bash
# W9 tiling day: fast open/close, layout cycling under settings storms
# (valid, invalid, merge patches, null), style changes with bad shaders,
# D58 updates from another client during relayouts, a client that never
# acks its configures, hotplug with windows on the removed output, and
# plugin restarts mid-layout.
WF=w9; STOP_ON_DEATH=1; . "$(dirname "$0")/lib.sh"
config_home
session_start "1280x800,1280x800+1280+0" tests/battle/keep.sh "/tmp/bt-$WF-tiling.pid" "$bin/chora-tiling" || exit 1
ready tiling get_layout '{}' || finding "W9: tiling silent at start" wrong "start tiling"
rss_note start
c0=$(rss chora); t0=$(rss chora-tiling)
set_ok() { ctl call tiling settings.set "$1" >/dev/null 2>&1; }

# 1. a fast open/close churn: 12 windows, kill every other one, 6 more
pids=()
for i in $(seq 12); do pids+=("$(open $((i*20)) 90 $((255-i*20)))"); done
wait_count "app_id:chora-probe kind:surface" 12 || finding "W9: $(count_windows) of 12 windows" wrong "12 runs"
for i in 0 2 4 6 8 10; do kill "${pids[$i]}" 2>/dev/null; done
for i in $(seq 6); do open 200 $((i*30)) 30 >/dev/null; done
wait_count "app_id:chora-probe kind:surface" 12 || finding "W9: $(count_windows) windows after churn, expected 12" wrong "kill 6, open 6"
health "open/close churn"; rss_note "open/close churn"

# 2. layout cycling under a settings storm
layouts=(tile tile_left tile_bottom tile_top fair fair_horizontal spiral dwindle max fullscreen magnifier floating)
bad=('{"gap": -5}' '{"gap": "x"}' '{"tags": 0}' '{"tags": 99}' '{"layouts": []}'
  '{"layouts": ["nope"]}' '{"master_width": 7}' '{"modifier": "Hyper"}' '{"bindings": {"MOD+nokey": "close"}}'
  '{"style": {"opacity": -1}}' '{"focus": null, "nope": 1}' '[1,2]' '"x"' '{"bindings": {"MOD+Return": {"run": ""}}}')
for i in $(seq 36); do
  ctl call tiling set_layout "{\"layout\": \"${layouts[$((i % 12))]}\"}" >/dev/null 2>&1 \
    || finding "W9: set_layout refused under the settings storm" wrong "set_layout + settings.set"
  b=${bad[$((i % ${#bad[@]}))]}
  set_ok "$b" && finding "W9: settings.set took $b" wrong "choractl call tiling settings.set '$b'"
  set_ok "{\"gap\": $((i % 9))}" || finding "W9: settings.set gap refused" wrong "settings.set gap"
  [ $((i % 6)) = 0 ] && { set_ok '{"gap": null, "bindings": {"MOD+F12": null}}' || finding "W9: settings.set null patch refused" wrong "settings.set null"; }
done
health "36 layouts with settings storm"; rss_note "36 layouts with settings storm"
# the file must parse, and reload must answer the same settings
before=$(ctl call tiling settings.get)
after=$(ctl call tiling settings.reload 2>&1) || finding "W9: settings.reload refused the file that settings.set wrote: ${after:0:120}" wrong "storm, then settings.reload"
[ "$before" = "$after" ] || step "reload differs from get (key order?)"
# a broken file by hand: reload fails and keeps the settings
f=$XDG_CONFIG_HOME/chora/chora-tiling.yaml
cp "$f" "$f.good"
printf 'gap: [\n\x00\xff' > "$f"
ctl call tiling settings.reload >/dev/null 2>&1 && finding "W9: settings.reload took a broken file" wrong "garbage file, settings.reload"
ctl call tiling get_layout '{}' >/dev/null 2>&1 || finding "W9: tiling dead after a broken reload" crash "garbage file, settings.reload"
head -c 3000000 /dev/urandom | base64 > "$f"
timeout 15 "$bin/choractl" --name agent call tiling settings.reload >/dev/null 2>&1
grep -q "the request is longer than" "$CHORA_LOG" \
  && finding "W9: a 4 MB settings file + settings.reload disconnects tiling (error echoes the file)" crash "base64 of 3 MB urandom as chora-tiling.yaml, settings.reload"
cp "$f.good" "$f"
ready tiling get_layout '{}' || finding "W9: tiling did not come back after the 4 MB reload" crash "4 MB file, settings.reload, good file back"
ctl call tiling settings.reload >/dev/null 2>&1 || finding "W9: reload of the good file failed" wrong "settings.reload"
health "reload storm"; rss_note "reload storm"

# 3. style changes mid-layout, with shaders the core cannot use
printf 'void main() { this is not glsl }\n' > "/tmp/bt-$WF-broken.frag"
for s in "/tmp/bt-$WF-broken.frag" /dev/null /proc/self/mem; do
  if set_ok "{\"style\": {\"opacity\": 0.8, \"effects\": [{\"shader\": \"$s\"}]}}"; then
    step "settings.set took the shader $s"
    p=$(open 255 255 0)
    ctl call tiling set_layout '{"layout": "fair"}' >/dev/null 2>&1
    if ! wait_sel "app_id:chora-probe kind:surface owner:tiling" 5 || [ "$(count_windows)" -lt 13 ]; then
      finding "W9: a new window did not map with the style shader $s" wrong "settings.set style effects $s, open window"
    fi
    sleep 1
    n=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(sum(1 for n in json.load(sys.stdin) if not n["rects"]))')
    [ "$n" -gt 0 ] && step "with shader $s: $n windows without rects (on other tag?)"
    kill "$p"
  fi
  health "style shader $s"
done
set_ok '{"style": {"opacity": 1, "effects": []}}' || finding "W9: settings.set back to a plain style refused" wrong "settings.set style"

# 4. D58 updates from another client while tiling relayouts
ids=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(" ".join(n["id"] for n in json.load(sys.stdin)))')
for l in tile fair spiral max tile; do
  ctl call tiling set_layout "{\"layout\": \"$l\"}" >/dev/null 2>&1 &
  for id in $ids; do
    err=$(echo "[{\"update\": {\"id\": \"$id\", \"props\": {\"opacity\": 0.6}}}]" | timeout 10 "$bin/choractl" --name agent apply 2>&1) \
      || finding "W9: D58 opacity update refused during set_layout: ${err:0:100}" wrong "set_layout & choractl apply update opacity of $id"
  done
  wait $!
done
# an update that mixes owner-only props must fail
first=${ids%% *}
echo "[{\"update\": {\"id\": \"$first\", \"props\": {\"opacity\": 0.5, \"size\": [10, 10]}}}]" | timeout 10 "$bin/choractl" --name agent apply >/dev/null 2>&1 \
  && finding "W9: a D58 update with size went through" wrong "update opacity+size of a tiling window"
# the meta key window is the state of tiling
echo "[{\"update\": {\"id\": \"$first\", \"props\": {\"meta\": {\"window\": \"[{\\\"tags\\\": 0}]\"}}}}]" | timeout 10 "$bin/choractl" --name agent apply >/dev/null 2>&1
ctl call tiling set_layout '{"layout": "tile"}' >/dev/null 2>&1
sleep 1
held=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(sum(1 for n in json.load(sys.stdin) if abs((n.get("opacity") or 1)-0.6)<0.01))' 2>/dev/null)
step "D58: $held windows keep opacity 0.6 after relayouts"
[ "${held:-0}" -ge 10 ] || finding "W9: D58 opacity held on only ${held:-0} windows after relayouts" wrong "update opacity 0.6, cycle layouts"
health "D58 during relayouts"; rss_note "D58 during relayouts"

# 5. a client that never acks its configures: stop it, then relayout
stuck=$(open 0 255 255)
kill -STOP "$stuck"
for l in tile max fair tile_left; do
  t=$(date +%s%N)
  ctl call tiling set_layout "{\"layout\": \"$l\"}" >/dev/null 2>&1 || finding "W9: set_layout $l refused with a stopped client" wrong "SIGSTOP a window, set_layout"
  ms=$(( ($(date +%s%N) - t) / 1000000 ))
  [ $ms -gt 3000 ] && finding "W9: set_layout took $ms ms with a stopped client" slow "SIGSTOP a window, set_layout"
done
alive_frames stopped-client ctl call tiling set_layout '{"layout": "fair"}'
health "relayout with a stopped client"; rss_note "relayout with a stopped client"
kill -CONT "$stuck"

# 6. hotplug: windows on a third output, then remove it
name=$(admin add-output "1024x768+2560+0" | python3 -c 'import json,sys; print(json.load(sys.stdin)["name"])' 2>/dev/null)
if [ -n "$name" ]; then
  ctl wait "id:screen:$name" --timeout 5 >/dev/null
  inject "[{\"pointer_move\": {\"position\": [500, 400], \"output\": \"$name\"}}]" >/dev/null
  for i in $(seq 3); do open 120 120 $((i*60)) >/dev/null; done
  sleep 1
  on=$(ctl query "app_id:chora-probe kind:surface" | python3 -c "import json,sys; print(sum(1 for n in json.load(sys.stdin) if '$name' in n['rects']))")
  step "windows on $name: $on"
  before=$(count_windows)
  admin remove-output "$name" >/dev/null 2>&1 || finding "W9: remove-output $name failed" wrong "remove-output"
  sleep 2
  lost=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(sum(1 for n in json.load(sys.stdin) if not n["rects"]))')
  step "after removing $name: $(count_windows) of $before windows, $lost without rects"
  [ "$(count_windows)" = "$before" ] || finding "W9: windows lost when $name went" wrong "3 windows on a third output, remove-output"
  ctl call tiling view_tag '{"tag": 1}' >/dev/null 2>&1
  # a tile layout shows all tag-1 windows of an output; count again
  lost=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(sum(1 for n in json.load(sys.stdin) if not n["rects"]))')
  [ "$lost" -gt 4 ] && finding "W9: $lost windows show nowhere after the removal of $name" wrong "windows on a third output, remove-output, view_tag 1"
else
  finding "W9: add-output failed" wrong "add-output 1024x768+2560+0"
fi
health "hotplug with windows"; rss_note "hotplug with windows"

# 7. plugin restarts mid-layout, five times
for i in $(seq 5); do
  ctl call tiling set_layout "{\"layout\": \"${layouts[$i]}\"}" >/dev/null 2>&1 &
  restart_plugin tiling || finding "W9: restart $i of tiling failed" ugly "restart tiling"
  wait $!
  ready tiling get_layout '{}' || finding "W9: tiling silent after restart $i" wrong "restart tiling mid-layout"
  health "restart $i mid-layout"
done
ctl call tiling view_tag '{"tag": 1}' >/dev/null 2>&1
sleep 1
unowned=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(sum(1 for n in json.load(sys.stdin) if n.get("owner")!="tiling"))')
[ "$unowned" -gt 0 ] && finding "W9: $unowned windows not owned by tiling after restarts" wrong "5 restarts mid-layout"
held=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(sum(1 for n in json.load(sys.stdin) if abs((n.get("opacity") or 1)-0.6)<0.01))' 2>/dev/null)
step "after restarts: $held windows keep the D58 opacity"
alive_frames after-restarts ctl call tiling set_layout '{"layout": "spiral"}'

rss_note end
rss_growth "$c0" "$(rss chora)" chora
rss_growth "$t0" "$(rss chora-tiling)" chora-tiling
health "end of W9"; rss_note "end of W9"
log_scan
session_stop
