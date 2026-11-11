#!/bin/bash
# Gate of the HUD host in the look of mc (design/looks/mc-v2.md, the review
# of the pixels): chora-mc with `voice_hud: host`, the mc folder, a fake
# voice agent with a transcript and the keys Alt+y and Alt+n, and the flag
# --test-injected-press-is-real, where an injected press is an act of the
# person. In the light and in the dark theme it checks that
#
#   - the host draws no "Chat ..." row, no row of a helper and nothing over
#     a text of mc,
#   - the chat fills the box that mc announces, all four numbers, and the
#     box does not move or grow when a question comes,
#   - the card of the question is as wide as the column, in the fill of the
#     page, with 88 by 32 buttons 8 apart,
#   - a click on Yes answers and takes the card away, and Esc then closes
#     the chat, so the field has the keyboard again,
#   - Alt+y answers a second question and takes its card away,
#   - a message that is sent shows in the transcript at once.
#
# Usage: tests/gate-mc-host.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its first argument
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc chora-assistant chora-chat
need jq
need python3
export PATH=$bin:$PATH
export CHORA_BIN=$bin
unset DBUS_SESSION_BUS_ADDRESS
temp_config
work=$XDG_CONFIG_HOME
mkdir -p "$work/chora"

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
node_rect() { ctl query "id:$1" | jq -c '.[0].rects | to_entries[0].value'; }
node_exists() { ctl query "id:$1" | jq -e 'length > 0' >/dev/null; }
emit() { echo "$1 $2" >&3; }
click_node() {
    local rect=$1 x y
    x=$(jq -n --argjson r "$rect" '$r[0] + $r[2] / 2 | floor')
    y=$(jq -n --argjson r "$rect" '$r[1] + $r[3] / 2 | floor')
    echo "[{\"pointer_move\": {\"position\": [$x, $y]}}]" | ctl inject >/dev/null || fail "inject move"
    sleep 0.6
    echo '[{"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' | ctl inject >/dev/null ||
        fail "inject click"
}
ask() {
    emit voice.state '{"name": "Atlas", "state": "question", "task": "Write file", "seq": 5}'
    emit voice.confirm "{\"id\": \"$1\", \"prompt\": \"Atlas wants to write the file notes.txt.\", \"detail\": \"~/projects/notes/notes.txt (412 bytes)\", \"physical\": true, \"keys\": {\"yes\": \"Alt+y\", \"no\": \"Alt+n\"}, \"buttons\": true}"
}

cat >"$work/entries.json" <<'JSON'
[{"seq": 1, "role": "user", "text": "Tidy my notes."},
 {"seq": 2, "role": "agent", "text": "I read 14 notes and made 5 days. Moving them needs your yes."}]
JSON

run_theme() {
    local theme=$1 slot chat card page pixel yes no
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
    exec 3>&-
    printf 'theme: %s\ntest_accept_injected_keys: true\nvoice_hud: host\n' "$theme" >"$work/chora/chora-mc.yaml"
    printf 'enabled: true\nposition: null\nhud: mc\n' >"$work/chora/chora-assistant.yaml"
    export XDG_DATA_HOME=$work/data
    mkdir -p "$XDG_DATA_HOME/chora/huds"
    rm -rf "$XDG_DATA_HOME/chora/huds/mc" "$work/claims.answers"
    cp -rL "$repo/plugins/chora-mc/hud" "$XDG_DATA_HOME/chora/huds/mc"
    env_file=$(mktemp /tmp/gate-env.XXXXXX) CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800
    grant screen.capture input.inject methods.call
    rm -f "$work/voice.in"
    mkfifo "$work/voice.in"
    FAKE_VOICE_KEYS=1 FAKE_VOICE_ENTRIES=$work/entries.json python3 -I "$repo/tests/fake-voice.py" "$work/claims" \
        <"$work/voice.in" >"$work/voice.log" 2>&1 &
    pids+=($!)
    exec 3>"$work/voice.in"
    "$bin/chora-mc" >"$work/mc.log" 2>&1 &
    pids+=($!)
    ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"
    "$bin/chora-assistant" >"$work/assistant.log" 2>&1 &
    pids+=($!)
    ctl wait owner:assistant --timeout 20 >/dev/null || fail "the host does not start: $(tail -n 3 "$work/assistant.log")"
    ctl events mc.chat.slot >"$work/slots.jsonl" 2>/dev/null &
    pids+=($!)
    emit voice.state '{"state": "listening", "task": "", "said": ""}'
    emit voice.state '{"name": "Atlas", "state": "working", "task": "Tidy notes", "seq": 2}'
    until_ok 'ctl query "owner:mc kind:text" | jq -e "[.[] | select(.text == \"Atlas\")] | length > 0" >/dev/null' 40 ||
        fail "$theme: mc shows no row for the helper"
    until_ok 'node_exists hud-word' 40 || fail "$theme: the host shows no state word"
    sleep 1

    ctl query 'owner:assistant' | jq -e '[.[] | select(.id | startswith("hud-row-"))] | length == 0' >/dev/null ||
        fail "$theme: the host draws a row of its own in the column of mc"
    ok "$theme: the host draws no Chat row and no row of a helper"

    ctl query "owner:assistant kind:image" | jq -c '[.[] | select(.visible and (.rects | length) > 0) | {id, r: (.rects | to_entries[0].value)}]' >"$work/host.json"
    ctl query "owner:mc kind:text" | jq -c '[.[] | select(.visible and (.rects | length) > 0 and (.text | gsub(" "; "") != "")) | {text, r: (.rects | to_entries[0].value)}]' >"$work/mc.json"
    over=$(jq -nr --slurpfile h "$work/host.json" --slurpfile m "$work/mc.json" '
        [$h[0][] | select(.id != "hud-chat") as $a | $m[0][] as $b |
         select($a.r[0] < $b.r[0] + $b.r[2] and $b.r[0] < $a.r[0] + $a.r[2] and
                $a.r[1] < $b.r[1] + $b.r[3] and $b.r[1] < $a.r[1] + $a.r[3]) |
         "\($a.id) over \($b.text)"] | join(", ")')
    [ -z "$over" ] || fail "$theme: the host covers a text of mc: $over"
    ok "$theme: nothing of the host stands over a text of mc"

    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "$theme: Super+a does not show the chat"
    type_text 1
    key Return
    sleep 1
    slot=$(jq -c 'select(.data.rect != null) | .data.rect' "$work/slots.jsonl" | tail -n 1)
    chat=$(node_rect hud-chat)
    jq -ne --argjson s "$slot" --argjson c "$chat" '[range(4)] | all(. as $i | (($c[$i] - $s[$i]) | fabs) < 2)' >/dev/null ||
        fail "$theme: the chat $chat is not the box $slot that mc announces"
    ok "$theme: the chat fills the box of mc, $chat"

    type_text hello
    key Return
    until_ok 'ctl query id:hud-chat | jq -r ".[0].meta.text" | grep -q hello' 40 ||
        fail "$theme: the sent message does not show in the transcript"
    ok "$theme: a sent message shows in the transcript at once"

    ask 71
    until_ok 'node_exists hud-card' 40 || fail "$theme: no card for the question"
    sleep 1
    card=$(node_rect hud-card)
    [ "$(node_rect hud-chat)" = "$chat" ] || fail "$theme: the chat moved when the question came: $(node_rect hud-chat)"
    jq -ne --argjson s "$slot" --argjson c "$card" '$c[0] == $s[0] and $c[2] == $s[2] and $c[1] >= $s[1] and $c[1] + $c[3] <= $s[1] + $s[3]' >/dev/null ||
        fail "$theme: the card $card is not as wide as the column $slot"
    yes=$(node_rect hud-card-yes)
    no=$(node_rect hud-card-no)
    jq -ne --argjson y "$yes" --argjson n "$no" '$y[2] == 88 and $y[3] == 32 and $n[2] == 88 and $n[0] - $y[0] - $y[2] == 8' >/dev/null ||
        fail "$theme: the buttons are $yes and $no"
    page=$work/card-$theme.png
    ctl capture "$page" >/dev/null || fail "capture"
    pixel=$(pixel "$page" "$(jq -n --argjson c "$card" '$c[0] + $c[2] - 8')" "$(jq -n --argjson c "$card" '$c[1] + 8')")
    background=$(pixel "$page" 700 500)
    [ "${pixel% *}" = "${background% *}" ] || fail "$theme: the card is filled $pixel, the page is $background"
    ok "$theme: the card is as wide as the column, in the fill of the page, with 88 by 32 buttons 8 apart"

    click_node "$yes"
    until_ok '! node_exists hud-card' 20 || fail "$theme: the card stays after a click on Yes"
    grep -q '"yes": true' "$work/claims.answers" || fail "$theme: voice got no answer"
    key Escape
    until_ok '! node_exists hud-chat' 20 || fail "$theme: Esc does not close the chat after a click on Yes"
    ok "$theme: a click on Yes answers, the card goes, and Esc closes the chat"

    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "$theme: no chat for the second question"
    ask 72
    until_ok 'node_exists hud-card' 40 || fail "$theme: no card for the second question"
    sleep 1
    key Alt+y
    until_ok '! node_exists hud-card' 20 || fail "$theme: the card stays after Alt+y"
    grep -c '"yes": true' "$work/claims.answers" | grep -qx 2 || fail "$theme: Alt+y did not answer"
    ok "$theme: Alt+y answers and takes the card away"
    key Escape
}

run_theme light
run_theme dark
