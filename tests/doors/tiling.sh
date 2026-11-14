#!/bin/bash
# The cells of the doors gate (tests/gate-doors.sh) for tiling: the keys
# are the bindings of tiling.yaml, the doors are the modules of the bar
# (Apps, the helper word, the clock, the volume word, Keys and the power
# icon), clicked at their place in a bar of 1280 pixels, and the key list
# is the panel tiling-keys. waybar, jq and the binaries are needed.
#
# Usage: tests/doors/tiling.sh   BIN: directory with the binaries
#   ONLY  a regular expression of cell names; the others do not run
# shellcheck disable=SC2329 # doors_run calls the cells by name
set -u
. "$(dirname "$0")/lib.sh"
need waybar
need jq
need_bins chora choractl chora-tiling chora-voice chora-assistant chora-chat chora-probe
flavor=$repo/flavors/tiling
temp_config
doors_fakes
fuzzel_stand_in
export CHORA_FLAVOR_DIR=$flavor HOME=$fakes/home XDG_PICTURES_DIR=$fakes/pictures
mkdir -p "$HOME"
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800 -- "$bin/chora-tiling"
setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
run_pid $!
grant screen.capture input.inject methods.call
doors_voice || fail "voice does not answer"
doors_assistant || fail "the HUD host does not start"
helper_start || fail "no helper starts"
doors_window
shot=$fakes/bar.png
bar_shows() {
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 &&
        [ "$(pixel "$shot" 640 24)" != "$(pixel "$shot" 640 300)" ]
}
until_ok bar_shows 80 || fail "no bar"

bar_y=24
apps_x=55
helper_x=929
clock_x=640
volume_x=1018
keys_x=1209
power_x=1247

keys_open() { ctl query id:tiling-keys 2>/dev/null | jq -e '.[0].visible' >/dev/null; }
keys_close() {
    keys_open || return 0
    key Escape
    until_ok '! keys_open'
}
chat_open() { ctl query id:hud-chat 2>/dev/null | jq -e 'length > 0' >/dev/null; }
chat_close() {
    chat_open || return 0
    key Escape
    until_ok '! chat_open'
}
ink_at() {
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || return 1
    local x colours
    colours=$(for x in $(seq $(($1 - 30)) 2 $(($1 + 30))); do pixel "$shot" "$x" "$bar_y"; done | sort -u | wc -l)
    [ "$colours" -ge 3 ]
}
shots() { ls "$XDG_PICTURES_DIR"/Screenshots/*.png >/dev/null 2>&1; }
why() {
    echo "tiling: $*"
    return 1
}
helper_says() { [ "$(module_text helper)" = "$1" ]; }
module_text() { sh "$flavor/waybar/$1.sh" 2>/dev/null | jq -r .text; }

cell_keys_key() {
    keys_close
    key Super+F1
    until_ok keys_open && keys_close
}
cell_keys_alias() {
    keys_close
    key Super+slash
    until_ok keys_open && keys_close
}
cell_keys_names() {
    local chord bad=0
    keys_close
    key Super+F1
    until_ok keys_open || return 1
    for chord in Super+F1 Super+Space Super+a Super+Shift+Esc Super+Shift+e Super+Print Vol+ Vol- Mute "Volume up" "Volume down"; do
        has_text "$chord" || bad=1
    done
    keys_close
    return "$bad"
}
cell_keys_door() {
    keys_close
    click "$keys_x" "$bar_y"
    until_ok keys_open && keys_close
}
cell_f1_free() {
    keys_close
    key F1
    got_key 59 && ! keys_open
}
cell_launcher_key() {
    unmark
    key Super+space
    marked '^fuzzel'
}
cell_launcher_door() {
    unmark
    click "$apps_x" "$bar_y"
    marked '^fuzzel'
}
cell_launcher_chat() {
    unmark
    key Super+space
    marked "^data .*$repo/flavors/shared" &&
        grep -qx 'Name=Chat with a helper' "$repo/flavors/shared/applications/chora-chat.desktop"
}
cell_helper_key() {
    chat_close
    key Super+a
    until_ok chat_open && chat_close
}
cell_helper_door() {
    helper_says "coder · idle" || why "the helper word is $(module_text helper)" || return 1
    ink_at "$helper_x" || why "no word in the bar at $helper_x" || return 1
    chat_close
    click "$helper_x" "$bar_y"
    until_ok chat_open || why "the click opened no chat: $(tail -n 4 "$work/assistant.log" "$GATE_ENV")" || return 1
    click_node hud-chat-close
    until_ok '! chat_open' || why "Close did not close the chat"
}
cell_helper_none() {
    voice_stop
    doors_voice none || return 1
    until_ok 'helper_says "No helper"' 20 || return 1
    key Super+a
    until_ok 'has_text "No helper is set up"' 20 && chat_close
}
cell_lock_key() {
    unmark
    key Super+Shift+Escape
    marked '^(swaylock|loginctl lock-session)'
}
cell_lock_door() {
    unmark
    echo Lock >"$fakes/choice"
    click "$power_x" "$bar_y"
    marked '^swaylock'
    local status=$?
    rm -f "$fakes/choice"
    return "$status"
}
cell_power_key() {
    unmark
    key Super+Shift+e
    marked '^row Shut down' && grep -qx 'row Lock' "$marks" && grep -qx 'row Log out' "$marks" && grep -qx 'row Restart' "$marks"
}
cell_power_door() {
    unmark
    click "$power_x" "$bar_y"
    marked '^row Shut down'
}
cell_screenshot_key() {
    rm -rf "$XDG_PICTURES_DIR/Screenshots"
    key Super+Print
    until_ok shots 20
}
cell_screenshot_door() {
    grep -q '^Exec=chora-tiling --launch screenshot$' "$repo/flavors/shared/applications/chora-screenshot.desktop" || return 1
    rm -rf "$XDG_PICTURES_DIR/Screenshots"
    "$bin/chora-tiling" --launch screenshot >/dev/null 2>&1
    until_ok shots 20
}
cell_volume_key() {
    unmark
    key XF86AudioRaiseVolume
    marked '^wpctl set-volume'
}
cell_volume_door() {
    [ "$(module_text volume)" = "Volume 50%" ] || return 1
    unmark
    click "$volume_x" "$bar_y"
    marked '^wpctl set-mute'
}
cell_clock_door() { ink_at "$clock_x"; }

doors_run tiling
status=$?
layout_audit "tiling doors" >/dev/null
[ "$(ctl query kind:surface | jq length)" -ge 1 ] || fail "the window of the gate is lost"
exit "$status"
