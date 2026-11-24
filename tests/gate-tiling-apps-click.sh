#!/bin/bash
# A click on a row of the Apps list of the tiling flavor (ti-1) starts the
# program of that row, not the first one. The real fuzzel gets the
# programs Aa alpha and Ab Zebra before every row of the flavor, so that the
# list of nine rows that fuzzel shows has Zebra second whatever the flavor
# adds; the pointer moves to the Zebra row, whose y is
# computed from the rows that the launcher gives a stand-in (the helper row
# stands first, so Zebra is third) and from its index in that list (the middle of the first row and the
# row pitch of 37 px that fuzzel.ini gives are constants), and clicks, and the window of zebra must appear while the
# window of alpha, the first row, must not. fuzzel 1.11.1 takes the first
# motion of the pointer as its start (wayland.c wl_pointer_motion) and
# selects the row under the pointer from the second motion on, and a
# pointer_move that enters the surface is an enter, not a motion: the
# pointer moves three times, as a hand does. The cell runs with a fuzzel
# that comes 3 s late and a tag switch of the bar after the empty capture
# (the clock of the bar ticks every minute, a loaded host starts fuzzel
# late): the launcher is waited for in the row, not in any change of the
# screen (sh-39).
#
# Usage: tests/gate-tiling-apps-click.sh   BIN: directory with the binaries
# shellcheck disable=SC2016
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need fuzzel
need_bins chora choractl chora-tiling
temp_config
work=$XDG_CONFIG_HOME
until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
export HOME=$work/home
export XDG_DATA_HOME=$work/data
export XDG_DATA_DIRS=$work/none
export XDG_PICTURES_DIR=$HOME/Pictures
export CHORA_FLAVOR_DIR=$repo/flavors/tiling
mkdir -p "$HOME" "$XDG_DATA_HOME/applications"
entry() {
    printf '[Desktop Entry]\nType=Application\nName=%s\nExec=%s\n' "$2" "$3" >"$XDG_DATA_HOME/applications/$1.desktop"
}
entry zebra "Ab Zebra" "foot -a zebra sh -c 'sleep 600'"
entry alpha "Aa alpha" "foot -a alpha sh -c 'sleep 600'"
late_fuzzel() {
    mkdir -p "$work/lag"
    printf '#!/bin/sh\nsleep 3\nexec %s "$@"\n' "$(command -v fuzzel)" >"$work/lag/fuzzel"
    chmod +x "$work/lag/fuzzel"
}
late_fuzzel
export PATH=$work/lag:$bin:$PATH
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture input.inject

row_x=600
first_row_y=294
row_h=37
mkdir -p "$work/fake"
cat >"$work/fake/fuzzel" <<FAKE
#!/bin/sh
case " \$* " in *" --dmenu "*) cat >"$work/fake/rows" ;; esac
FAKE
chmod +x "$work/fake/fuzzel"
PATH=$work/fake:$PATH timeout 20 "$bin/chora-tiling" --launch fuzzel >/dev/null 2>&1
zebra_row=$(grep -nxF "Ab Zebra" "$work/fake/rows" | head -n 1 | cut -d: -f1)
[ -n "$zebra_row" ] || fail "no Zebra row in the list: $(tr '\n' '|' <"$work/fake/rows")"
zebra_index=$((zebra_row - 1))
[ "$zebra_index" -lt 9 ] || fail "the Zebra row is not among the nine that fuzzel shows: $zebra_index"
zebra_row_y=$((first_row_y + zebra_index * row_h))
ctl capture "$work/empty.png" >/dev/null || fail "capture of the empty desktop"
bar_changes() { ctl call tiling view_tag '{"tag": 2}' >/dev/null || fail "view_tag 2"; }
bar_changes
started=$(ctl run "$bin/chora-tiling" --launch fuzzel) || fail "the run of the launcher"
run_pid "$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")"
row_pixel() { pixel "$1" "$row_x" "$zebra_row_y"; }
until_ok 'ctl capture "$work/launcher.png" >/dev/null && [ "$(row_pixel "$work/launcher.png")" != "$(row_pixel "$work/empty.png")" ]' 80 || fail "the launcher did not show"
enter_y=$((zebra_row_y - 13))
approach_y=$((zebra_row_y - 3))
echo "[{\"pointer_move\": {\"position\": [$row_x, $enter_y]}}, {\"pointer_move\": {\"position\": [$row_x, $approach_y]}}, {\"pointer_move\": {\"position\": [$row_x, $zebra_row_y]}}, {\"click\": {\"button\": 272}}]" |
    ctl inject - >/dev/null || fail "inject the click"
ctl wait --timeout 10 'app_id:zebra kind:surface' >/dev/null || fail "a click on the Zebra row did not start zebra"
ctl query 'app_id:alpha kind:surface' | jq -e 'length == 0' >/dev/null || fail "a click on the Zebra row started alpha, the first row"
ok "a click on a row of the Apps list starts the program of that row"
