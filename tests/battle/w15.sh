#!/bin/bash
# W15 voice HUD: tests/fake-voice.py stands in for the voice agent. In
# the construct flavor, in mc and in xp: the states, a normal and a
# physical `voice.confirm`, an answer by key (Alt+y reaches the grabber)
# and by click on a button (`confirm.answer`), a foreign client that
# calls `indicator.claim` and `confirm.answer` of voice (must be
# refused), and spoofed events of a client named `voice.evil` (must be
# ignored).
WF=w15; STOP_ON_DEATH=1; . "$(dirname "$0")/lib.sh"; . "$(dirname "$0")/lib-wm.sh"
config_home
trap 'exec 3>&- 4>&- 2>/dev/null; session_stop' EXIT
prompt="Delete the build cache of 96 projects?"
detail="rm -rf /home/user/projects/build-cache"
evil=ZZEVILZZ

# voice_start: the fake voice (fd 3) and the spoofing one (fd 4); both
# read events from fifos. Programs started after this inherit the fds, so
# the plugin starts first.
voice_start() {
  rm -f "$work/v.in" "$work/e.in" "$work/claims" "$work/claims.answers" "$work/claims.keys" "$work/eclaims"
  mkfifo "$work/v.in" "$work/e.in"
  FAKE_VOICE_PRIVILEGED=1 FAKE_VOICE_KEYS=1 python3 -I "$repo/tests/fake-voice.py" "$work/claims" < "$work/v.in" 2>> "$RUN" &
  vpid=$!; track $vpid
  exec 3> "$work/v.in"
  FAKE_VOICE_NAME=voice.evil FAKE_VOICE_PRIVILEGED=1 python3 -I "$repo/tests/fake-voice.py" "$work/eclaims" < "$work/e.in" 2>> "$RUN" &
  epid=$!; track $epid
  exec 4> "$work/e.in"
}
voice_stop() {
  exec 3>&- 4>&-
  kill "$vpid" "$epid" 2>/dev/null
  wait "$vpid" "$epid" 2>/dev/null
}
emit() { echo "$1 $2" >&3; }
emit_evil() { echo "$1 $2" >&4; }
claims() { cat "$work/claims" 2>/dev/null || echo 0; }
state() { emit voice.state "{\"state\": \"$1\", \"task\": \"Look up the train to Basel\", \"said\": \"\"}"; }
# ui_has <text>: a node of the plugin shows the text
# (xp draws the balloon as an image: its speech node stands for the text)
ui_has() {
  if [ "$plugin" = xp ]; then
    ctl query id:xp-assistant-speech | py 'import json, sys; n = json.load(sys.stdin); sys.exit(not (n and n[0]["visible"] and n[0]["opacity"] > 0 and n[0]["rects"]))'
  else
    # a node that shows (visible, opaque, with a rect) has the text
    ctl query "owner:$plugin" | py 'import json, sys
t = sys.argv[1].lower()
sys.exit(not any(t in (n.get("text") or "").lower() and n["visible"] and n["opacity"] > 0 and n["rects"] for n in json.load(sys.stdin)))' "$1"
  fi
}
node_centre() { ctl query "id:$1" | py 'import json, sys; n = json.load(sys.stdin); r = list(n[0]["rects"].values())[0]; print(round(r[0] + r[2] / 2), round(r[1] + r[3] / 2))' 2>/dev/null; }
click_node() {
  local c
  c=$(node_centre "$1") || return 1
  [ -n "$c" ] || return 1
  inject "[{\"pointer_move\": {\"position\": [${c% *}, ${c#* }]}}, {\"click\": {}}]" > /dev/null
}
foreign() { # method params -> the answer of a client that has methods.call only
  python3 -I - "$foreign_sock" "$1" "$2" 2>&1 <<'EOF'
import json, socket, sys
path, method, params = sys.argv[1:4]
s = socket.socket(socket.AF_UNIX); s.settimeout(10); s.connect(path); f = s.makefile()
def rpc(m, p, i=[0]):
    i[0] += 1
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": i[0], "method": m, "params": p}).encode() + b"\n")
    while True:
        r = json.loads(f.readline())
        if r.get("id") == i[0]: return r
rpc("hello", {"name": "foreign"})
r = rpc("methods.call", {"client": "voice", "method": method, "params": json.loads(params)})
print("REFUSED" if "error" in r else "ACCEPTED", json.dumps(r.get("error") or r.get("result"))[:120])
EOF
}

# plugin_round <name> <startup command...>
plugin_round() {
  plugin=$1; shift
  step "== $plugin"
  config_home
  wm_session 1920x1080 || return 1
  "$@" >> "$RUN" 2>&1 &
  pid=$!; track $pid
  ctl wait --timeout 20 "owner:$plugin" > /dev/null || finding "W15: $plugin does not start" breaks "start $plugin"
  # the foreign client of this round: methods.call without methods.privileged
  admin clients create foreign > "$work/foreign.json" 2>> "$RUN"
  foreign_sock=$(py 'import json, sys; print(json.load(sys.stdin).get("path", ""))' < "$work/foreign.json")
  printf "foreign:\n  grants: [methods.call]\n" | admin clients up - >> "$RUN" 2>&1
  voice_start
  expect "$plugin: no claim of the indicator" breaks 'test "$(claims)" = 1' 10
  sleep 0.5
  [ "$(claims)" = 1 ] || finding "W15: $plugin: $(claims) claims, not one" nit "start voice"

  # states
  for s in asleep listening thinking working question speaking dreaming; do
    # dreaming is no state of mc
    [ "$plugin" = mc ] && [ "$s" = dreaming ] && continue
    state "$s"
    sleep 0.7
    case $plugin in
      construct) expect "construct: state $s shows '$(ctl query id:construct-hud-state | py 'import json, sys; n = json.load(sys.stdin); print(n[0]["text"] if n else "")')'" breaks "test \"\$(ctl query id:construct-hud-state | py 'import json, sys; n = json.load(sys.stdin); print(n[0][\"text\"] if n else \"\")')\" = ${s^^}" 5 ;;
      mc) ctl call mc agent.list > "$work/agents.json" 2>&1; grep -q "\"$s\"" "$work/agents.json" || step "mc agent.list lacks $s: $(tr -d '\n ' < "$work/agents.json" | cut -c1-200)" ;;
    esac
    case $s in listening | thinking) shot_wm "$plugin-$s" ;; esac
  done
  health "$plugin states"

  # a normal confirm
  emit voice.confirm "{\"id\": \"41\", \"prompt\": \"$prompt\", \"detail\": \"$detail\"}"
  expect "$plugin: a normal question does not show" breaks 'ui_has "96 projects"' 8
  shot_wm "$plugin-question"
  emit voice.confirm '{"id": "40", "done": true}'
  sleep 0.5
  ui_has "96 projects" || finding "W15: $plugin: a done of another id closed the question" breaks "voice.confirm done id 40"
  emit voice.confirm '{"id": "41", "done": true}'
  expect "$plugin: the done does not close the question" breaks '! ui_has "96 projects"' 8

  # a physical question with buttons: key and click
  for how in click key; do
    rm -f "$work/claims.answers" "$work/claims.keys"
    emit voice.confirm "{\"id\": \"43\", \"prompt\": \"Set allow to [sh]?\", \"detail\": \"allow: [sh]\", \"physical\": true, \"keys\": {\"yes\": \"Alt+y\", \"no\": \"Alt+n\"}, \"buttons\": true}"
    expect "$plugin: the physical question does not show" breaks 'ui_has "allow: [sh]" || ui_has "Set allow"' 8
    case $plugin in construct) yes_id=construct-ask-yes ;; mc) yes_id=mc-answer-yes ;; xp) yes_id=xp-assistant-yes ;; esac
    expect "$plugin: no Yes button on the physical question" breaks "ctl query id:$yes_id | grep -q $yes_id" 8
    sleep 0.5
    shot_wm "$plugin-physical-$how"
    if [ $how = click ]; then
      click_node $yes_id || finding "W15: $plugin: cannot click the Yes button" breaks "click $yes_id"
      expect "$plugin: a click on Yes gave $(cat "$work/claims.answers" 2>/dev/null)" breaks 'grep -qx "{\"id\": \"43\", \"yes\": true}" "$work/claims.answers"' 8
    else
      key Alt+y
      expect "$plugin: Alt+y did not reach the grabber of the voice agent" breaks 'test -s "$work/claims.keys"' 8
    fi
    emit voice.confirm '{"id": "43", "done": true}'
    expect "$plugin: the done does not close the physical question" breaks '! ui_has "allow: [sh]"' 8
  done

  # a foreign client: both methods are privileged, it has methods.call only
  step "foreign claim: $(foreign indicator.claim '{}')"
  emit voice.confirm '{"id": "44", "prompt": "Second question?", "detail": "d", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true}'
  sleep 1
  rm -f "$work/claims.answers"
  out=$(foreign indicator.claim '{}')
  case $out in REFUSED*) ;; *) finding "W15: $plugin: a foreign client got through indicator.claim: $out" endangers "methods.call voice indicator.claim without methods.privileged" ;; esac
  out=$(foreign confirm.answer '{"id": "44", "yes": true}')
  case $out in REFUSED*) ;; *) finding "W15: $plugin: a foreign client got through confirm.answer: $out" endangers "methods.call voice confirm.answer without methods.privileged" ;; esac
  sleep 0.5
  [ ! -s "$work/claims.answers" ] || finding "W15: $plugin: confirm.answer of a foreign client reached the voice agent: $(cat "$work/claims.answers")" endangers "foreign confirm.answer"
  [ "$(claims)" = 1 ] || finding "W15: $plugin: a foreign claim counted: $(claims) claims" endangers "foreign indicator.claim"
  emit voice.confirm '{"id": "44", "done": true}'
  sleep 0.5

  # spoofed events of voice.evil
  # the core already refuses a client voice.evil that emits voice.state;
  # its own namespace voice.evil.* reaches every `voice.*` subscriber
  emit_evil voice.state "{\"state\": \"working\", \"task\": \"$evil\", \"said\": \"$evil\"}"
  for ev in state say confirm; do
    emit_evil voice.evil.$ev "{\"state\": \"working\", \"task\": \"$evil\", \"said\": \"$evil\", \"text\": \"$evil\", \"duration_ms\": 3000, \"id\": \"66\", \"prompt\": \"$evil prompt\", \"detail\": \"$evil\", \"physical\": true, \"keys\": {\"yes\": \"Alt+y\", \"no\": \"Alt+n\"}, \"buttons\": true}"
  done
  emit_evil voice.confirm "{\"id\": \"66\", \"prompt\": \"$evil prompt\", \"detail\": \"$evil\", \"physical\": true, \"keys\": {\"yes\": \"Alt+y\", \"no\": \"Alt+n\"}, \"buttons\": true}"
  state listening
  sleep 2
  ui_has "$evil" && finding "W15: $plugin shows text of a client named voice.evil" endangers "voice.evil emits voice.state, voice.say, voice.confirm"
  case $plugin in
    construct) [ "$(ctl query id:construct-hud-state | py 'import json, sys; n = json.load(sys.stdin); print(n[0]["text"] if n else "")')" = LISTENING ] || finding "W15: construct follows voice.evil: state label changed" endangers "voice.evil voice.state" ;;
    mc) ctl call mc agent.list 2>&1 | grep -qF "$evil" && finding "W15: mc lists an agent state of voice.evil" endangers "voice.evil voice.state" ;;
  esac
  shot_wm "$plugin-evil"
  health "$plugin spoof"

  # the restarted voice claims again
  voice_stop
  sleep 1
  voice_start
  expect "$plugin: no claim after the restart of voice" breaks 'test "$(claims)" = 1' 10
  voice_stop
  kill "$pid" 2>/dev/null
  wait "$pid" 2>/dev/null
  session_stop
}

plugin_round construct "$bin/chora-construct"
plugin_round mc "$bin/chora-mc"
plugin_round xp "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
log_scan
