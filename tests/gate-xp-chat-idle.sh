#!/bin/bash
# The chat header of xp ends its turn honestly (xp-28): after the question
# is answered and the helper reports idle, the header says "Idle" with no
# step and Stop is off. The fake voice plays the turn: working (step 1 of
# 1), a question, the answer, idle.
# BIN: directory with the binaries, SHOTS: directory for the logs.
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need_bins chora choractl chora-xp chora-assistant
shots=${SHOTS:-$(mktemp -d /tmp/gate-xp-chat-idle.XXXXXX)}
mkdir -p "$shots"

temp_config
export HOME=$XDG_CONFIG_HOME/home
mkdir -p "$HOME"
session_start "1280x800@1" -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
start_assistant_like_a_loaded_machine() { sleep 0.8; exec "$bin/chora-assistant"; }
start_assistant_like_a_loaded_machine >"$shots/assistant.log" 2>&1 &
pids+=($!)
grant screen.capture
entries=$shots/entries.json
echo '[{"seq": 1, "role": "user", "text": "Please write notes.txt"}, {"seq": 2, "role": "agent", "text": "Done."}]' >"$entries"
mkfifo "$shots/voice.in"
FAKE_VOICE_ENTRIES=$entries python3 -I "$repo/tests/fake-voice.py" "$shots/claims" \
    <"$shots/voice.in" >"$shots/voice.log" 2>&1 &
pids+=($!)
exec 5>"$shots/voice.in"
echo 'voice.state {"name": "Atlas", "state": "working", "task": "Write file", "step": 1, "steps": 1, "seq": 2, "epoch": 1}' >&5
sleep 1.5
ctl call assistant chat.show '{"name": "atlas"}' >/dev/null || fail "chat.show failed"
ctl wait id:hud-chat --timeout 10 >/dev/null || fail "no chat view"
dump() { ctl query owner:assistant | jq -c '[.[] | {id, text: (.meta.text // .props.text // null)}]'; }
before=$(dump)
echo "$before" | grep -q 'Working' || fail "the header does not say Working while the helper works: $before"
echo 'voice.confirm {"id": "51", "prompt": "Write notes.txt?", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true}' >&5
ctl wait id:hud-card --timeout 10 >/dev/null || fail "no card"
echo 'voice.confirm {"id": "51", "done": true}' >&5
echo 'voice.state {"name": "Atlas", "state": "idle", "task": "", "seq": 2, "epoch": 1}' >&5
sleep 1.5
after=$(dump)
echo "$shots: $after" >"$shots/after.json"
exec 5>&-
echo "$after" | grep -q 'Working' && fail "the header still says Working after the answer: $after"
echo "$after" | grep -q 'step 1 of 1' && fail "the header still says step 1 of 1 after the answer: $after"
echo "$after" | grep -q 'Idle' || fail "the header does not say Idle after the answer: $after"
ok "the header says Idle after the answer, with no step"
gate_cleanup 0
