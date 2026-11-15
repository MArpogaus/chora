#!/bin/bash
# A click on a row of the Apps list of the tiling flavor (ti-1) starts the
# program of that row, not the first one. The real fuzzel gets the
# programs alpha, Screenshot and Zebra sorted by name; the pointer moves to
# the Zebra row and clicks, and the window of zebra must appear while the
# window of alpha, the first row, must not.
#
# Usage: tests/gate-tiling-apps-click.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need fuzzel
need_bins chora choractl chora-tiling
temp_config
work=$XDG_CONFIG_HOME
export HOME=$work/home
export XDG_DATA_HOME=$work/data
export XDG_DATA_DIRS=$work/none
export XDG_PICTURES_DIR=$HOME/Pictures
export CHORA_FLAVOR_DIR=$repo/flavors/tiling
mkdir -p "$HOME" "$XDG_DATA_HOME/applications"
entry() {
    printf '[Desktop Entry]\nType=Application\nName=%s\nExec=%s\n' "$2" "$3" >"$XDG_DATA_HOME/applications/$1.desktop"
}
entry zebra Zebra "foot -a zebra sh -c 'sleep 600'"
entry alpha alpha "foot -a alpha sh -c 'sleep 600'"
export PATH=$bin:$PATH
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture input.inject

row_x=600
zebra_row_y=496
started=$(ctl run "$bin/chora-tiling" --launch fuzzel) || fail "the run of the launcher"
run_pid "$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")"
sleep 3
echo "[{\"pointer_move\": {\"position\": [$row_x, $zebra_row_y]}}, {\"click\": {\"button\": 272}}]" |
    ctl inject - >/dev/null || fail "inject the click"
ctl wait --timeout 10 'app_id:zebra kind:surface' >/dev/null || fail "a click on the Zebra row did not start zebra"
sleep 1
ctl query 'app_id:alpha kind:surface' | jq -e 'length == 0' >/dev/null || fail "a click on the Zebra row started alpha, the first row"
ok "a click on a row of the Apps list starts the program of that row"
