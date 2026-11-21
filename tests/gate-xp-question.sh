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
    echo 'voice.state {"name": "Atlas", "state": "working", "task": "Write file", "step": 1, "steps": 1, "seq": 2}' >&5
    sleep 1.5
    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null || fail "chat.show failed"
    echo 'voice.confirm {"id": "51", "prompt": "Atlas wants to write the file notes.txt.", "detail": "~/projects/notes/notes.txt (412 bytes)", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true}' >&5
    local nodes=$shots/nodes-$scale.json polls=0 cards hints
    while [ "$polls" -lt 100 ]; do
        sleep 0.1
        polls=$((polls + 1))
        ctl query owner:assistant >"$nodes"
        cards=$(jq '[.[] | select(.id == "hud-card")] | length' "$nodes")
        hints=$(jq '[.[] | select(.id == "hud-chat-hint")] | length' "$nodes")
        [ "$cards" = 0 ] && continue
        [ "$hints" != 0 ] || fail "${scale}x: the card showed without the hint (poll $polls): $(jq -c "[.[].id]" "$nodes")"
        break
    done
    [ "$cards" != 0 ] || fail "${scale}x: no card within 10 s"
    echo 'voice.state {"name": "Atlas", "state": "question", "task": "Write file", "step": 1, "steps": 1, "seq": 2}' >&5
    sleep 1
    ctl capture "$shots/xp-question-${scale}x.png" >/dev/null || fail "the capture failed"
    ctl query owner:assistant >"$nodes"
    exec 5>&-
    local chat card yes no hint
    chat=$(rect_of "$nodes" hud-chat)
    card=$(rect_of "$nodes" hud-card)
    yes=$(rect_of "$nodes" hud-card-yes)
    no=$(rect_of "$nodes" hud-card-no)
    hint=$(rect_of "$nodes" hud-chat-hint)
    inside "$chat" "$card" || fail "${scale}x: the card ($card) is not inside the balloon ($chat)"
    inside "$card" "$yes" || fail "${scale}x: the Yes button ($yes) is not in the card ($card)"
    inside "$card" "$no" || fail "${scale}x: the No button ($no) is not in the card ($card)"
    [ "${yes##* }" = 28 ] || fail "${scale}x: the Yes button is not 28 px high: $yes"
    local card_png chat_png cx cy cw ch bx by hy yy
    read -r card_png chat_png <<<"$(jq -r '[(.[] | select(.id == "hud-card").image), (.[] | select(.id == "hud-chat").image)] | join(" ")' "$nodes")"
    read -r cx cy cw ch <<<"$card"
    read -r bx by _ _ <<<"$chat"
    read -r _ hy _ _ <<<"$hint"
    read -r _ yy _ _ <<<"$yes"
    [ $((cy + ch)) -le "$hy" ] || fail "${scale}x: the card ($card) reaches the typing hint ($hint)"
    python3 -I - "$card_png" "$chat_png" "$scale" "$((cx - bx))" "$((cy - by))" "$cw" "$ch" "$((yy - cy))" <<'PY' || fail "${scale}x: the card is not under the transcript or not set at 14 px"
import sys
from PIL import Image
card, chat, scale, x, y, w, h, buttons = Image.open(sys.argv[1]).convert("RGBA"), Image.open(sys.argv[2]).convert("RGBA"), *map(int, sys.argv[3:])
x, y, w, h = x * scale, y * scale, w * scale, h * scale
under = chat.crop((x, y, x + w, y + h)).getcolors(1 << 20)
assert len(under) == 1, f"the transcript runs under the card: {len(under)} colors there"
bands, start = [], None
for row in range(card.height):
    ink = any(sum(card.getpixel((col, row))[:3]) < 330 and card.getpixel((col, row))[3] > 200 for col in range(card.width))
    if ink and start is None:
        start = row
    if not ink and start is not None:
        bands.append(row - start)
        start = None
lines = [b for b in bands if b > 3 * scale]
assert len(lines) >= 5, f"the card has {len(lines)} lines of text: {bands}"
assert min(lines) >= 9 * scale, f"a line of the card is under 14 px: {bands}"
PY
    [ "$(jq '[.[] | select(.id == "hud-chat-hint")] | length' "$nodes")" = 1 ] || fail "${scale}x: the hint is not shown once"
    ok "${scale}x: the question card is in the balloon with Yes and No buttons, the typing hint once: $hint"
    gate_cleanup 0
    pids=()
}
for scale in ${SCALE:-1 2}; do
    question_cell "$scale"
done
