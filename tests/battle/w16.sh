#!/bin/bash
# W16 voice agent, for real: the real chora-voice (not tests/fake-voice.py)
# in a headless session beside chora-mc, with a fake ASR, TTS and LLM
# (tests/battle/w16-fakes.py: the Rust fakes of services/voice/src/fake.rs
# are test-only code). The mic is a script that plays a recorded sentence
# on request, the speaker a script that logs what it plays. The ASR and the
# LLM follow a script that this file pushes: what the "user" says, and the
# tool calls of the "LLM".
#
# Regression steps of the live bugs of 2026-10-31:
#  1. a button click on a question stops the speech of that question at once
#  2. settings.reload with a new chora_mcp restarts the tool process
#  3. listen: push_to_talk ignores speech without the key
#  4. a security setting needs a key or button; a spoken yes does nothing
#  5. allow_programs: [foot] does not allow `foot -e sh`
#  6. call_method mc settings.set with a {patch: ...} wrapper works
WF=w16; STOP_ON_DEATH=1; . "$(dirname "$0")/lib.sh"; . "$(dirname "$0")/lib-wm.sh"
config_home
trap 'session_stop' EXIT
wm_session 1280x800 || exit 1
fakes=$repo/tests/battle/w16-fakes.py
vlog=$work/voice.log
cfg=$XDG_CONFIG_HOME/chora/chora-voice.yaml

# --- the fakes, the mic and the settings ---------------------------------
python3 -I - "$repo/services/voice/tests/data/corpus/hallo-chora/mic.wav" "$work/speech.pcm" <<'EOF'
import sys, wave
with wave.open(sys.argv[1]) as w, open(sys.argv[2], "wb") as out:
    assert (w.getframerate(), w.getnchannels(), w.getsampwidth()) == (24000, 1, 2)
    out.write(w.readframes(w.getnframes()))
EOF
: > "$work/mic.cmd"
bg python3 -I "$fakes" servers "$work"
expect "the fakes write no ports" breaks 'test -s "$work/ports"' 10 || exit 1
port() { py "import json; print(json.load(open('$work/ports'))['$1'])"; }
asr=$(port asr) tts=$(port tts) llm=$(port llm)
cat > "$cfg" <<EOF
asr_url: ws://127.0.0.1:$asr
tts_url: ws://127.0.0.1:$tts
llm_url: http://127.0.0.1:$llm/v1
capture_cmd: [python3, -I, $fakes, mic, $work]
playback_cmd: [python3, -I, $fakes, play, $work]
playback_buffer_ms: 200
earcons: false
allow_programs: [foot, "true"]
allow_methods: [mc.settings.set]
chora_mcp: [sh, -c, 'echo A >> $work/mcp.log; exec "\$0" "\$@"', $bin/choractl]
EOF

push() { curl -s -X POST --data-binary "$2" "http://127.0.0.1:$llm/$1" > /dev/null; }
# say <text>: the ASR hears the text, the mic plays a sentence
say() { push _asr "{\"text\": \"$1\"}"; echo speech >> "$work/mic.cmd"; }
tool() { push _llm "{\"tool\": \"$1\", \"args\": $2}"; }
reply() { push _llm "{\"text\": \"$1\"}"; }
vhas() { grep -qE "$1" "$vlog"; }
# the lines of a log file now: the next check looks at the new ones only
mark() { { wc -l < "$1"; } 2> /dev/null || echo 0; }
since() { tail -n "+$(($2 + 1))" "$1" 2> /dev/null; }
# a tool call of the LLM and the question that it makes: wait until the turn ends
turn_done() { expect "$1: the turn did not end (no final answer)" breaks "since $vlog $2 | grep -q 'llm: stop'" "${3:-20}"; }
state() { ctl call voice state 2>&1 | py 'import json, sys; print(json.load(sys.stdin).get("state", ""))' 2> /dev/null; }
settings() { ctl call "$1" settings.get 2> /dev/null; }
# the age in s of the last chunk that the speaker got; big without one
play_age() { py 'import json, sys, time
t = [json.loads(l)["t"] for l in open(sys.argv[1]) if json.loads(l)["ev"] == "chunk"]
print(round(time.time() - t[-1], 2) if t else 999)' "$work/play.log" 2> /dev/null || echo 999; }
node_centre() { ctl query "id:$1" | py 'import json, sys; n = json.load(sys.stdin); r = list(n[0]["rects"].values())[0]; print(round(r[0] + r[2] / 2), round(r[1] + r[3] / 2))' 2> /dev/null; }
click_node() {
  local c
  c=$(node_centre "$1") || return 1
  [ -n "$c" ] || return 1
  inject "[{\"pointer_move\": {\"position\": [${c% *}, ${c#* }]}}, {\"click\": {}}]" > /dev/null
}
button() { ctl query "id:$1" 2> /dev/null | grep -q "\"$1\""; }

# --- start: mc, the voice agent ---------------------------------------------
# the tool process of the agent is a client of its own, with the grants that the user gives it
admin grant voice-mcp methods.call methods.privileged activation.token >> "$RUN" 2>&1
bg "$bin/chora-mc"
ctl wait --timeout 20 owner:mc > /dev/null || finding "W16: mc does not start" breaks "chora-mc"
RUST_LOG="chora_voice=debug,info,tract_linalg=warn,rmcp=warn" "$bin/chora-voice" >> "$vlog" 2>&1 &
vpid=$!; track $vpid
expect "voice does not answer" breaks 'ctl call voice state > /dev/null 2>&1' 30 || exit 1
expect "the voice agent does not say that it is ready" breaks 'grep -q "Ready" "$work/tts.log" 2>/dev/null' 30
# the speaker must be quiet before the first step
expect "the ready phrase never ended" nit 'test "$(play_age)" != 999 && py "import sys; sys.exit(not $(play_age) > 1.5)"' 15
ctl wait --timeout 10 owner:mc > /dev/null
health "voice started"
step "voice state: $(state)"

# === 1. a click on the button stops the speech of the question ============
step "== 1 click stops the question"
tool voice__settings__set '{"patch": {"allow_programs": ["sh"]}}'
reply "Not done."
l0=$(mark "$vlog"); a0=$(mark "$work/llm.log")
say "Please allow the shell."
expect "1: the physical question shows no No button in mc" breaks 'button mc-answer-no' 25
# the question must be in the speaker when the click comes: it follows
# "One moment." in the same speaker, so wait for its words
expect "1: the question was never sent to the TTS" breaks 'grep -q "Security setting" "$work/tts.log"' 15
expect "1: the speaker plays nothing of the question" breaks 'py "import sys; sys.exit(not $(play_age) < 0.3)"' 15
sleep 1.5
t0=$(date +%s.%N)
click_node mc-answer-no || finding "W16: cannot click the No button" breaks "click mc-answer-no"
expect "1: voice did not take the click as a no" breaks "since $vlog $l0 | grep -q 'confirm: no'" 8
sleep 1
# the speaker process of the question (the last one that started before
# the click) got nothing 0.5 s after the click, and is gone; what it got
# is a part of the question, not all of it
py 'import json, os, sys
t0 = float(sys.argv[2])
log = [json.loads(l) for l in open(sys.argv[1])]
pid = [e for e in log if e["ev"] == "start" and e["t"] < t0][-1]["pid"]
chunks = [e for e in log if e["ev"] == "chunk" and e["pid"] == pid]
tts = [json.loads(l) for l in open(sys.argv[3])]
question = next(e for e in tts if "Security setting" in e["text"])
whole = len(question["text"]) * 2880
late = round(chunks[-1]["t"] - t0, 2)
total = chunks[-1]["total"]
try:
    os.kill(pid, 0)
    alive = True
except OSError:
    alive = False
print(f"speaker {pid}: last chunk {late:+} s after the click, {total} of {whole} bytes of the question, alive: {alive}")
sys.exit(0 if late < 0.5 and total < whole * 0.8 and not alive else 1)' "$work/play.log" "$t0" "$work/tts.log" > "$work/step1.txt" 2>&1
r=$?
step "1: $(cat "$work/step1.txt")"
[ $r = 0 ] || finding "W16: a click on a question did not stop its speech: $(cat "$work/step1.txt")" breaks "ask for allow_programs, click mc-answer-no while it speaks"
turn_done "1" "$l0"
settings voice | py 'import json, sys; sys.exit("sh" not in json.load(sys.stdin)["allow_programs"])' && finding "W16: a no on the button changed allow_programs" endangers "click no"
health "step 1"

# === 5. allow_programs [foot] does not allow foot -e sh ====================
step "== 5 allow_programs"
tool run_program '{"argv": ["true"]}'
reply "Ran true."
l0=$(mark "$vlog")
say "Run true."
turn_done "5 control" "$l0"
since "$vlog" "$l0" | grep -q 'in allow, no question' || finding "W16: run_program true (in allow_programs) asked or did not run" breaks "allow_programs [foot, true], run_program [true]"
tool run_program '{"argv": ["foot", "-e", "sh"]}'
reply "Not run."
l0=$(mark "$vlog")
say "Open a shell in a terminal."
expect "5: foot -e sh asked no question" endangers "since $vlog $l0 | grep -q 'say: Shall I start the program foot'" 20
sleep 2
since "$vlog" "$l0" | grep -q 'run_program: in allow' && finding "W16: allow_programs [foot] allowed foot -e sh" endangers "run_program [foot, -e, sh]"
# answer by voice: no
say "no"
turn_done "5" "$l0" 25
since "$vlog" "$l0" | grep -qE 'tool run_program.*foot' && finding "W16: foot -e sh ran after a no" endangers "run_program foot -e sh, answer no"
health "step 5"

# === 6. call_method mc settings.set with a patch wrapper ===================
step "== 6 call_method settings.set {patch}"
tool call_method '{"client": "mc", "method": "settings.set", "params": {"patch": {"default_scene": "overview"}}}'
reply "Set."
l0=$(mark "$vlog"); a0=$(mark "$work/llm.log")
say "Make overview the default scene."
turn_done "6" "$l0"
expect "6: mc default_scene is $(settings mc | py 'import json, sys; print(json.load(sys.stdin).get("default_scene"))' 2>/dev/null), not overview" breaks 'settings mc | grep -q "\"default_scene\": *\"overview\""'
# the result that the LLM got back is no error
since "$work/llm.log" "$a0" | tail -1 | py 'import json, sys
m = [m for m in json.loads(sys.stdin.read())["messages"] if m["role"] == "tool"][-1]["content"]
print(str(m)[:150]); sys.exit(str(m).startswith("error"))' > "$work/step6.txt" 2>&1 || finding "W16: call_method mc settings.set with a patch wrapper failed: $(cat "$work/step6.txt")" breaks "call_method mc settings.set params {patch: {default_scene}}"
step "6: $(cat "$work/step6.txt")"
health "step 6"

# === 2. settings.reload restarts the tool process ==========================
step "== 2 reload restarts chora_mcp"
before=$(cat "$work/mcp.log" 2> /dev/null | tr '\n' ' ')
sed -i 's/echo A >>/echo B >>/' "$cfg"
l0=$(mark "$vlog")
ctl call voice settings.reload '{}' > "$work/reload.json" 2>&1 || finding "W16: settings.reload refused: $(head -c 150 "$work/reload.json")" breaks "choractl call voice settings.reload"
expect "2: the log shows no restart of the tool process" breaks "since $vlog $l0 | grep -q 'mcp: restart with'" 10
tool call_method '{"client": "mc", "method": "scene.list", "params": {}}'
reply "Listed."
l0=$(mark "$vlog"); a0=$(mark "$work/llm.log")
say "Which scenes are there?"
turn_done "2" "$l0"
step "2: mcp.log before [$before] after [$(tr '\n' ' ' < "$work/mcp.log")]"
[ "$(tail -1 "$work/mcp.log")" = B ] || finding "W16: a tool call after the reload did not use the new chora_mcp: mcp.log $(tr '\n' ' ' < "$work/mcp.log")" breaks "edit chora_mcp, settings.reload, tool call"
since "$work/llm.log" "$a0" | tail -1 | py 'import json, sys
m = [m for m in json.loads(sys.stdin.read())["messages"] if m["role"] == "tool"][-1]["content"]
sys.exit(not "overview" in str(m))' || finding "W16: the tool call after the reload gave no scene list" breaks "call_method mc scene.list after reload"
health "step 2"

# === 4. a security key change needs a key or button ========================
step "== 4 security setting"
tool voice__settings__set '{"patch": {"always_words": ["always", "immer", "for sure"]}}'
reply "Done."
l0=$(mark "$vlog")
say "Add a word for always."
expect "4: no physical question (state: $(state))" breaks 'button mc-answer-yes' 25
sleep 3
# the spoken yes: the ASR hears it, the question stays
say "yes"
expect "4: the ASR did not get the yes" nit 'grep -qx yes "$work/asr.log"' 15
sleep 3
since "$vlog" "$l0" | grep -q 'confirm: yes' && finding "W16: a spoken yes answered a security question" endangers "settings.set always_words, say yes"
button mc-answer-yes || finding "W16: the question closed on a spoken yes" endangers "settings.set always_words, say yes"
settings voice | grep -q 'for sure' && finding "W16: a spoken yes changed a security setting" endangers "settings.set always_words, say yes"
# the key answers it
key Alt+y
turn_done "4" "$l0"
since "$vlog" "$l0" | grep -q 'confirm: yes' || finding "W16: Alt+y did not answer the question" breaks "Alt+y at a security question"
settings voice | grep -q 'for sure' || finding "W16: the key yes did not change the setting" breaks "Alt+y at a security question"
health "step 4"

# 4b. the same change by call_method: the key answers, and the call reaches the agent
step "== 4b security setting by call_method"
tool call_method '{"client": "voice", "method": "settings.set", "params": {"patch": {"always_words": ["always", "immer", "for sure", "really"]}}}'
reply "Done."
l0=$(mark "$vlog"); a0=$(mark "$work/llm.log")
say "Add one more word for always."
expect "4b: no physical question" breaks 'button mc-answer-yes' 25
key Alt+y
turn_done "4b" "$l0"
tool_result() { since "$work/llm.log" "$a0" | tail -1 | py 'import json, sys
m = [m for m in json.loads(sys.stdin.read())["messages"] if m["role"] == "tool"][-1]["content"]
print(str(m)[:200])'; }
step "4b: tool result: $(tool_result)"
settings voice | grep -q 'really' || finding "W16: call_method voice settings.set after the key yes did not change the setting: $(tool_result)" breaks "call_method voice settings.set {patch: {always_words}}, Alt+y"
health "step 4b"

# === 3. listen: push_to_talk ==============================================
step "== 3 push to talk"
printf 'listen: push_to_talk\n' >> "$cfg"
ctl call voice settings.reload '{}' > /dev/null 2>&1
expect "3: the state record does not tell push_to_talk" breaks 'ctl call voice state | grep -q push_to_talk' 10
sleep 1
reply "Heard you."
a0=$(mark "$work/llm.log"); c0=$(mark "$work/asr.log"); l0=$(mark "$vlog")
echo speech >> "$work/mic.cmd"
sleep 8
[ "$(mark "$work/asr.log")" = "$c0" ] || finding "W16: push_to_talk: speech without the key reached the ASR" breaks "listen push_to_talk, speech without Super+v"
[ "$(mark "$work/llm.log")" = "$a0" ] || finding "W16: push_to_talk: speech without the key started a turn" breaks "listen push_to_talk, speech without Super+v"
# the same sentence with the key held. `input.inject` lets go of every key
# at the end of its request, so the hold is `voice talk` (the same input
# of the core as the key); the key itself is tapped below.
ctl call voice talk '{"down": true}' > /dev/null
say "key words"
sleep 7
ctl call voice talk '{"down": false}' > /dev/null
expect "3: speech with the key held was not heard" breaks 'grep -qx "key words" "$work/asr.log"' 15
turn_done "3" "$l0"
since "$work/llm.log" "$a0" | grep -q "key words" || finding "W16: the words said with the key held did not reach the LLM" breaks "listen push_to_talk, hold, speech"
# a tap of the real key reaches the agent (the grab of Super+v)
l1=$(mark "$vlog")
key Super+v
expect "3: the key Super+v did not reach the agent" breaks "since $vlog $l1 | grep -q 'ptt down'" 8
step "3: asr.log $(tr '\n' '|' < "$work/asr.log")"
health "step 3"

voice_down=$(ctl call voice state > /dev/null 2>&1 && echo up || echo down)
[ "$voice_down" = up ] || finding "W16: the voice agent is gone at the end" crash "end of W16"
log_scan
cp "$vlog" "$shots/w16-voice.log"
grep -c "panicked" "$vlog" | grep -qx 0 || finding "W16: the voice agent panicked: $(grep -m1 panicked "$vlog" | cut -c1-150)" crash "see $shots/w16-voice.log"
