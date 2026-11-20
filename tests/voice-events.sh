#!/bin/bash
# Smoke test of client events (D65): chora-mc and chora-voice in a
# headless session. The voice agent emits voice.state on a change of its
# state. The HUD host draws it, not chora-mc: mc takes no state of the
# voice agent itself and shows only its helpers (docs/configuration.md,
# "The HUD host chora-assistant draws the voice HUD of mc"; the unit test
# the_host_draws_the_voice_status of chora-mc). So the script reads the
# events with `choractl events`, and checks that mc shows no agent
# `voice`. The agent runs without audio and with servers that do not
# answer.
#
# Usage: tests/voice-events.sh   (needs the debug binaries, or BIN)
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${BIN:-$repo/target/debug}
tmp=$(mktemp -d /tmp/chora-ve.XXXXXX)
session=
voice=
cleanup() {
    if [[ -n $voice ]]; then
        # The capture command is a child of the agent.
        pkill -P "$voice" 2>/dev/null || true
        kill "$voice" 2>/dev/null || true
    fi
    if [[ -n $session ]]; then
        kill "$session" 2>/dev/null || true
        wait "$session" 2>/dev/null || true
    fi
    rm -rf "$tmp"
}
trap cleanup EXIT

export XDG_CONFIG_HOME=$tmp/config XDG_DATA_HOME=$tmp/data XDG_STATE_HOME=$tmp/state
mkdir -p "$XDG_CONFIG_HOME/chora"
# No sound and no servers: port 9 refuses at once.
cat >"$XDG_CONFIG_HOME/chora/chora-voice.yaml" <<'EOF'
capture_cmd: [sh, -c, exec sleep 1000]
playback_cmd: [sh, -c, exec cat >/dev/null]
asr_url: ws://127.0.0.1:9
tts_url: ws://127.0.0.1:9
llm_url: http://127.0.0.1:9/v1
earcons: false
EOF

CHORA_BIN=$bin "$repo/tests/headless-session.sh" -- "$bin/chora-mc" >"$tmp/env" &
session=$!
for _ in $(seq 100); do
    grep -q CHORA_SOCKET "$tmp/env" && break
    sleep 0.1
done
set -a
# shellcheck disable=SC1091
. "$tmp/env"
set +a
ctl() { "$bin/choractl" --name check "$@"; }
until ctl call mc agent.list >/dev/null 2>&1; do sleep 0.2; done

PATH=$bin:$PATH "$bin/chora-voice" >"$tmp/voice.log" 2>&1 &
voice=$!
until ctl call voice state >/dev/null 2>&1; do sleep 0.2; done

ctl events voice.state >"$tmp/events" &
watcher=$!
sleep 0.3
state_of_voice() {
    ctl call mc agent.list | python3 -c '
import json, sys
agents = json.load(sys.stdin)
agents = agents.get("agents", agents) if isinstance(agents, dict) else agents
print(next((a["state"] for a in agents if a["name"] == "voice"), "none"))'
}
event_states() {
    python3 -c '
import json, sys
for line in open(sys.argv[1]):
    event = json.loads(line)
    data = event.get("data") or {}
    if event.get("client") == "voice" and "name" not in data:
        print(data.get("state", ""))' "$tmp/events"
}
for call in sleep:asleep wake:listening; do
    ctl call voice "${call%%:*}" >/dev/null
    for _ in $(seq 25); do
        [[ $(event_states | tail -1) == "${call#*:}" ]] && break
        sleep 0.2
    done
    got=$(event_states | tail -1)
    [[ $got == "${call#*:}" ]] || {
        echo "FAIL: after voice ${call%%:*}, the last voice.state is '$got', not ${call#*:}" >&2
        cat "$tmp/events" >&2
        exit 1
    }
    echo "ok: voice ${call%%:*} emits voice.state ${call#*:}"
    [[ $(state_of_voice) == none ]] || {
        echo "FAIL: mc shows the voice agent as $(state_of_voice); the HUD host draws it" >&2
        exit 1
    }
    echo "ok: mc shows no agent voice after voice ${call%%:*}"
done
kill $watcher
grep -q '"event":"voice.state".*"client":"voice"' "$tmp/events" || {
    echo "FAIL: choractl events saw no voice.state" >&2
    cat "$tmp/events" >&2
    exit 1
}
echo "ok: choractl events voice.state saw $(wc -l <"$tmp/events") events"
