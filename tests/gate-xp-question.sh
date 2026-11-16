#!/bin/bash
# The question card of xp (xp-27): in the open chat it is set in the look,
# stands under the transcript and not over it, has Yes and No buttons for
# the mouse, and the words "Typing does not answer" show at most once.
# BIN: directory with the binaries, SHOTS: directory for the captures.
set -u
. "$(dirname "$0")/gate-lib.sh"
need jq
need_bins chora choractl chora-xp chora-assistant
shots=${SHOTS:-$(mktemp -d /tmp/gate-xp-question.XXXXXX)}
mkdir -p "$shots"

nodes_with() { ctl query owner:assistant | jq -r --arg w "$1" '[.[] | select(((.text // "") + (.label // "")) | contains($w))] | length'; }

question_cell() {
    local scale=$1 entries
    temp_config
    export HOME=$XDG_CONFIG_HOME/home
    mkdir -p "$HOME"
    session_start "1280x800@$scale" -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
    "$bin/chora-assistant" >"$shots/assistant-$scale.log" 2>&1 &
    pids+=($!)
    grant screen.capture
    entries=$shots/entries.json
    echo '[{"seq": 1, "role": "user", "text": "Please write notes.txt"}, {"seq": 2, "role": "agent", "text": "I will write notes.txt now."}]' >"$entries"
    mkfifo "$shots/voice-$scale.in"
    FAKE_VOICE_ENTRIES=$entries python3 -I "$repo/tests/fake-voice.py" "$shots/claims-$scale" \
        <"$shots/voice-$scale.in" >"$shots/voice-$scale.log" 2>&1 &
    pids+=($!)
    exec 5>"$shots/voice-$scale.in"
    echo 'voice.state {"name": "Atlas", "state": "question", "task": "Write file", "step": 1, "steps": 1, "seq": 2}' >&5
    sleep 1.5
    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null || fail "chat.show failed"
    echo 'voice.confirm {"id": "51", "prompt": "Atlas wants to write the file notes.txt.", "detail": "~/projects/notes/notes.txt (412 bytes)", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true}' >&5
    sleep 3
    ctl capture "$shots/xp-question-${scale}x.png" >/dev/null || fail "the capture failed"
    ctl query owner:assistant >"$shots/nodes-$scale.json"
    exec 5>&-
    gate_cleanup 0
    pids=()
}
question_cell "${SCALE:-1}"
