#!/bin/bash
# Test of the chat field (docs/voice.md, "The chat field") in a headless
# session: chora-chat --input runs as a layer surface beside a window of
# chora-probe and the fake ACP agent of services/voice/tests/fake-acp.sh.
#
# It checks that the field takes the keyboard only by a click and never by
# a token without a real act, that typed, composed (text-input-v3) and
# pasted text reach the agent with Enter, that a typed yes answers nothing
# while a question waits, that the field keeps its draft when it is placed
# again and when Esc closes it, and that the window gets the keyboard back.
# A real press needs a device; the headless session has none, so the path
# of a token of a real act is a test of the core (state.rs).
#
# Usage: tests/chat.sh   (needs the debug binaries, or BIN)
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${BIN:-$repo/target/debug}
tmp=$(mktemp -d /tmp/chora-chat.XXXXXX)
pids=()
cleanup() {
    local pid
    for pid in "${pids[@]}"; do
        kill "$pid" 2>/dev/null || true
    done
    for pid in "${pids[@]}"; do
        wait "$pid" 2>/dev/null || true
    done
    rm -rf "$tmp"
}
trap cleanup EXIT INT TERM
fail() {
    echo "FAIL: $*" >&2
    tail -n 5 "$tmp/field.log" "$tmp/events" "$tmp/window" >&2 2>/dev/null || true
    exit 1
}

export XDG_CONFIG_HOME=$tmp/config XDG_DATA_HOME=$tmp/data XDG_STATE_HOME=$tmp/state
mkdir -p "$XDG_CONFIG_HOME/chora" "$tmp/demo"
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
agent_projects: {demo: $tmp/demo}
YAML

CHORA_BIN=$bin "$repo/tests/headless-session.sh" >"$tmp/env" &
pids+=($!)
for _ in $(seq 100); do
    grep -q CHORA_SOCKET "$tmp/env" && break
    sleep 0.1
done
set -a
# shellcheck disable=SC1091
. "$tmp/env"
set +a
ctl() { timeout 10 "$bin/choractl" --name check "$@"; }
"$bin/choractl" grant check input.inject >/dev/null

PATH=$bin:$PATH "$bin/chora-voice" >"$tmp/voice.log" 2>&1 &
pids+=($!)
until ctl call voice state >/dev/null 2>&1; do sleep 0.2; done

ctl events 'chat-field.*' >"$tmp/events" 2>&1 &
pids+=($!)
"$bin/chora-probe" clip clipboard-text >"$tmp/window" 2>&1 &
pids+=($!)
window_id=$(ctl wait 'kind:surface' | python3 -c 'import json,sys; print(json.loads(sys.stdin.readline())[0]["id"])')
printf '[{"attach": {"surface": "%s", "parent": "root"}}, {"focus": {"surface": "%s"}}]' \
    "$window_id" "$window_id" | ctl apply >/dev/null

count() { grep -c -- "$2" "$1" || true; }
wait_for() {
    local what=$1 file=$2 pattern=$3 want=${4:-1}
    for _ in $(seq 100); do
        [[ $(count "$file" "$pattern") -ge $want ]] && return 0
        sleep 0.1
    done
    fail "$what: no $pattern in $file ($(tail -n 3 "$file" | tr '\n' '|'))"
}
stays() {
    local what=$1 file=$2 pattern=$3 before
    before=$(count "$file" "$pattern")
    sleep 1.5
    [[ $(count "$file" "$pattern") == "$before" ]] || fail "$what"
}
inject() { printf "%s" "$1" | ctl inject >/dev/null; }
press() { inject "[{\"shortcut\": {\"keys\": \"$1\"}}]"; }
type_text() {
    local c
    for ((i = 0; i < ${#1}; i++)); do
        c=${1:i:1}
        press "$c"
    done
}
click() { inject "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"click\": {}}]"; }
agent_log=$XDG_DATA_HOME/chora/agents/coder.log
state() {
    ctl call voice agent '{"tool": "agent_status", "name": "coder"}' | tr -d '"'
}
place() { ctl call chat-field place "$1" >/dev/null; }
bottom='{"anchor": "bottom", "margin": [0, 0, 40, 0], "width": 400}'
top='{"anchor": "top", "margin": [40, 0, 0, 0], "width": 400}'
field_bottom=(640 742)
field_top=(640 58)

ctl call voice agent '{"tool": "agent_start", "project": "demo", "name": "coder"}' >/dev/null
"$bin/chora-chat" --input coder >"$tmp/field.log" 2>&1 &
pids+=($!)
for _ in $(seq 100); do
    ctl call chat-field place "$bottom" >/dev/null 2>&1 && break
    sleep 0.1
done
wait_for "the window has the keyboard" "$tmp/window" enter
stays "a field without a token took the keyboard" "$tmp/events" '"on":true'
[[ $(count "$tmp/events" '"on":true') == 0 ]] || fail "a field without a token took the keyboard"
echo "ok: a field placed without a token takes no keyboard"

token=$(ctl run sh -c true | python3 -c 'import json,sys; print(json.load(sys.stdin)["token"])')
place "{\"anchor\": \"bottom\", \"margin\": [0, 0, 40, 0], \"token\": \"$token\"}"
stays "a token without a real act gave the keyboard" "$tmp/events" '"on":true'
[[ $(count "$tmp/events" '"on":true') == 0 ]] || fail "a token without a real act gave the keyboard"
echo "ok: a token that no real press stamped gives no keyboard"

click "${field_bottom[@]}"
wait_for "a click on the field" "$tmp/events" '"on":true'
wait_for "the window lost the keyboard" "$tmp/window" leave
echo "ok: a click on the field takes the keyboard"

type_text x
inject '[{"text": {"text": "ÄÖ"}}]'
press Ctrl+v
press Return
wait_for "Enter, a commit and a paste reach the agent" "$agent_log" 'xÄÖclipboard-text'
echo "ok: Enter sends typed, composed and pasted text to the agent"

for _ in $(seq 50); do
    [[ $(state) == *question* ]] && break
    sleep 0.2
done
[[ $(state) == *question* ]] || fail "no question: $(state)"
type_text yes
press Return
press Return
sleep 2
[[ $(state) == *question* ]] || fail "a typed yes answered the question: $(state)"
[[ $(count "$agent_log" '"outcome":"selected"') == 0 ]] || fail "a typed yes answered"
[[ $(count "$agent_log" 'session/prompt') == 1 ]] || fail "a held text went out at once"
echo "ok: a typed yes answers nothing and sends nothing while a question waits"

place "$top"
click 640 400
wait_for "a click on the window" "$tmp/events" '"on":false'
click "${field_top[@]}"
wait_for "the field at its new place" "$tmp/events" '"on":true' 2
echo "ok: the field moves to a new place"

ctl call voice confirm '{"answer": "no"}' >/dev/null
wait_for "the held text goes after the question" "$agent_log" '"text":"yes"'
[[ $(count "$agent_log" 'session/prompt') == 2 ]] || fail "the held text was not one prompt"
echo "ok: the draft survived the new place and goes after the question"

for _ in $(seq 50); do
    [[ $(state) == *question* ]] && break
    sleep 0.2
done
ctl call voice confirm '{"answer": "no"}' >/dev/null
sleep 1

type_text draft
windows=$(count "$tmp/window" enter)
press Escape
wait_for "Esc closes the field" "$tmp/events" 'chat-field.closed'
wait_for "the window has the keyboard back" "$tmp/window" enter $((windows + 1))
echo "ok: Esc closes the field and the window gets the keyboard back"

place "$bottom"
click "${field_bottom[@]}"
wait_for "the field again" "$tmp/events" '"on":true' 3
press Return
wait_for "the draft after Esc" "$agent_log" '"text":"draft"'
echo "ok: the draft stays after Esc"

if grep -qiE 'panic|error' "$tmp/field.log"; then
    fail "the field logged: $(tail -n 3 "$tmp/field.log")"
fi
ctl call chat-field quit >/dev/null
echo "ok: chat field"
