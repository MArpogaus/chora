#!/bin/bash
# The cells of the doors gate (tests/gate-doors.sh) for xp: the keys are
# the bindings of chora-xp, the doors are the Start button and its menu,
# the notification area (helper, speaker, clock) and the dialog Turn Off
# Computer, clicked where the scene says they are. The key list is the
# Help and Support window on the page Keyboard shortcuts, and the keys
# show as key caps, so the cell reads them without regard to case. After
# the cells a check runs that Log Off of the Start menu ends the session
# through the shared power script.
#
# Usage: tests/doors/xp.sh   BIN: directory with the binaries
#   ONLY  a regular expression of cell names; the others do not run
# shellcheck disable=SC2329 # doors_run calls the cells by name
set -u
. "$(dirname "$0")/lib.sh"
need jq
need_bins chora choractl chora-xp chora-voice chora-assistant chora-chat chora-probe
temp_config
doors_fakes
fuzzel_stand_in
export HOME=$fakes/home XDG_PICTURES_DIR=$fakes/pictures
mkdir -p "$HOME"
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800 -- "$bin/chora-xp" --theme "$repo/flavors/xp/theme"
grant screen.capture input.inject methods.call
doors_voice || fail "voice does not answer"
doors_assistant || fail "the HUD host does not start"
helper_start || fail "no helper starts"
doors_window

output=HEADLESS-1
until_ok 'ctl query id:xp-start-HEADLESS-1 | jq -e "length > 0" >/dev/null' 40 || fail "no taskbar"

has_text_any_case() { ctl query '*' 2>/dev/null | grep -Fqi -- "$1"; }
node_exists() { ctl query "id:$1" 2>/dev/null | jq -e 'length > 0' >/dev/null; }
node_text() { ctl query "id:$1" 2>/dev/null | jq -r '.[0].meta.text // empty'; }
help_open() { node_exists xp-dlg-help; }
help_close() {
    key Escape
    until_ok '! help_open'
}
chat_open() { ctl query id:hud-chat 2>/dev/null | jq -e 'length > 0' >/dev/null; }
chat_close() {
    key Escape
    until_ok '! chat_open'
}
menu_open() { has_text "Turn Off Computer"; }
menu_close() {
    key Escape
    until_ok '! menu_open'
}
menu_show() {
    click 50 785
    until_ok menu_open
}
click_text() {
    local at
    at=$(ctl query kind:image 2>/dev/null |
        jq -er --arg t "$1" '[.[] | select(.meta.text == $t and (.rects | length) > 0)] | .[0].rects | to_entries[0].value | "\(.[0] + .[2] / 2 | floor) \(.[1] + .[3] / 2 | floor)"') || return 1
    # shellcheck disable=SC2086 # two words, X and Y
    click $at
}
dialog_open() { node_exists xp-session; }
dialog_cancel() {
    click_text Cancel
    until_ok '! dialog_open'
}
four_words() {
    local word bad=0
    for word in Lock "Log Off" Restart "Turn Off"; do
        has_text "$word" || bad=1
    done
    return "$bad"
}
hover() {
    echo "[{\"pointer_move\": {\"position\": [640, 300]}}, {\"pointer_move\": {\"position\": [$1, $2]}}]" | ctl inject - >/dev/null
}
hover_node() {
    local at
    at=$(center "$1") || return 1
    # shellcheck disable=SC2086 # two words, X and Y
    hover $at
}
away() { echo '[{"pointer_move": {"position": [640, 300]}}]' | ctl inject - >/dev/null; }
tip_shows() { [ -n "$(node_text xp-tooltip)" ]; }
tip_of() {
    hover_node "$1" || return 1
    until_ok tip_shows
    node_text xp-tooltip
}
shots() { ls "$XDG_PICTURES_DIR"/Screenshots/*.png >/dev/null 2>&1; }
settle() {
    away
    ! dialog_open || dialog_cancel
    ! menu_open || menu_close
    ! help_open || help_close
    ! chat_open || chat_close
}

cell_keys_key() {
    settle
    key Super+F1
    until_ok help_open && help_close
}
cell_keys_alias() {
    settle
    key Super+slash
    until_ok help_open && help_close
}
cell_keys_names() {
    settle
    local chord bad=0
    key Super+F1
    until_ok help_open || return 1
    for chord in Super+F1 Super+Space Super+a Super+Shift+Escape Super+Shift+e Super+Print XF86AudioRaiseVolume XF86AudioLowerVolume XF86AudioMute; do
        has_text_any_case "$chord" || bad=1
    done
    help_close
    return "$bad"
}
cell_keys_door() {
    settle
    menu_show || return 1
    click_text "Help and Support" || return 1
    until_ok help_open || return 1
    click_node xp-dlg-help-content-c-topic-4 || return 1
    until_ok 'has_text_any_case "Super+Shift+Escape"'
    local status=$?
    help_close
    return "$status"
}
cell_f1_free() {
    settle
    key F1
    got_key 59 && ! until_ok help_open 8
}
cell_launcher_key() {
    settle
    unmark
    key Super+space
    marked '^fuzzel'
}
cell_launcher_door() {
    settle
    unmark
    menu_show || return 1
    click_text "Run..." || return 1
    marked '^fuzzel'
}
cell_launcher_chat() {
    settle
    unmark
    key Super+space
    marked "^data .*$repo/flavors/shared" &&
        grep -qx 'Name=Chat with a helper' "$repo/flavors/shared/applications/chora-chat.desktop" || return 1
    menu_show || return 1
    has_text "Chat with a helper"
    local status=$?
    menu_close
    return "$status"
}
cell_helper_key() {
    settle
    key Super+a
    until_ok chat_open && chat_close
}
cell_helper_door() {
    settle
    local word
    word=$(tip_of "xp-helper-$output")
    away
    [ -n "$word" ] && [ "$word" != "No helper" ] || return 1
    click_node "xp-helper-$output"
    until_ok chat_open && chat_close
}
cell_helper_none() {
    settle
    voice_stop
    doors_voice none || return 1
    local word
    word=$(tip_of "xp-helper-$output")
    away
    [ "$word" = "No helper" ] || return 1
    key Super+a
    until_ok 'has_text "No helper is set up"' && chat_close
}
cell_lock_key() {
    settle
    unmark
    key Super+Shift+Escape
    marked '^(swaylock|loginctl lock-session)'
}
cell_lock_door() {
    settle
    unmark
    menu_show || return 1
    click_text "Lock" || return 1
    marked '^swaylock'
}
cell_power_key() {
    settle
    key Super+Shift+e
    until_ok dialog_open || return 1
    four_words
    local status=$?
    dialog_cancel
    return "$status"
}
cell_power_door() {
    settle
    menu_show || return 1
    click_text "Turn Off Computer" || return 1
    until_ok dialog_open || return 1
    four_words
    local status=$?
    dialog_cancel
    return "$status"
}
cell_screenshot_key() {
    settle
    rm -rf "$XDG_PICTURES_DIR/Screenshots"
    key Super+Print
    until_ok shots
}
cell_screenshot_door() {
    settle
    rm -rf "$XDG_PICTURES_DIR/Screenshots"
    menu_show || return 1
    click_text "Screenshot" || return 1
    until_ok shots
}
cell_volume_key() {
    settle
    unmark
    key XF86AudioRaiseVolume
    marked '^wpctl set-volume'
}
cell_volume_door() {
    settle
    [ "$(tip_of "xp-speaker-$output")" = "Volume 50%" ] || return 1
    unmark
    click_node "xp-speaker-$output"
    marked '^wpctl set-mute'
}
cell_clock_door() {
    node_text "xp-tray-$output" | grep -Eq '[0-9]{1,2}:[0-9]{2}'
}

check_log_off() {
    unmark
    menu_show || return 1
    click_text "Log Off" || return 1
    until_ok dialog_open || return 1
    click_node xp-session-button-1 || return 1
    marked '^loginctl terminate-'
}

doors_run xp
status=$?
if check_log_off; then
    echo "check xp log-off pass"
else
    echo "check xp log-off fail"
    status=1
fi
[ "$(ctl query kind:surface | jq length)" -ge 1 ] || fail "the window of the gate is lost"
exit "$status"
