#!/bin/bash
# Gate of the chat view (design/hud-sprint.md, step H7): one event script
# plays against each HUD folder in a headless session of 1280x720, and the
# visible lines of the chat must be the same in all of them. A look
# changes colours, fonts, shapes and place, never words, order or what a
# click does, so the lines are one fixed text.
#
# The script opens the chat with the helper Atlas on the fake voice agent
# (tests/fake-voice.py with FAKE_VOICE_ENTRIES), whose transcript holds a
# message with an ESC, a BEL and a right-to-left override. It reads the
# header and the lines of the node hud-chat, asks a question and reads the
# hint of the node hud-chat-hint, closes the chat and checks that voice saw
# chat.open and chat.close.
#
# Usage: tests/gate-chat.sh [FOLDER...]   (default: every folder that
#   chora-assistant --list-huds names)
#   FOLDER: a HUD folder by the name that chora-assistant --list-huds gives
#   BIN: directory with the binaries (default: target/debug)
#   SHOTS: directory for the logs and the lines (default: a temp dir)
set -uo pipefail

. "$(dirname "$0")/gate-lib.sh"

expected_lines() {
    cat <<'LINES'
header: Atlas · working · Write file (2/2)
You
Please write notes.txt
Atlas
I will write the file now.
Write notes.txt
Write notes.txt?
Atlas
Done.evil
hint: Typing does not answer. Press Alt+Y or Alt+N, or click Yes or No.
LINES
}

entries() {
    cat <<'JSON'
[
 {"seq": 1, "role": "user", "text": "Please write notes.txt"},
 {"seq": 2, "role": "agent", "text": "I will write\u001b]52;c;Zm9v\u0007 the file now."},
 {"seq": 3, "role": "tool", "text": "Write notes.txt"},
 {"seq": 4, "role": "question", "text": "Write notes.txt?"},
 {"seq": 5, "role": "agent", "text": "Done.‮evil"}
]
JSON
}

run_folder() {
    local folder=$1 out=$2 work
    need jq
    need foot
    need python3
    need_bins chora choractl chora-assistant chora-tiling
    temp_config
    work=$XDG_CONFIG_HOME
    mkdir -p "$work/chora" "$out"
    export PATH=$bin:$PATH
    export CHORA_BIN=$bin
    entries >"$work/entries.json"
    session_start 1280x720
    grant input.inject screen.capture methods.call
    mkfifo "$work/voice.in"
    FAKE_VOICE_ENTRIES=$work/entries.json python3 -I "$repo/tests/fake-voice.py" "$work/claims" \
        <"$work/voice.in" >"$out/voice.log" 2>&1 &
    pids+=($!)
    exec 3>"$work/voice.in"
    "$bin/chora-tiling" >"$out/tiling.log" 2>&1 &
    pids+=($!)
    ctl wait owner:tiling --timeout 20 >/dev/null || fail "$folder: tiling does not start"
    printf 'enabled: true\nposition: null\nhud: %s\nhuds: []\n' "$folder" >"$work/chora/chora-assistant.yaml"
    "$bin/chora-assistant" >"$out/assistant.log" 2>&1 &
    pids+=($!)
    emit() { echo "$1 $2" >&3; }
    ctl wait id:hud-row-chat --timeout 20 >/dev/null || fail "$folder: no Chat row"
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
    emit voice.state '{"name": "Atlas", "state": "working", "task": "Write file", "step": 2, "steps": 2, "seq": 5}'
    sleep 1
    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null || fail "$folder: chat.show failed"
    ctl wait id:hud-chat --timeout 10 >/dev/null || fail "$folder: no chat view"
    local text lines=$out/$folder.lines
    for _ in $(seq 50); do
        text=$(ctl query id:hud-chat | jq -r '.[0].meta.text // empty')
        [[ $text == *"Done.evil"* ]] && break
        sleep 0.2
    done
    {
        echo "header: $(sed -n 1p <<<"$text")"
        sed -n '2,$p' <<<"$text"
    } >"$lines"
    emit voice.state '{"name": "Atlas", "state": "question", "task": "Write file", "seq": 5}'
    emit voice.confirm '{"id": "51", "prompt": "Atlas wants to write the file notes.txt.", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true}'
    ctl wait id:hud-chat-hint --timeout 10 >/dev/null || fail "$folder: no hint while a question waits"
    echo "hint: $(ctl query id:hud-chat-hint | jq -r '.[0].meta.text // empty')" >>"$lines"
    ctl wait id:hud-card --timeout 10 >/dev/null || fail "$folder: the question shows no card"
    ctl capture "$out/$folder.png" --output HEADLESS-1 >/dev/null || fail "$folder: no capture"
    emit voice.confirm '{"id": "51", "done": true}'
    ctl call assistant chat.hide '{}' >/dev/null || fail "$folder: chat.hide failed"
    ctl wait id:hud-chat --gone --timeout 10 >/dev/null || fail "$folder: the chat stays after chat.hide"
    grep -q '"method": "chat.open", "params": {"name": "atlas"}' "$work/claims.chat" || fail "$folder: voice saw no chat.open"
    grep -q '"method": "chat.close"' "$work/claims.chat" || fail "$folder: voice saw no chat.close"
    ok "$folder: the chat opened, asked, and closed"
}

if [ "${1:-}" = --one ]; then
    run_folder "$2" "$3"
    exit $?
fi

folders=("$@")
[ ${#folders[@]} -gt 0 ] || mapfile -t folders < <("$bin/chora-assistant" --list-huds | awk '{print $1}')
[ ${#folders[@]} -gt 0 ] || fail "chora-assistant --list-huds names no folder"
out=${SHOTS:-$(mktemp -d /tmp/gate-chat.XXXXXX)}
mkdir -p "$out"
for folder in "${folders[@]}"; do
    "$0" --one "$folder" "$out" || exit 1
done
for folder in "${folders[@]}"; do
    expected_lines | diff -u - "$out/$folder.lines" >"$out/$folder.diff" ||
        fail "the lines of $folder differ from the expected lines: $(cat "$out/$folder.diff")"
done
ok "gate-chat: the same visible lines in ${#folders[@]} folder(s): ${folders[*]}"
