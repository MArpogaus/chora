#!/bin/bash
# Gate of the keyboard of the chat field in tiling (flow F1-K, F2): with
# the flag --test-injected-press-is-real of the core (headless only),
# where an injected press is an act of the person, Super+a gives the
# field the keyboard. With one helper it opens the chat with that helper:
# typed text and Enter reach the helper (the fake ACP agent of
# services/voice/tests/fake-acp.sh), and Esc closes the chat, so the
# window gets the keyboard back. The first key after that hand-over
# reaches the window; a failure prints the enter, leave and key events
# that the window got. With a second helper Super+a opens the list: text
# typed there reaches no helper until the number of one follows, and
# then the helper of that number. The first helper asks its question
# while the list shows (FAKE_ACP_ASK_AFTER of the fake agent), and the
# number still picks: a question of some helper does not hold the Enter
# of the list (F28). While the chat is open, the card of a notice of
# chora-notify stands beside the chat panel, not over it (rw-3 of kit5).
#
# Usage: tests/gate-chat-keys.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling chora-voice chora-assistant chora-chat chora-probe
temp_config
work=$XDG_CONFIG_HOME
export XDG_DATA_HOME=$work/data XDG_STATE_HOME=$work/state PATH=$bin:$PATH
export FAKE_ACP_ASK_AFTER=$work/ask
mkdir -p "$XDG_CONFIG_HOME/chora" "$work/demo"
cat >"$XDG_CONFIG_HOME/chora/chora-voice.yaml" <<YAML
capture_cmd: [sh, -c, exec sleep 1000]
playback_cmd: [sh, -c, exec cat >/dev/null]
asr_url: ws://127.0.0.1:9
tts_url: ws://127.0.0.1:9
llm_url: http://127.0.0.1:9/v1
earcons: false
indicator: false
unsandboxed: true
agent_cmd: [sh, $repo/services/voice/tests/fake-acp.sh]
agent_projects: {demo: $work/demo}
YAML

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
type_text() {
    local i
    for ((i = 0; i < ${#1}; i++)); do
        key "${1:i:1}"
    done
}
window_has() { [ "$(grep -c "$1" "$work/window")" -ge "$2" ]; }
helper_got() { grep -q "$2" "$XDG_DATA_HOME/chora/agents/$1.log" 2>/dev/null; }

CHORA_TEST_FLAGS=--test-injected-press-is-real session_start -- "$bin/chora-tiling"
grant input.inject
"$bin/chora-voice" >"$work/voice.log" 2>&1 &
pids+=($!)
until_ok 'ctl call voice state >/dev/null 2>&1' 80 || fail "voice does not answer"
ctl call voice agent '{"tool": "agent_start", "project": "demo", "name": "coder"}' >/dev/null ||
    fail "agent_start"
"$bin/chora-assistant" >"$work/assistant.log" 2>&1 &
pids+=($!)
"$bin/chora-probe" clip clipboard-text >"$work/window" 2>&1 &
pids+=($!)
until_ok 'window_has enter 1' 80 || fail "the window has no keyboard"
sleep 1

key Super+a
until_ok 'window_has leave 1' 40 || fail "Super+a did not give the chat field the keyboard"
ok "Super+a gives the chat field the keyboard"

type_text hallo
key Return
until_ok 'helper_got coder hallo' 40 || fail "with one helper, typed text did not reach it"
grep -q hallo "$work/window" && fail "typed text reached the window"
ok "with one helper, typed text and Enter reach the helper"

notify_start
ctl call notify notify.show '{"summary": "Beside the chat", "body": "a notice while the chat is open"}' >/dev/null ||
    fail "notify.show"
until_ok "ctl query id:notify-card-body | grep -q rects" 40 || fail "the notice shows no card"
apart() {
    { ctl query id:hud-chat; ctl query id:notify-card-body; } | python3 -I -c '
import json, sys
text, at, found = sys.stdin.read(), 0, []
while text[at:].strip():
    nodes, at = json.JSONDecoder().raw_decode(text, len(text) - len(text[at:].lstrip()))
    found += nodes
boxes = [next(iter(n["rects"].values())) for n in found if n.get("rects")]
if len(boxes) != 2:
    sys.exit(f"want the boxes of the chat and of the card, got {boxes}")
[ax, ay, aw, ah], [bx, by, bw, bh] = boxes
if ax < bx + bw and bx < ax + aw and ay < by + bh and by < ay + ah:
    sys.exit(f"the card {boxes[1]} covers the chat {boxes[0]}")
'
}
until_ok apart 20 || fail "$(apart 2>&1); notify: $(ctl call notify state.get | jq -c '{shown, chat}')"
ok "a notice of chora-notify steps aside of the open chat panel"

key Escape
until_ok 'window_has enter 2' 40 || fail "Esc did not give the window the keyboard back"
ok "Esc closes the chat and the window gets the keyboard back"

key m
until_ok 'window_has "^key 50$" 1' 20 ||
    fail "the first key after the hand-over did not reach the window: $(tr '\n' ' ' <"$work/window")"
[ "$(tail -n 2 "$work/window" | tr '\n' ' ')" = "enter key 50 " ] ||
    fail "the window got more than the one key: $(tr '\n' ' ' <"$work/window")"
ok "the first key after the hand-over reaches the window"

ctl call voice agent '{"tool": "agent_start", "project": "demo", "name": "second"}' >/dev/null ||
    fail "agent_start of a second helper"
key Super+a
until_ok 'window_has leave 2' 40 || fail "Super+a did not give the list the keyboard"
type_text moin
key Return
sleep 1
helper_got coder moin && fail "a text went to a helper that nobody picked"
: >"$FAKE_ACP_ASK_AFTER"
until_ok "ctl query id:hud-card | grep -q '\"id\"'" 40 || fail "the question of the first helper does not show"
type_text 2
key Return
until_ok 'helper_got second moin' 40 || fail "the text typed before the pick did not reach the picked helper"
helper_got coder moin && fail "the text reached the other helper too"
ok "with two helpers, a text and then a number send the text to that helper"
key Escape
until_ok 'window_has enter 3' 40 || fail "Esc did not give the window the keyboard back"
