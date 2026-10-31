#!/bin/bash
# W11 mc day: scene switches during write animations, panel.set floods
# with huge, unicode and empty texts, overlapping hud.say, app.open of
# many apps (some exit at once, some never map), scene.save with hostile
# names and cells, settings changes while animating, plugin restarts with
# app scenes kept, a small output and hotplug.
WF=w11; STOP_ON_DEATH=1; . "$(dirname "$0")/lib.sh"
config_home
scenes=$XDG_CONFIG_HOME/chora/mc/scenes
mkdir -p "$scenes"
session_start "640x480" tests/battle/keep.sh "/tmp/bt-$WF-mc.pid" "$bin/chora-mc" || exit 1
ready mc scene.list '{}' || finding "W11: mc silent at start" wrong "start mc"
rss_note start
c0=$(rss chora); m0=$(rss chora-mc)
mc() { ctl call mc "$@" >/dev/null 2>&1; }
gone() { grep -q "the request is longer than\|I come back" "$CHORA_LOG"; }

# a scene of four panels
mc scene.save '{"name": "day", "grid": [2, 2], "panels": [
  {"id": "a", "cell": [0, 0, 1, 1], "title": "A", "lines": ["one"]},
  {"id": "b", "cell": [1, 0, 1, 1], "title": "B", "lines": ["two"]},
  {"id": "c", "cell": [0, 1, 2, 1], "title": "C", "lines": ["09:00\tthree"]}]}' \
  || finding "W11: scene.save of a plain scene refused" wrong "scene.save day"

# 1. scene switches during the write animation
for i in $(seq 20); do
  for s in day agenda terminal; do mc scene.show "{\"name\": \"$s\"}"; done
done
mc scene.show '{"name": "day"}'
cur=$(ctl call mc scene.list '{}' | python3 -c 'import json,sys; print(json.load(sys.stdin)["current"])')
[ "$cur" = day ] || finding "W11: scene.list says $cur after 60 switches, expected day" wrong "60 scene.show"
sleep 2
vis=$(ctl query "kind:group owner:mc" | python3 -c 'import json,sys
print(" ".join(n["id"] for n in json.load(sys.stdin) if n["id"].startswith("mc-s-") and n["id"].count("-")==2 and n.get("visible")))')
[ "$vis" = "mc-s-day" ] || finding "W11: visible scene groups after 60 switches: '$vis'" wrong "60 scene.show, then day"
health "60 scene switches"

# 2. panel.set floods: 100 changes, huge, unicode, empty
for i in $(seq 100); do mc panel.set "{\"scene\": \"day\", \"panel\": \"a\", \"lines\": [\"line $i\"]}" & [ $((i % 10)) = 0 ] && wait_bg; done
wait_bg
k4=$(head -c 4096 /dev/zero | tr '\0' 'w')
k5=$(head -c 4097 /dev/zero | tr '\0' 'w')
mc panel.set "{\"scene\": \"day\", \"panel\": \"a\", \"lines\": [\"$k4\"]}" || finding "W11: panel.set of exactly 4 KiB refused" wrong "panel.set 4096 bytes"
mc panel.set "{\"scene\": \"day\", \"panel\": \"a\", \"lines\": [\"$k5\"]}" && finding "W11: panel.set of 4097 bytes taken" wrong "panel.set 4097 bytes"
uni='"مرحبا بالعالم", "é́́́ zalgo", "🚀🧪👩‍👩‍👧‍👦", "​‍﻿", "\t\t", "\u0000nul"'
mc panel.set "{\"scene\": \"day\", \"panel\": \"b\", \"title\": \"Ü\u0000ber\", \"lines\": [$uni]}" ; step "panel.set unicode: rc=$?"
mc panel.set '{"scene": "day", "panel": "c", "lines": []}' || finding "W11: panel.set with no lines refused" wrong "panel.set lines []"
mc panel.set '{"scene": "day", "panel": "c", "lines": [""], "title": ""}' || finding "W11: panel.set with an empty line refused" wrong "panel.set lines [\"\"]"
# 1000 lines of one char each: fits 4 KiB, but is a tall text
lines=$(python3 -c 'import json; print(json.dumps(["x"]*1500))')
mc panel.set "{\"scene\": \"day\", \"panel\": \"a\", \"lines\": $lines}" ; step "panel.set 1500 lines: rc=$?"
# 1500 margin notes
lines=$(python3 -c 'import json; print(json.dumps(["9\tx"]*900))')
mc panel.set "{\"scene\": \"day\", \"panel\": \"a\", \"lines\": $lines}" ; step "panel.set 900 notes: rc=$?"
gone && finding "W11: mc lost its connection during panel.set floods" crash "panel.set floods"
ready mc scene.list '{}' || finding "W11: mc silent after panel.set floods" crash "panel.set floods"
health "panel.set floods"

# 3. overlapping hud.say: long, many words, odd durations
mc hud.say '{"text": "one two three four five six seven", "duration_ms": 3000, "state": "listening"}'
mc hud.say '{"text": "", "duration_ms": 0}'
mc hud.say '{"text": "x", "duration_ms": -5}'
mc hud.say '{"text": "huge duration", "duration_ms": 1e308}'
long=$(python3 -c 'print(" ".join(["wordy%d" % i for i in range(1200)]))')
mc hud.say "{\"text\": \"$long\", \"duration_ms\": 200}" ; step "hud.say 1200 words in 200 ms: rc=$?"
sleep 4
gone && finding "W11: a long hud.say ends the connection of mc" crash "hud.say 1200 words (about 10 kB), duration 200 ms"
mc hud.set "{\"state\": \"working\", \"text\": \"$k5\"}" ; step "hud.set 4097 bytes: rc=$?"
mc hud.say '{"text": "a b c", "duration_ms": 300}'
mc hud.set '{"state": "", "text": ""}'
ready mc scene.list '{}' || finding "W11: mc silent after the hud.say tests" crash "hud.say overlap"
health "hud.say overlap"

# 4. app.open: apps that exit at once, never map, and real windows
mc app.open '{"command": "true"}' || finding "W11: app.open true refused" wrong "app.open true"
mc app.open '{"command": "sleep 30"}' || finding "W11: app.open sleep refused" wrong "app.open sleep 30"
mc app.open '{"command": "   "}' && finding "W11: app.open of blanks taken" wrong "app.open '   '"
for i in $(seq 8); do mc app.open "{\"command\": \"$bin/chora-probe window $((i*30)) 30 30\"}"; done
for i in $(seq 30); do mc app.open "{\"command\": \"true $i\"}"; done
sleep 3
n=$(ctl call mc scene.list '{}' | python3 -c 'import json,sys; print(len(json.load(sys.stdin)["scenes"]))')
step "scenes after 41 app.open: $n"
cur=$(ctl call mc scene.list '{}' | python3 -c 'import json,sys; print(json.load(sys.stdin)["current"])')
step "current after app.open bursts: $cur"
[ "$(count_windows)" -ge 7 ] || finding "W11: $(count_windows) of 8 app windows" wrong "8x app.open chora-probe window"
# the scene of an app that exited shows an empty screen: is it still current?
mc app.open '{"command": "true"}'
sleep 2
cur=$(ctl call mc scene.list '{}' | python3 -c 'import json,sys; print(json.load(sys.stdin)["current"])')
[ "$cur" = true ] && finding "W11: the scene of an app that exited stays current (an empty screen)" ugly "app.open true"
health "app.open bursts"

# 5. scene.save with hostile names and cells
for n in '../x' '.hidden' 'a/b' "$(printf 'n%.0s' $(seq 65))" '' 'ü' 'con' 'x y'; do
  if mc scene.save "{\"name\": \"$n\", \"grid\": [1, 1], \"panels\": []}"; then
    case $n in con) ;; *) finding "W11: scene.save took the name '${n:0:20}'" wrong "scene.save name '$n'" ;; esac
  fi
done
ls -a "$XDG_CONFIG_HOME/chora/mc" "$scenes" >> "$RUN" 2>&1
[ -e "$XDG_CONFIG_HOME/chora/mc/x.yaml" ] && finding "W11: scene.save wrote outside its directory" crash "scene.save ../x"
for cell in '[4294967295, 0, 1, 1]' '[0, 0, 4294967295, 1]' '[0, 0, 0, 1]' '[1, 0, 1, 1]'; do
  out=$(ctl call mc scene.save "{\"name\": \"cells\", \"grid\": [1, 1], \"panels\": [{\"id\": \"p\", \"cell\": $cell}]}" 2>&1); rc=$?
  step "scene.save cell $cell: rc=$rc ${out:0:80}"
  [ $rc = 0 ] && finding "W11: scene.save took the cell $cell outside a 1x1 grid" wrong "scene.save cells $cell"
  ready mc scene.list '{}' || { finding "W11: mc died on scene.save with cell $cell" crash "scene.save {grid: [1,1], panels: [{id: p, cell: $cell}]}"; ready mc scene.list '{}'; }
done
mc scene.save '{"name": "big", "grid": [4294967295, 4294967295], "panels": [{"id": "p", "cell": [0, 0, 1, 1], "lines": ["x"]}]}' ; step "scene.save huge grid: rc=$?"
mc scene.show '{"name": "big"}'
ready mc scene.list '{}' || finding "W11: mc died on a huge grid" crash "scene.save big grid [u32::MAX, u32::MAX]"
# 300 panels in one scene
panels=$(python3 -c 'import json; print(json.dumps([{"id": "p%d" % i, "cell": [i % 20, i // 20, 1, 1], "title": "T%d" % i, "lines": ["%d\tline" % i]} for i in range(300)]))')
mc scene.save "{\"name\": \"many\", \"grid\": [20, 15], \"panels\": $panels}" ; step "scene.save 300 panels: rc=$?"
mc scene.show '{"name": "many"}'
sleep 2
ready mc scene.list '{}' || finding "W11: mc silent after a scene of 300 panels" crash "scene.save 300 panels, show"
health "scene.save abuse"

# 6. settings while the write animation runs
for i in $(seq 6); do
  mc scene.show "{\"name\": \"$([ $((i % 2)) = 0 ] && echo day || echo many)\"}"
  t=$([ $((i % 2)) = 0 ] && echo dark || echo light)
  mc settings.set "{\"theme\": \"$t\", \"write\": {\"duration_ms\": $((i * 300))}}" || finding "W11: settings.set theme $t refused" wrong "settings.set theme"
done
for b in '{"theme": "neon"}' '{"scale": {"base": 0}}' '{"scale": {"base": -3}}' '{"scale": {"ratio": 1e308}}' \
  '{"styles": {"body": {"family": "Nope Sans", "size": 1e9}}}' '{"styles": {"body": {"base": "body"}}}' \
  '{"styles": {"loop1": {"base": "loop2"}, "loop2": {"base": "loop1"}}}' '{"page": {"measure": 0}}' \
  '{"page": {"measure": -1, "gap": 1e300}}' '{"default_scene": "../../etc"}' '{"bindings": {"Super+nokey": "next_scene"}}' \
  '{"chapter": {"layout": "large"}}' '{"write": {"enabled": false}}' '{"write": {"enabled": true, "duration_ms": -1}}'; do
  out=$(timeout 20 "$bin/choractl" --name agent call mc settings.set "$b" 2>&1); rc=$?
  step "settings.set $b: rc=$rc ${out:0:100}"
  ready mc scene.list '{}' || { finding "W11: mc died on settings.set $b" crash "settings.set $b"; ready mc scene.list '{}'; }
done
health "settings while animating"

# 7. restarts with app scenes kept
apps_before=$(count_windows)
for i in $(seq 4); do
  mc scene.show '{"name": "day"}' &
  restart_plugin mc || finding "W11: restart of mc failed" ugly "restart mc"
  wait $!
  ready mc scene.list '{}' || finding "W11: mc silent after restart $i" wrong "restart mc"
done
sleep 2
owned=$(ctl query "app_id:chora-probe kind:surface owner:mc" | grep -c '"id"')
[ "$owned" -ge "$apps_before" ] || finding "W11: mc owns $owned of $apps_before app windows after 4 restarts" wrong "4 restarts with app scenes"
health "restarts with apps"

# 8. hotplug: a bigger output comes, the first one goes, it comes back
name=$(admin add-output "1280x800+640+0" | python3 -c 'import json,sys; print(json.load(sys.stdin)["name"])' 2>/dev/null)
admin remove-output HEADLESS-1 >/dev/null 2>&1 || finding "W11: remove-output HEADLESS-1 failed" wrong "remove-output HEADLESS-1"
# mc is busy with its scenes, slow on a debug build: wait up to 30 s for
# it to build on the output that stays
for _ in $(seq 30); do
  on=$(ctl query "id:mc-bg" | python3 -c 'import json,sys; x=json.load(sys.stdin); print(list(x[0]["rects"].keys()) if x else "gone")')
  case $on in *"$name"*) break;; esac
  sleep 1
done
step "mc-bg after the first output went: $on"
case $on in *"$name"*) ;; *) finding "W11: mc shows nothing after its output went (mc-bg rects: $on, $name stays)" wrong "add-output 1280x800, remove-output HEADLESS-1";; esac
mc scene.show '{"name": "agenda"}' || finding "W11: scene.show refused with only $name" wrong "remove the first output, scene.show"
shot only-second "$name" > /dev/null
for i in $(seq 5); do
  n2=$(admin add-output "640x480+0+0" | python3 -c 'import json,sys; print(json.load(sys.stdin)["name"])' 2>/dev/null)
  mc scene.show '{"name": "day"}'
  admin remove-output "$n2" >/dev/null 2>&1
  health "hotplug $i"
done
OUT=$name alive_frames after-hotplug mc scene.show '{"name": "many"}'

rss_note end
rss_growth "$c0" "$(rss chora)" chora
rss_growth "$m0" "$(rss chora-mc)" chora-mc
health "end of W11"
log_scan
session_stop
