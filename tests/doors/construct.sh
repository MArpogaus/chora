#!/bin/bash
# The cells of the doors gate (tests/gate-doors.sh) for construct: the keys
# are the bindings of construct.yaml and must work in the walk and flown
# into a pane; the doors are the words of the corner strip at the top left
# (Programs, Keys, Lock, Power, Screenshot, the volume word and the clock),
# which shows in the walk only and comes back after a step out of a pane,
# and the island of the HUD host, clicked at their nodes; the key list is
# the panel construct-help, read cap by cap. The pane is the window of
# chora-probe. The check pane-corner flies into the pane and clicks where
# the door Lock stood: the window gets the press, nothing locks, and the
# strip is gone until the step out.
#
# Usage: tests/doors/construct.sh   BIN: directory with the binaries
#   ONLY  a regular expression of cell names; the others do not run
# shellcheck disable=SC2329 # doors_run calls the cells by name
set -u
. "$(dirname "$0")/lib.sh"
need jq
need_bins chora choractl chora-construct chora-voice chora-assistant chora-chat chora-probe
temp_config
doors_fakes
fuzzel_stand_in
export CHORA_FLAVOR_DIR=$repo/flavors/construct HOME=$fakes/home XDG_PICTURES_DIR=$fakes/pictures
mkdir -p "$HOME" "$XDG_CONFIG_HOME/chora"
printf 'test_accept_injected_keys: true\n' >"$XDG_CONFIG_HOME/chora/chora-construct.yaml"
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800 -- "$bin/chora-construct"
grant screen.capture input.inject methods.call
doors_voice || fail "voice does not answer"
doors_assistant || fail "the HUD host does not start"
helper_start || fail "no helper starts"
doors_window
until_ok 'ctl query id:construct-look | jq -e "length > 0" >/dev/null' 40 || fail "no space"

keys_open() { ctl query id:construct-help 2>/dev/null | jq -e '.[0].visible' >/dev/null; }
keys_close() {
    key Escape
    until_ok '! keys_open' 20
}
chat_open() { ctl query id:hud-chat 2>/dev/null | jq -e 'length > 0' >/dev/null; }
chat_close() {
    key Escape
    until_ok '! chat_open' 20
}
flown() { ctl query id:construct-look 2>/dev/null | jq -e '.[0].visible == false' >/dev/null; }
into_pane() {
    key Return
    until_ok flown 20
}
out_of_pane() {
    flown || return 0
    key Alt+Escape
    until_ok '! flown' 20
}
walking_and_flown() {
    "$@" || return 1
    into_pane || return 1
    "$@"
    local status=$?
    out_of_pane
    return "$status"
}
chords() {
    ctl query owner:construct 2>/dev/null | jq -r '
        [.[] | select(.id | test("^construct-help-body-row-[0-9]+-(keys|alt)-[0-9]+-text$"))
            | (.id | capture("row-(?<row>[0-9]+)-(?<part>keys|alt)-(?<cap>[0-9]+)-text$")) + {text}]
        | group_by([.row, .part])[]
        | sort_by(.cap | tonumber) | map(.text) | join("+")'
}
node_text() { ctl query "id:$1" 2>/dev/null | jq -r '.[0].text // empty'; }
island_word() { ctl query id:hud-state 2>/dev/null | jq -r '.[0].meta.text // empty'; }
island_says() { [ "$(island_word)" = "$1" ]; }
click_door() { until_ok "center $1 >/dev/null" 20 && click_node "$1"; }
door_says() { [ "$(node_text "$1")" = "$2" ]; }
shots() { ls "$XDG_PICTURES_DIR"/Screenshots/*.png >/dev/null 2>&1; }

check_keys() {
    key "$1"
    until_ok keys_open 20 && keys_close
}
check_helper_key() {
    key Super+a
    until_ok chat_open 20 && chat_close
}
check_launcher_key() {
    unmark
    key Super+space
    marked '^fuzzel'
}
check_lock_key() {
    unmark
    key Super+Shift+Escape
    marked '^(swaylock|loginctl lock-session)'
}
check_power_key() {
    unmark
    key Super+Shift+e
    marked '^row Shut down' && grep -qx 'row Lock' "$marks" && grep -qx 'row Log out' "$marks" && grep -qx 'row Restart' "$marks"
}
check_screenshot_key() {
    rm -rf "$XDG_PICTURES_DIR/Screenshots"
    key Super+Print
    until_ok shots 20
}
check_volume_key() {
    unmark
    key XF86AudioRaiseVolume
    marked '^wpctl set-volume' || return 1
    unmark
    key XF86AudioLowerVolume
    marked '^wpctl set-volume' || return 1
    unmark
    key XF86AudioMute
    marked '^wpctl set-mute'
}

cell_keys_key() { walking_and_flown check_keys Super+F1; }
cell_keys_alias() { walking_and_flown check_keys Super+slash; }
cell_keys_names() {
    local chord bad=0 seen
    key Super+F1
    until_ok keys_open 20 || return 1
    seen=$(chords)
    for chord in Super+F1 Super+Space Super+A Super+Shift+Esc Super+Shift+E Super+Print Vol+ Vol- Mute; do
        grep -qx -- "$chord" <<<"$seen" || bad=1
    done
    has_text "Volume up" && has_text "Volume down" || bad=1
    ! has_text "Mute the sound" || bad=1
    keys_close
    return "$bad"
}
cell_keys_door() {
    click_door construct-door-keys
    until_ok keys_open 20 && keys_close
}
cell_f1_free() {
    key F1
    sleep 0.5
    ! keys_open || return 1
    into_pane || return 1
    key F1
    got_key 59
    local status=$?
    ! keys_open || status=1
    out_of_pane
    return "$status"
}
cell_launcher_key() { walking_and_flown check_launcher_key; }
cell_launcher_door() {
    unmark
    click_door construct-door-programs
    marked '^fuzzel'
}
cell_launcher_chat() {
    unmark
    key Super+space
    marked "^data .*$repo/flavors/shared" &&
        grep -qx 'Name=Chat with a helper' "$repo/flavors/shared/applications/chora-chat.desktop"
}
cell_helper_key() { walking_and_flown check_helper_key; }
cell_helper_door() {
    [ -n "$(island_word)" ] || return 1
    click_node hud-figure
    until_ok chat_open 20 && chat_close
}
cell_helper_none() {
    voice_stop
    doors_voice none || return 1
    until_ok 'island_says "No helper"' 20 || return 1
    key Super+a
    until_ok 'has_text "No helper is set up"' 20 && chat_close
}
cell_lock_key() { walking_and_flown check_lock_key; }
cell_lock_door() {
    unmark
    click_door construct-door-lock
    marked '^swaylock'
}
cell_power_key() { walking_and_flown check_power_key; }
cell_power_door() {
    unmark
    click_door construct-door-power
    marked '^row Shut down'
}
cell_screenshot_key() { walking_and_flown check_screenshot_key; }
cell_screenshot_door() {
    rm -rf "$XDG_PICTURES_DIR/Screenshots"
    click_door construct-door-screenshot
    until_ok shots 20
}
cell_volume_key() { walking_and_flown check_volume_key; }
cell_volume_door() {
    until_ok 'door_says construct-door-volume-text "Volume 50%"' 20 || return 1
    unmark
    click_door construct-door-volume
    marked '^wpctl set-mute'
}
cell_clock_door() {
    local shown now before
    shown=$(node_text construct-door-clock-text)
    before=$(date -d '1 minute ago' +%H:%M)
    now=$(date +%H:%M)
    [[ $shown =~ ^[0-2][0-9]:[0-5][0-9]$ ]] || return 1
    [ "$shown" = "$now" ] || [ "$shown" = "$before" ] || return 1
    ctl query id:construct-door-clock | jq -e '.[0].visible' >/dev/null
}

check_pane_corner() {
    local at
    until_ok 'center construct-door-lock >/dev/null' 20 || return 1
    at=$(center construct-door-lock) || return 1
    into_pane || return 1
    unmark
    : >"$work/window"
    # shellcheck disable=SC2086 # two words, X and Y
    click $at
    until_ok "grep -q '^press' \"$work/window\"" 12
    local status=$?
    ! grep -q '^swaylock' "$marks" || status=1
    ! ctl query id:construct-doors | jq -e 'length > 0' >/dev/null || status=1
    out_of_pane
    until_ok "ctl query id:construct-door-lock | jq -e 'length > 0' >/dev/null" 20 || status=1
    return "$status"
}

doors_run construct
status=$?
if check_pane_corner; then
    echo "check construct pane-corner pass"
else
    echo "check construct pane-corner fail"
    status=1
fi
layout_audit "construct doors" >/dev/null
[ "$(ctl query kind:surface | jq length)" -ge 1 ] || fail "the window of the gate is lost"
exit "$status"
