#!/bin/bash
# The cells of the doors gate (tests/gate-doors.sh) for mc: the keys are
# the bindings of mc.yaml, the doors are the words of the margin column
# (the key rows, the door of the chat and the row of doors at the foot of
# the column: Programs, Lock, Power, Screenshot and the volume word), and
# the clock is the one of the header. The HUD host draws the chat. The
# check header-SIZE asks that no two words of the first two lines of the
# screen share pixels: the header of mc, the first line of the column and
# the island of the host, over a window with the column, with all keys
# and without the column, and at the end on the page of the agenda. The
# date of the header is whole there: a weekday, a day and a month, with
# the time after it or without, and no ellipsis. A state has 6 s to settle, since mc lays a line out over
# two frames: settled is two equal readings of these words a quarter of
# a second apart. After that every reading of one second counts, and one
# with an overlap or a cut date fails the check. The copies NAME-probe,
# on which mc measures a text under a group of opacity 0, are no words
# of the screen.
#
# Usage: tests/doors/mc.sh   BIN: directory with the binaries
#   ONLY  a regular expression of cell names; the others do not run
#   SIZE  the size of the output (default 1280x800)
# shellcheck disable=SC2329 # doors_run calls the cells by name
set -u
. "$(dirname "$0")/lib.sh"
need jq
need_bins chora choractl chora-mc chora-voice chora-assistant chora-chat chora-probe
size=${SIZE:-1280x800}
temp_config
doors_fakes
fuzzel_stand_in
export CHORA_BIN=$bin HOME=$fakes/home XDG_PICTURES_DIR=$fakes/pictures
unset DBUS_SESSION_BUS_ADDRESS
mkdir -p "$HOME" "$XDG_CONFIG_HOME/chora" "$XDG_CONFIG_HOME/data/chora/huds"
printf 'theme: light\ntest_accept_injected_keys: true\nvoice_hud: host\n' >"$XDG_CONFIG_HOME/chora/chora-mc.yaml"
printf 'enabled: true\nposition: null\nhud: mc\n' >"$XDG_CONFIG_HOME/chora/chora-assistant.yaml"
cp -rL "$repo/plugins/chora-mc/hud" "$XDG_CONFIG_HOME/data/chora/huds/mc"
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start "$size" -- "$bin/chora-mc" --manage-windows
grant screen.capture input.inject methods.call
ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"
doors_voice || fail "voice does not answer"
doors_assistant || fail "the HUD host does not start"
helper_start || fail "no helper starts"
doors_window

shown='select(.visible and (.rects | length) > 0)'
texts() { ctl query "owner:mc kind:text" | jq -r ".[] | $shown | .text"; }
shows() { texts | grep -qxF -- "$1"; }
click_text() {
    local at
    at=$(ctl query "owner:mc kind:text" | jq -r --arg t "$1" \
        "[.[] | $shown | select(.text == \$t)][0].rects | to_entries[0].value | \"\\(.[0] + .[2] / 2 | floor) \\(.[1] + .[3] / 2 | floor)\"") || return 1
    # shellcheck disable=SC2086 # two words, X and Y
    click $at
}
key Super+n
until_ok 'shows "Start a program"' 80 || fail "no column: $(texts | tr '\n' '|')"

keys_open() { shows "Show overview"; }
keys_close() {
    key Super+F1
    until_ok '! keys_open' 20
}
chat_open() { shows "Esc  Close"; }
chat_close() {
    key Escape
    until_ok '! chat_open' 20
}
door_state() {
    ctl query '*' | jq -c '[.[] | select((.id | test("doors-(screenshot|volume)")) and (.owner == "mc") and (.rects | length > 0)) | {id, visible, text, r: (.rects | to_entries[0].value)}]' >&2
    ctl query kind:surface | jq -c '[.[] | {id, owner, rects}]' >&2
}
shots() { ls "$XDG_PICTURES_DIR"/Screenshots/*.png >/dev/null 2>&1; }
header_words() {
    ctl query '*' | jq -c '
        [.[] | select(.visible and (.rects | length) > 0)
            | select((.owner == "mc" and .kind == "text" and ((.text // "") | test("\\S"))
                    and (.id | endswith("-probe") | not))
                or .id == "hud-word" or .id == "hud-figure")
            | (.rects | to_entries[0].value) as $r
            | select($r[1] + $r[3] / 2 < 56) | {id, text, r: $r}]'
}
header_findings() {
    jq -r '
        . as $words
        | (range(0; $words | length) as $i | range($i + 1; $words | length) as $j
            | [$words[$i], $words[$j]]
            | select(.[0].r[0] < .[1].r[0] + .[1].r[2] and .[1].r[0] < .[0].r[0] + .[0].r[2]
                and .[0].r[1] < .[1].r[1] + .[1].r[3] and .[1].r[1] < .[0].r[1] + .[0].r[3])
            | "overlap \(.[0].id) on \(.[1].id)"),
        ([$words[] | select(.id | test("-header-(clock|date)$"))] as $dates
            | if ($dates | length) == 0 then "no date in the header" else empty end,
            ($dates[] | select(.text | test("^\\S+ [0-9]{1,2} [^\\s…]+(  [0-9]{2}:[0-9]{2})?$") | not)
                | "the date is cut: \(.text)"))'
}
header_clear() {
    local last='' now='' found _
    for _ in $(seq 24); do
        now=$(header_words) || now=''
        [ -n "$now" ] && [ "$now" = "$last" ] && break
        last=$now
        sleep 0.25
    done
    [ -n "$now" ] && [ "$now" = "$last" ] || {
        echo "the header does not settle" >&2
        return 1
    }
    for _ in $(seq 10); do
        found=$(header_words | header_findings) || found="no answer"
        [ -z "$found" ] || {
            echo "$found" >&2
            return 1
        }
        sleep 0.1
    done
}
header_step() {
    until_ok "$2" 20 && header_clear && return 0
    echo "header: $1 fails" >&2
    return 1
}
header_states() {
    local bad=0
    header_step "the column over a window" true || bad=1
    key Super+F1
    header_step "all keys" keys_open || bad=1
    keys_close
    key Super+n
    header_step "no column" '! shows "Start a program"' || bad=1
    key Super+n
    header_step "the column again" 'shows "Start a program"' || bad=1
    return "$bad"
}

cell_keys_key() {
    key Super+F1
    until_ok keys_open 20 && keys_close
}
cell_keys_alias() {
    key Super+slash
    until_ok keys_open 20 && keys_close
}
cell_keys_names() {
    local chord bad=0
    key Super+F1
    until_ok keys_open 20 || return 1
    for chord in Super+F1 Super+Space Super+a Super+Shift+Escape Super+Shift+e Super+Print Super+Alt+1-4 Vol+ Vol- Mute "Volume up" "Volume down"; do
        shows "$chord" || bad=1
    done
    ! shows "Mute or unmute" || bad=1
    keys_close
    return "$bad"
}
cell_keys_door() {
    click_text "All keys"
    until_ok keys_open 20 && keys_close
}
cell_f1_free() {
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
    click_text "Programs"
    marked '^fuzzel'
}
cell_launcher_chat() {
    unmark
    key Super+space
    marked "^data .*$repo/flavors/shared" &&
        grep -qx 'Name=Chat with a helper' "$repo/flavors/shared/applications/chora-chat.desktop"
}
cell_helper_key() {
    key Super+a
    until_ok chat_open 20 && chat_close
}
cell_helper_door() {
    shows "Idle" || return 1
    click_text "Chat with a helper ..."
    until_ok chat_open 20 && chat_close
}
cell_helper_none() {
    voice_stop
    doors_voice none || return 1
    until_ok 'shows "No helper"' 40 || return 1
    key Super+a
    until_ok says_no_helper 20 || return 1
    chat_close
}
cell_lock_key() {
    unmark
    key Super+Shift+Escape
    marked '^swaylock'
}
cell_lock_door() {
    unmark
    click_text "Lock"
    marked '^swaylock'
}
cell_power_key() {
    unmark
    key Super+Shift+e
    marked '^row Shut down' && grep -qx 'row Lock' "$marks" && grep -qx 'row Log out' "$marks" && grep -qx 'row Restart' "$marks"
}
cell_power_door() {
    unmark
    click_text "Power"
    marked '^row Shut down'
}
cell_screenshot_key() {
    rm -rf "$XDG_PICTURES_DIR/Screenshots"
    unmark
    key Super+Print
    marked '^grim' && until_ok shots 20
}
cell_screenshot_door() {
    rm -rf "$XDG_PICTURES_DIR/Screenshots"
    unmark
    until_ok 'shows "Screenshot"' 20 || {
        echo "screenshot-door: the words of mc are $(texts | tr '\n' '|')" >&2
        return 1
    }
    click_text "Screenshot"
    marked '^grim' && until_ok shots 20 && return 0
    echo "screenshot-door: no grim after the click" >&2
    door_state
    return 1
}
cell_volume_key() {
    unmark
    key XF86AudioRaiseVolume
    marked '^wpctl set-volume'
}
cell_volume_door() {
    until_ok 'shows "Volume 50%"' 20 || {
        echo "volume-door: the words of mc are $(texts | tr '\n' '|')" >&2
        return 1
    }
    unmark
    click_text "Volume 50%"
    marked '^wpctl set-mute' && return 0
    echo "volume-door: no set-mute after the click" >&2
    door_state
    return 1
}
cell_clock_door() {
    ctl query "owner:mc kind:text" |
        jq -e "[.[] | $shown | select(.id | endswith(\"-header-clock\")) | select(.text != \"\")] | length > 0" >/dev/null
}

header=pass
header_states || header=fail
doors_run mc
status=$?
key Super+Home
header_step "the page of the agenda" 'shows "1  AGENDA"' || header=fail
echo "check mc header-$size $header"
[ "$(ctl query kind:surface | jq length)" -ge 1 ] || fail "the window of the gate is lost"
exit "$status"
