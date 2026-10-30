#!/bin/bash
# The live gate of the voice agent (docs/voice.md section 13, WP8): one
# scripted voice turn through the whole agent against the servers of the
# settings, with the latency of each stage.
#
# Usage: services/voice/tests/live-gate.sh [TEXT]
#
# It speaks TEXT with the TTS (voice GATE_VOICE), starts chora-voice as
# the client voice-gate with a capture command that plays that clip
# into the agent in real time once the LLM is warm, waits for the reply,
# and prints the stages. Exit status 0 for an answered turn.
#
# Needs a chora session (CHORA_SOCKET), and chora-voice and choractl on
# PATH or in VOICE_BIN. The running agent `voice` stays as it is; the
# gate has its own name, no indicator and no push-to-talk key of its own.
#
# Environment:
#   VOICE_CONFIG   the settings with the servers (default
#                  $XDG_CONFIG_HOME/chora/chora-voice.yaml); the gate takes
#                  only the server keys of it
#   VOICE_BIN      the directory of chora-voice and choractl
#   GATE_VOICE     the voice of the user (default vivian)
#   GATE_PLAYBACK  1 plays the reply with the playback command of the
#                  settings; else the sound goes to /dev/null
#   GATE_TIMEOUT   seconds for the warm-up and for the reply (default 120)
set -euo pipefail

text=${1:-"Welche Fenster sind gerade offen?"}
[[ -n ${CHORA_SOCKET:-} ]] || { echo "CHORA_SOCKET is not set" >&2; exit 2; }
if [[ -n ${VOICE_BIN:-} ]]; then
    PATH=$VOICE_BIN:$PATH
fi
command -v chora-voice > /dev/null || { echo "no chora-voice" >&2; exit 2; }
base=${VOICE_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config}/chora/chora-voice.yaml}
timeout=${GATE_TIMEOUT:-120}

dir=$(mktemp -d)
agent=
cleanup() {
    local status=$?
    if [[ -n $agent ]]; then
        kill "$agent" 2> /dev/null || true
        wait "$agent" 2> /dev/null || true
    fi
    if [[ -n ${GATE_LOG:-} ]]; then
        cp "$dir/agent.log" "$GATE_LOG" || true
    fi
    rm -rf "$dir"
    exit "$status"
}
trap cleanup EXIT

# The server keys of the settings; the rest takes the defaults.
keys='asr_url|tts_url|llm_url|model|language|tts_language|context_tokens|reasoning_effort|chora_mcp'
{
    [[ -f $base ]] && grep -E "^($keys):" "$base" || true
    echo "indicator: false"
    echo "earcons: false"
    echo "ptt_key: Super+Alt+End"
    echo "capture_cmd: [python3, $dir/feed.py, $dir/question.wav, $dir/go]"
    if [[ ${GATE_PLAYBACK:-0} != 1 ]]; then
        echo 'playback_cmd: [sh, -c, "cat > /dev/null"]'
    fi
} > "$dir/gate.yaml"

# The voice of the user for the question.
sed "/^voice:/d" "$dir/gate.yaml" > "$dir/tts.yaml"
echo "voice: ${GATE_VOICE:-vivian}" >> "$dir/tts.yaml"
chora-voice --config "$dir/tts.yaml" tts "$text" --out "$dir/question.wav" > "$dir/tts.out"

# Quiet frames until the flag file exists, then the clip in real time,
# then quiet again: 768 samples of PCM16 each 32 ms.
cat > "$dir/feed.py" << 'EOF'
import os, sys, time, wave
clip, flag = sys.argv[1], sys.argv[2]
with wave.open(clip) as w:
    pcm = w.readframes(w.getnframes())
frame, out, at = 768 * 2, sys.stdout.buffer, time.monotonic()
played = False
while True:
    if not played and os.path.exists(flag):
        for i in range(0, len(pcm), frame):
            out.write(pcm[i:i + frame].ljust(frame, b"\0"))
            at += 0.032
            time.sleep(max(0.0, at - time.monotonic()))
        played = True
    out.write(b"\0" * frame)
    out.flush()
    at += 0.032
    time.sleep(max(0.0, at - time.monotonic()))
EOF

chora-voice --name voice-gate --config "$dir/gate.yaml" 2> "$dir/agent.log" &
agent=$!

# Waits for a line in the log of the agent.
await() {
    local deadline=$((SECONDS + timeout))
    until grep -qE "$1" "$dir/agent.log"; do
        if ((SECONDS > deadline)) || ! kill -0 "$agent" 2> /dev/null; then
            echo "no '$1' within ${timeout}s; the log:" >&2
            tail -n 30 "$dir/agent.log" >&2
            return 1
        fi
        sleep 0.2
    done
}

# The question starts once the prompt is in the cache of the LLM.
await 'llm warm=|llm warm-up:'
touch "$dir/go"
await 'reply=1 .*outcome='

# field LINE KEY: the value of KEY= in the first log line with LINE.
field() {
    grep -E "$1" "$dir/agent.log" | head -n 1 | grep -oE "(^| )$2=[^ ]+" | cut -d= -f2- || true
}
echo "text: $text"
echo "clip: $(grep -oE ' audio=[^ ]+' "$dir/tts.out" | cut -d= -f2) of speech," \
    "TTS took $(grep -oE 'total=[^ ]+' "$dir/tts.out" | cut -d= -f2)"
printf '%-36s %s\n' \
    "LLM warm-up" "$(field 'llm warm=' warm)" \
    "speech (VAD)" "$(field 'turn=1 speech=' speech)" \
    "end of speech to end of utterance" "$(field 'turn=1 speech=' end_wait)" \
    "ASR, final text after the commit" "$(field 'turn=1 asr=' asr)" \
    "LLM first token after the text" "$(field 'reply=1 ' llm_first)" \
    "tools (calls/time)" "$(field 'reply=1 ' tools)" \
    "LLM rounds" "$(field 'reply=1 ' rounds)" \
    "TTS first audio" "$(field 'reply=1 ' tts_first)" \
    "first sound after the utterance" "$(field 'reply=1 ' first_sound)" \
    "outcome" "$(field 'reply=1 ' outcome)"
grep -oE 'turn=1 asr=.*' "$dir/agent.log" | grep -oE 'text=.*' | sed 's/^text=/heard: /' || true
grep -oE 'tool [a-z_]+ .* [0-9]+ms' "$dir/agent.log" | cut -c1-100 || true
grep -oE 'say: .*' "$dir/agent.log" | cut -c1-100 || true
grep -oE '(ERROR|WARN).*' "$dir/agent.log" | sed 's/^/log: /' || true
[[ $(field 'reply=1 ' outcome) == answered ]]
