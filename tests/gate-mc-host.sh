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
click_at_move() {
    local x y
    x=$(jq -n --argjson r "$1" '$r[0] + $r[2] / 2 | floor')
    y=$(jq -n --argjson r "$1" '$r[1] + $r[3] / 2 | floor')
    echo "[{\"pointer_move\": {\"position\": [$x, $y]}}]" | ctl inject >/dev/null || fail "inject move"
}
host_mark() {
    ctl query "owner:assistant kind:rect" | jq -e --arg id "$1-mark" '[.[] | select(.id == $id and .opacity == 1)] | length == 1' >/dev/null
}
hover_marks() {
    until_ok "node_exists $1" 20 || fail "$theme: no $1"
    click_at_move "$(node_rect "$1")"
    until_ok "host_mark $1" 20 || fail "$theme: no hover mark on $1"
}
mark_of() {
    ctl query "owner:mc kind:rect" | jq -ec --arg s "$1-mark" '[.[] | select((.id | endswith($s)) and .opacity == 1 and (.rects | length) > 0)][0].rects | to_entries[0].value'
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

    figure=$(node_rect hud-figure)
    ctl capture "$work/state-$theme.png" >/dev/null || fail "capture of the state"
    dot=$(pixel "$work/state-$theme.png" "$(jq -n --argjson f "$figure" '$f[0] + $f[2] / 2 | floor')" "$(jq -n --argjson f "$figure" '$f[1] + $f[3] / 2 | floor')")
    case $theme in
    light) want="140 36 31" ;;
    *) want="204 120 102" ;;
    esac
    [ "${dot% *}" = "$want" ] || fail "$theme: the state dot is ${dot% *}, not the accent of the theme $want"
    ok "$theme: the state dot has the accent of the theme"

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
    sleep 2
    hover_marks hud-chat-pick-1
    click_at_move "[400, 400, 1, 1]"
    until_ok '! host_mark hud-chat-pick-1' 20 || fail "$theme: the mark of the list row stays after the pointer left"
    jq -ne --argjson r "$(node_rect hud-chat-pick-1)" '$r[3] == 28 and $r[1] % 28 == 0' >/dev/null ||
        fail "$theme: the row of the list $(node_rect hud-chat-pick-1) is not a line of the grid of 28"
    ok "$theme: a row of the list is a line of the grid, shows a hover mark and loses it"
    type_text 1
    key Return
    sleep 1
    hover_marks hud-chat-stop
    ok "$theme: Stop shows a hover mark"
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
    ctl query id:hud-card | jq -r '.[0].meta.text' | head -n 1 | grep -qx "Question from Atlas" ||
        fail "$theme: the card does not name the helper: $(ctl query id:hud-card | jq -r '.[0].meta.text' | head -n 1)"
    yes=$(node_rect hud-card-yes)
    no=$(node_rect hud-card-no)
    hover_marks hud-card-yes
    hover_marks hud-card-no
    ok "$theme: Yes and No show a hover mark"
    jq -ne --argjson y "$yes" --argjson n "$no" '$y[2] == 88 and $y[3] == 32 and $n[2] == 88 and $n[0] - $y[0] - $y[2] == 8' >/dev/null ||
        fail "$theme: the buttons are $yes and $no"
    page=$work/card-$theme.png
    ctl capture "$page" >/dev/null || fail "capture"
    pixel=$(pixel "$page" "$(jq -n --argjson c "$card" '$c[0] + $c[2] - 8')" "$(jq -n --argjson c "$card" '$c[1] + 8')")
    background=$(pixel "$page" 700 500)
    [ "${pixel% *}" = "${background% *}" ] || fail "$theme: the card is filled $pixel, the page is $background"
    ok "$theme: the card is as wide as the column, in the fill of the page, with 88 by 32 buttons 8 apart"

    hint_clear() {
        jq -ne --argjson c "$(node_rect hud-card)" --argjson h "$(node_rect hud-chat-hint)" '$c[1] + $c[3] <= $h[1]' >/dev/null
    }
    hint_clear || fail "$theme: the card $(node_rect hud-card) covers the hint $(node_rect hud-chat-hint)"
    for _ in $(seq 55); do key a; done
    until_ok 'hint_clear' 20 || fail "$theme: with a field of two lines the card $(node_rect hud-card) covers the hint $(node_rect hud-chat-hint)"
    ok "$theme: the card stands above the hint, also when the field grows to two lines"
    sleep 1
    yes=$(node_rect hud-card-yes)

    click_node "$yes"
    until_ok '! node_exists hud-card' 20 || fail "$theme: the card stays after a click on Yes"
    grep -q '"yes": true' "$work/claims.answers" || fail "$theme: voice got no answer"
    until_ok '! node_exists hud-chat-hint' 20 || fail "$theme: the hint about a question stays after the card went: $(ctl query id:hud-chat-hint | jq -r '.[0].meta.text')"
    close=$(ctl query "owner:mc kind:text" | jq -c '[.[] | select((.id | endswith("chat-close")) and (.rects | length) > 0)][0].rects | to_entries[0].value')
    click_at_move "$close"
    until_ok 'mark_of chat-close' 20 || fail "$theme: no hover mark on Esc Close"
    jq -ne --argjson m "$(mark_of chat-close)" --argjson c "$close" '$m[0] < $c[0] and $m[0] > $c[0] - 30 and $m[2] == 2 and $m[3] == 20' >/dev/null ||
        fail "$theme: the mark $(mark_of chat-close) does not stand beside Esc Close $close"
    ok "$theme: Esc Close shows the hover mark beside it"
    sleep 1
    key Escape
    until_ok '! node_exists hud-chat' 20 || fail "$theme: Esc does not close the chat after a click on Yes"
    ok "$theme: a click on Yes answers, the card goes, and Esc closes the chat"

    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "$theme: no chat for the second question"
    until_ok 'node_exists hud-chat-stop' 20 || fail "$theme: Super+a after Esc does not reopen the chat of the one helper"
    node_exists hud-chat-pick-1 && fail "$theme: Super+a after Esc shows the list of helpers again"
    ok "$theme: Super+a after Esc reopens the chat of the one helper"
    ask 72
    until_ok 'node_exists hud-card' 40 || fail "$theme: no card for the second question"
    sleep 1
    key Alt+y
    until_ok '! node_exists hud-card' 20 || fail "$theme: the card stays after Alt+y"
    grep -c '"yes": true' "$work/claims.answers" | grep -qx 2 || fail "$theme: Alt+y did not answer"
    ok "$theme: Alt+y answers and takes the card away"
    node_exists hud-chat-stop || fail "$theme: no Stop while the helper works or asks"
    emit voice.state '{"name": "Atlas", "state": "idle", "task": "", "seq": 9}'
    until_ok '! node_exists hud-chat-stop' 20 || fail "$theme: Stop stays while the helper is idle"
    ok "$theme: Stop shows only while the helper runs a prompt"
    key Escape
}

for theme in ${GATE_THEMES:-light dark}; do
    run_theme "$theme"
done
