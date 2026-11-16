#!/bin/bash
# Gate of the sofa and kitchen scenarios of chora-mc (journey 1 of the rc 11
# review, mc-32 to mc-37): chora-mc --manage-windows with the HUD host on
# a headless output of 1280x720, as the tour starts it, in one theme.
#
#   remote      after a helper changed the content of the pages and with no
#               click, Right and Left of the remote turn the page and Escape
#               goes back one step (mc-32)
#
#   wide        Super+n hides the column and the page takes its room; Super+n
#               gives it back (mc-37)
#
# Usage: tests/gate-mc-sofa.sh   BIN: directory with the binaries
#   THEME  light or dark (default dark)
#   CELLS  a regular expression of the cells that run (default all)
#   SHOTS  a directory that keeps one capture of each cell as
#          mc-sofa-CELL-THEME.png
# shellcheck disable=SC2016 # until_ok evaluates its first argument
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc chora-assistant
need jq
export PATH=$bin:$PATH
export CHORA_BIN=$bin
export LANG=C.UTF-8 LC_ALL=C.UTF-8
unset DBUS_SESSION_BUS_ADDRESS
theme=${THEME:-dark}
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
move_to() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}]" | ctl inject >/dev/null || fail "inject move"; }
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
shown='select(.visible and (.rects | length) > 0)'
texts() { ctl query "owner:mc kind:text" | jq -r ".[] | $shown | .text"; }
has_text() { texts | grep -qxF "$1"; }
current() { ctl call mc scene.list '{}' | jq -r '.current'; }
shot() {
    [ -n "${SHOTS:-}" ] || return 0
    mkdir -p "$SHOTS"
    sleep 1.2
    ctl capture "$SHOTS/mc-sofa-$1-$theme.png" >/dev/null || fail "capture $1"
}

printf 'theme: %s\ntest_accept_injected_keys: true\n' "$theme" >"$work/chora/chora-mc.yaml"
cat >"$work/entries.json" <<'JSON'
[{"seq": 1, "role": "user", "text": "Please tidy the notes for Friday."},
 {"seq": 2, "role": "agent", "text": "I will go through your notes and group them by day."}]
JSON
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x720 -- "$bin/chora-mc" --manage-windows
grant screen.capture input.inject methods.call
mkfifo "$work/voice.in"
FAKE_VOICE_KEYS=1 FAKE_VOICE_ENTRIES=$work/entries.json python3 -I "$repo/tests/fake-voice.py" "$work/claims" \
    <"$work/voice.in" >"$work/voice.log" 2>&1 &
pids+=($!)
exec 3>"$work/voice.in"
ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"
"$bin/chora-assistant" >"$work/assistant.log" 2>&1 &
pids+=($!)
ctl wait owner:assistant --timeout 20 >/dev/null || fail "the host does not start: $(tail -n 3 "$work/assistant.log")"
until_ok 'has_text "AGENDA"' || fail "no first page"
sleep 5
panel() { ctl call mc panel.set "{\"scene\": \"$1\", \"panel\": \"$2\", \"lines\": $3}" >/dev/null || fail "panel.set $1 $2"; }
panel agenda today '["18:30\tDinner with the family", "19:30\tNews on the television", "20:00\tCall Anna"]'
panel morning lead '["Good evening", "Anna will call at eight."]'
panel morning weather '["Tomorrow", "Zurich, 11 degrees, dry."]'
panel morning mail '["Post", "One card from Anna."]'
sleep 2

cell_remote() {
    move_to 640 360
    sleep 1
    key Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "Right does not turn to page 2: $(current)"
    key Right
    until_ok '[ "$(current)" = agents ]' 20 || fail "a second Right does not turn to page 3: $(current)"
    key Left
    until_ok '[ "$(current)" = morning ]' 20 || fail "Left does not turn back to page 2: $(current)"
    key Escape
    until_ok '[ "$(current)" = agents ]' 20 || fail "Escape does not go back one step: $(current)"
    shot remote
}

lead_width() {
    ctl query "owner:mc kind:text" | jq -r --arg t "$1" \
        '[.[] | select(.visible and (.rects | length) > 0 and (.text | startswith($t)))][0].rects | to_entries[0].value[2]'
}
cell_wide() {
    local before after
    key Super+Home
    key Super+Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "no page 2: $(current)"
    panel morning lead '["Boil the water", "Fill the large pot with two litres of water, add a spoon of salt and bring it to a rolling boil, then add the spaghetti and cook it for nine minutes."]'
    sleep 1
    before=$(lead_width "Boil the water")
    shot wide-column
    key Super+n
    until_ok '! has_text "Back one step"' 20 || fail "Super+n does not hide the column"
    sleep 1.5
    after=$(lead_width "Boil the water")
    shot wide-page
    [ "$(jq -n "$after > $before + 100")" = true ] || fail "the page does not widen when the column hides: $before then $after"
    key Super+n
    until_ok 'has_text "Back one step"' 20 || fail "Super+n does not bring the column back"
    sleep 1.5
    [ "$(lead_width "Boil the water")" = "$before" ] || fail "the page does not narrow when the column is back: $(lead_width "Boil the water") not $before"
}

failed=()
run_cell() {
    local name=$1
    [[ $name =~ ${CELLS:-.} ]] || return 0
    if ("cell_${name//-/_}"); then
        ok "$name ($theme)"
    else
        failed+=("$name")
    fi
}
cells=(remote wide)
for name in "${cells[@]}"; do
    run_cell "$name"
done
[ ${#failed[@]} = 0 ] || fail "cells failed: ${failed[*]}"
ok "every cell of the mc sofa gate ($theme)"
