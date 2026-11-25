#!/bin/bash
# The helper of xp (rc 1 shared bar "the helper (mouse and keys)", POLS).
# Cells, each in a session of its own (argument: one cell, else all):
#   xp29-click      xp-29: a click on the paperclip itself opens its chat
#                   (the tray door stays), with a helper
#   xp29-none       the same click with no helper set up: the chat opens
#   xp30-helper     xp-30: the first start of a session greets once, naming
#                   how to ask and how to close (no control the balloon
#                   lacks)
#   xp30-none       the same with no helper set up: the honest "no helper"
#                   text
#   xp30-once       the second start of the same state: no greeting
#   xp30-click      a click anywhere else on the desktop takes the greeting away
#   xp30-window     a window under the balloon: no greeting until the window is gone
#   xp28-editor     xp-28: a new window starts right of the desktop icons
# BIN: directory with the binaries, SHOTS: directory for the logs.
set -u
. "$(dirname "$0")/gate-lib.sh"
cell=${1:-all}
if [ "$cell" = all ]; then
    for c in xp29-click xp29-none xp30-helper xp30-none xp30-once xp30-click xp30-window xp28-editor; do
        bash "$0" "$c" || exit 1
    done
    exit 0
fi
need jq
need_bins chora choractl chora-xp chora-assistant
shots=${SHOTS:-$(mktemp -d /tmp/gate-xp-helper.XXXXXX)}
mkdir -p "$shots"

temp_config
work=$XDG_CONFIG_HOME
export HOME=$work/home
export XDG_STATE_HOME=$work/state
export XDG_DATA_HOME=$work/data
export XDG_DATA_DIRS=$work/none
mkdir -p "$HOME" "$XDG_STATE_HOME" "$XDG_DATA_HOME" "$work/chora"
sed "s|^editor:.*|editor: foot -a notepad sh -c 'sleep 600'|" "$repo/plugins/chora-xp/xp.yaml" >"$work/chora/chora-xp.yaml"
export PATH=$bin:$PATH
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
click_at() {
    echo "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null
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
    echo "$t" | grep -qw "X" && fail "$1: the greeting names an X that the balloon does not have: $t"
    echo "$t" | grep -qi "close" || fail "$1: the greeting does not name how to close: $t"
}

case $cell in
xp29-click)
    voice
    start_assistant xp29
    ctl wait id:hud-figure --timeout 10 >/dev/null || fail "xp-29: no paperclip to click"
    click hud-figure
    ctl wait id:hud-chat --timeout 5 >/dev/null || fail "xp-29: a click on the paperclip opens no chat"
    ok "xp-29: a click on the paperclip opens the chat (helper)"
    ctl capture "$shots/xp-29-chat.png" >/dev/null
    ;;
xp29-none)
    start_assistant xp29none
    ctl wait id:hud-figure --timeout 10 >/dev/null || fail "xp-29: no paperclip to click"
    click hud-figure
    ctl wait id:hud-chat --timeout 5 >/dev/null || fail "xp-29: a click on the paperclip with no helper opens no chat"
    ok "xp-29: a click on the paperclip opens the chat (no helper)"
    ctl capture "$shots/xp-29-none.png" >/dev/null
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
xp30-click)
    start_assistant clicked
    greets xp-30
    click_at 400 300
    for _ in $(seq 20); do
        [ -z "$(balloon)" ] && break
        sleep 0.2
    done
    t=$(balloon)
    [ -z "$t" ] || fail "xp-30: a click elsewhere leaves the greeting in the balloon: $t"
    ok "xp-30: a click on the desktop takes the greeting away"
    ;;
xp30-window)
    need foot
    foot --window-size-pixels=1180x640 -T cover sleep 600 >"$shots/foot.log" 2>&1 &
    fpid=$!
    pids+=("$fpid")
    ctl wait --timeout 15 'kind:surface' >/dev/null || fail "xp-30: no window to cover the balloon"
    sleep 1.5
    read -r wx wy ww wh < <(ctl query 'kind:surface' | jq -r '.[0].rects | to_entries[0].value | map(floor) | join(" ")')
    { [ "$wx" -lt 1184 ] && [ $((wx + ww)) -gt 900 ] && [ "$wy" -lt 700 ] && [ $((wy + wh)) -gt 400 ]; } ||
        fail "xp-30: the window $wx,$wy ${ww}x$wh does not lie under the place of the balloon"
    start_assistant covered
    sleep 5
    t=$(balloon)
    [ -z "$t" ] || fail "xp-30: the paperclip greets over a window: $t"
    ctl capture "$shots/xp-30-covered.png" >/dev/null
    kill "$fpid"
    wait_balloon || fail "xp-30: the greeting never comes after the window is gone"
    ok "xp-30: no greeting over a window, it comes when the window is gone: $t"
    ;;
xp28-editor)
    need foot
    need bc
    click_at 50 785
    for _ in $(seq 40); do
        ctl query id:xp-menu | jq -e '.[0].visible == true' >/dev/null && break
        sleep 0.25
    done
    for _ in $(seq 40); do
        item=$(ctl query kind:image | jq -c '[.[] | select(.meta.text == "Notepad" and (.id | startswith("xp-item-")))] | .[0] // empty')
        [ -n "$item" ] && break
        sleep 0.25
    done
    [ -n "$item" ] || fail "xp-28: the Start menu shows no Notepad"
    read -r x y w h < <(jq -r '.rects | to_entries[0].value | map(floor) | join(" ")' <<<"$item")
    click_at $((x + w / 2)) $((y + h / 2))
    ctl wait --timeout 15 'app_id:notepad kind:surface' >/dev/null || fail "xp-28: Notepad did not start"
    prev=
    for _ in $(seq 50); do
        cur=$(ctl query 'app_id:notepad kind:surface' | jq -c '.[0].rects | to_entries[0].value // empty')
        [ -n "$cur" ] && [ "$cur" != "[0,0,0,0]" ] && [ "$cur" = "$prev" ] && break
        prev=$cur
        sleep 0.2
    done
    ctl capture "$shots/xp-28-editor.png" >/dev/null
    icons=$(ctl query kind:image | jq -r '[.[] | select(.id | test("^xp-icon-[0-9]")) | .rects | to_entries[0].value | select(.[0] < 200) | .[0] + .[2]] | max')
    left=$(ctl query 'app_id:notepad kind:surface' | jq -r '.[0].rects | to_entries[0].value | .[0]')
    frame=$(ctl query 'app_id:notepad kind:surface' | jq -c '[.[0].id, .[0].rects]')
    [ "$(echo "$left >= $icons" | bc)" = 1 ] || fail "xp-28: the editor starts at x $left, over the column of the desktop icons (it ends at x $icons): $frame"
    ok "xp-28: the editor starts at x $left, right of the icon column (x $icons)"
    ;;
esac
gate_cleanup 0
