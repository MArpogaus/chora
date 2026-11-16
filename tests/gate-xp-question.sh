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

rect_of() { jq -r --arg id "$2" '[.[] | select(.id == $id)][0].rects | to_entries[0].value | map(floor) | join(" ")' "$1"; }
inside() {
    local ax ay aw ah bx by bw bh
    read -r ax ay aw ah <<<"$1"
    read -r bx by bw bh <<<"$2"
    [ "$bx" -ge "$ax" ] && [ "$by" -ge "$ay" ] && [ $((bx + bw)) -le $((ax + aw)) ] && [ $((by + bh)) -le $((ay + ah)) ]
}

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
    local nodes=$shots/nodes-$scale.json chat card yes no hint
    chat=$(rect_of "$nodes" hud-chat)
    card=$(rect_of "$nodes" hud-card)
    yes=$(rect_of "$nodes" hud-card-yes)
    no=$(rect_of "$nodes" hud-card-no)
    hint=$(rect_of "$nodes" hud-chat-hint)
    inside "$chat" "$card" || fail "${scale}x: the card ($card) is not inside the balloon ($chat)"
    inside "$card" "$yes" || fail "${scale}x: the Yes button ($yes) is not in the card ($card)"
    inside "$card" "$no" || fail "${scale}x: the No button ($no) is not in the card ($card)"
    [ "${yes##* }" = 28 ] || fail "${scale}x: the Yes button is not 28 px high: $yes"
    [ "$(jq '[.[] | select(.id == "hud-chat-hint")] | length' "$nodes")" = 1 ] || fail "${scale}x: the hint is not shown once"
    ok "${scale}x: the question card is in the balloon with Yes and No buttons, the typing hint once: $hint"
    gate_cleanup 0
    pids=()
}
for scale in ${SCALE:-1 2}; do
    question_cell "$scale"
done
