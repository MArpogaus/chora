#!/bin/bash
# Smoke test of the ACP sub-agents of chora-voice (docs/voice.md section
# 18) with chora-mc in a headless session: the fake agent of
# services/voice/tests/fake-acp.sh starts, works, asks, gets a yes, is
# done and stops, and mc shows each state. No audio and no servers.
# Its question edits a file, so it takes a key or a HUD button (section
# 18, "Questions"): `voice confirm yes` counts as nothing, and tests/
# voice-hud.py answers as a HUD does, with `confirm.answer`: it claims the
# indicator before the question and answers once the go file exists. The
# `voice confirm yes` before that answers nothing.
#
# Usage: tests/voice-acp.sh   (needs the debug binaries, or BIN)
#   SHOTS  a directory for screenshots of the agents scene (optional)
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${BIN:-$repo/target/debug}
tmp=$(mktemp -d /tmp/chora-acp.XXXXXX)
session=
voice=
hud=
cleanup() {
    [[ -z $hud ]] || kill "$hud" 2>/dev/null || true
    if [[ -n $voice ]]; then
        pkill -P "$voice" 2>/dev/null || true
        kill "$voice" 2>/dev/null || true
        wait "$voice" 2>/dev/null || true
    fi
    if [[ -n $session ]]; then
        kill "$session" 2>/dev/null || true
        wait "$session" 2>/dev/null || true
    fi
    rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

export XDG_CONFIG_HOME=$tmp/config XDG_DATA_HOME=$tmp/data XDG_STATE_HOME=$tmp/state
export FAKE_ACP_DELAY=2
mkdir -p "$XDG_CONFIG_HOME/chora" "$tmp/demo"
# No sound and no servers: port 9 refuses at once. The fake agent runs
# without the sandbox.
cat >"$XDG_CONFIG_HOME/chora/chora-voice.yaml" <<EOF
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
# The main socket, unconfined: voice.agent and voice.confirm are
# privileged.
ctl() { "$bin/choractl" --name check "$@"; }
until ctl call mc agent.list >/dev/null 2>&1; do sleep 0.2; done
PATH=$bin:$PATH "$bin/chora-voice" >"$tmp/voice.log" 2>&1 &
voice=$!
until ctl call voice state >/dev/null 2>&1; do sleep 0.2; done
ctl call mc scene.show '{"name": "agents"}' >/dev/null
if [[ -n ${SHOTS:-} ]]; then
    mkdir -p "$SHOTS"
    ctl grant check screen.capture >/dev/null
fi

state() {
    ctl call mc agent.list | python3 -c '
import json, sys
agents = json.load(sys.stdin)["agents"]
print(next((a["state"] for a in agents if a["name"] == "coder"), "none"))'
}
expect() {
    for _ in $(seq 60); do
        [[ $(state) == "$1" ]] && break
        sleep 0.25
    done
    [[ $(state) == "$1" ]] || {
        echo "FAIL: mc shows coder as $(state), not $1" >&2
        tail -20 "$tmp/voice.log" >&2
        exit 1
    }
    echo "ok: mc shows coder as $1"
    if [[ -n ${SHOTS:-} ]]; then
        sleep 1
        ctl capture "$SHOTS/acp-$1.png" >/dev/null
    fi
}

python3 "$repo/tests/voice-hud.py" yes "$tmp/go" >"$tmp/hud.log" 2>&1 &
hud=$!
ctl call voice agent '{"tool": "agent_start", "project": "demo", "name": "coder"}' >/dev/null
expect idle
ctl call voice agent '{"tool": "agent_say", "name": "coder", "text": "Schreib notes.txt."}' >/dev/null
expect working
expect question
ctl call voice confirm '{"answer": "yes"}' >/dev/null
sleep 1
[[ $(state) == question ]] || {
    echo "FAIL: a yes without the key answered the question: coder is $(state)" >&2
    exit 1
}
echo "ok: a yes without the key counts as nothing"
touch "$tmp/go"
wait "$hud" || {
    echo "FAIL: the HUD answer: $(cat "$tmp/hud.log")" >&2
    tail -20 "$tmp/voice.log" >&2
    exit 1
}
expect "done"
ctl call voice agent '{"tool": "agent_stop", "name": "coder"}' >/dev/null
expect none
grep -q '"method":"session/request_permission"' "$tmp/data/chora/agents/coder.log" || {
    echo "FAIL: no question in the log of the agent" >&2
    exit 1
}
echo "ok: the agent log has the ACP messages"
