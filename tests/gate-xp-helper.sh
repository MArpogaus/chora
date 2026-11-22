#!/bin/bash
# The helper of xp (rc 1 shared bar "the helper (mouse and keys)", POLS).
# Cells, each in a session of its own (argument: one cell, else all):
#   xp29-click      xp-29: a click on the paperclip itself opens its chat
#                   (the tray door stays), with a helper and with none
#   xp30-helper     xp-30: the first start of a session greets once, naming
#                   how to ask and how to close
#   xp30-none       the same with no helper set up: the honest "no helper"
#                   text
#   xp30-once       the second start of the same state: no greeting
# BIN: directory with the binaries, SHOTS: directory for the logs.
set -u
. "$(dirname "$0")/gate-lib.sh"
cell=${1:-all}
if [ "$cell" = all ]; then
    for c in xp29-click xp30-helper xp30-none xp30-once; do
        bash "$0" "$c" || exit 1
    done
    exit 0
fi
need jq
need_bins chora choractl chora-xp chora-assistant
shots=${SHOTS:-$(mktemp -d /tmp/gate-xp-helper.XXXXXX)}
mkdir -p "$shots"

temp_config
export HOME=$XDG_CONFIG_HOME/home
export XDG_STATE_HOME=$XDG_CONFIG_HOME/state
mkdir -p "$HOME" "$XDG_STATE_HOME"
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800 -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
grant screen.capture
grant input.inject

voice() {
    local entries=$shots/entries.json
    echo '[{"seq": 1, "role": "agent", "text": "Hello."}]' >"$entries"
    mkfifo "$shots/voice.in"
    FAKE_VOICE_ENTRIES=$entries python3 -I "$repo/tests/fake-voice.py" "$shots/claims" \
        <"$shots/voice.in" >"$shots/voice.log" 2>&1 &
    pids+=($!)
    exec 5>"$shots/voice.in"
    echo 'voice.state {"state": "listening"}' >&5
}
start_assistant() {
    "$bin/chora-assistant" >"$shots/assistant.$1.log" 2>&1 &
    apid=$!
    pids+=("$apid")
    ctl wait id:hud-figure --timeout 20 >/dev/null || fail "$1: no paperclip: $(tail -n 3 "$shots/assistant.$1.log")"
}
stop_assistant() {
    kill "$apid"
    wait "$apid" 2>/dev/null
    for _ in $(seq 100); do
        [ "$(ctl query id:hud-figure | jq length)" = 0 ] && return 0
        sleep 0.2
    done
    fail "the nodes of the stopped paperclip stay"
}
click() {
    echo "[{\"pointer_move\": {\"node\": \"$1\", \"point\": [10, 10]}}, {\"button\": {\"state\": \"pressed\"}}, {\"button\": {\"state\": \"released\"}}]" | ctl inject >/dev/null
}
balloon() { ctl query id:hud-line | jq -r '.[0] | select(.visible != false) | (.meta.text // .props.text // "")'; }
wait_balloon() {
    for _ in $(seq 50); do
        t=$(balloon)
        [ -z "$t" ] || return 0
        sleep 0.2
    done
    return 1
}
greets() {
    wait_balloon || fail "$1: the paperclip greets with no balloon at the first start: $(ctl query id:hud-line | jq -c .) $(tail -n 3 "$shots"/assistant.*.log)"
    echo "$t" | grep -qi "ask" || fail "$1: the greeting does not name how to ask: $t"
    echo "$t" | grep -qi "close" || fail "$1: the greeting does not name how to close: $t"
}

case $cell in
xp29-click)
    voice
    start_assistant xp29
    sleep 1
    click hud-figure
    ctl wait id:hud-chat --timeout 5 >/dev/null || fail "xp-29: a click on the paperclip opens no chat"
    ok "xp-29: a click on the paperclip opens the chat (helper)"
    ctl capture "$shots/xp-29-chat.png" >/dev/null
    ;;
xp30-helper)
    voice
    start_assistant xp30
    greets xp-30
    echo "$t" | grep -qi "no helper" && fail "xp-30: a helper listens, the greeting says none: $t"
    ctl capture "$shots/xp-30-helper.png" >/dev/null
    ok "xp-30: the first start greets once: $t"
    ;;
xp30-none)
    start_assistant xp30none
    greets xp-30
    echo "$t" | grep -qi "no helper" || fail "xp-30: no helper is set up, the greeting does not say so: $t"
    ctl capture "$shots/xp-30-none.png" >/dev/null
    ok "xp-30: no helper: the honest greeting: $t"
    ;;
xp30-once)
    start_assistant first
    greets xp-30
    stop_assistant
    start_assistant second
    sleep 6
    t=$(balloon)
    [ -z "$t" ] || fail "xp-30: the second start of the same state greets again: $t"
    ok "xp-30: the second start is quiet"
    ;;
esac
gate_cleanup 0
