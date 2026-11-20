#!/bin/bash
# Gate of an output of tiling with no window (sd-h1, the render half of
# sh-23): the panel and the wallpaper of the tiling flavor are companions
# (waybar.sh, wallpaper.sh) that a session starts; with them, an output of
# tiling that holds no window is not black.
#
# 1. xp on HEADLESS-1, tiling on HEADLESS-2, no window anywhere: the
#    capture of HEADLESS-2 shows the bar, a click on the Apps door at
#    HEADLESS-2 starts the launcher, and the wallpaper fills the rest.
# 2. xp and tiling each in a pane of the construct, no window: the output
#    of the pane of tiling shows the bar and the wallpaper at once, with
#    no click.
#
# Usage: tests/gate-tiling-empty-output.sh [outputs|panes]   (default: both,
#   each in a session of its own)   BIN: directory with the binaries
#   SHOT  a directory that keeps the captures of both parts
#   COMPANIONS  0 starts no waybar.sh and no wallpaper.sh: the gate then
#         fails on the black output, which is what a session without the
#         companions (a bare chora-tiling) shows
# waybar, swaybg, jq and the binaries are needed.
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
case "${1:-}" in
"") "$0" outputs || exit 1; exec "$0" panes ;;
outputs | panes) ;;
*) echo "usage: $0 [outputs|panes]" >&2; exit 2 ;;
esac
. "$(dirname "$0")/doors/lib.sh"
need waybar
need swaybg
need jq
need_bins chora choractl chora-tiling chora-xp chora-construct
flavor=$repo/flavors/tiling
temp_config
doors_fakes
fuzzel_stand_in
export CHORA_FLAVOR_DIR=$flavor HOME=$fakes/home
mkdir -p "$HOME"
[ -z "${SHOT:-}" ] || mkdir -p "$SHOT"

companions() {
    [ "${COMPANIONS:-1}" != 0 ] || return 0
    setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >"$fakes/waybar.log" 2>&1 &
    run_pid $!
    setsid sh "$flavor/wallpaper.sh" >"$fakes/wallpaper.log" 2>&1 &
    run_pid $!
}
drawn() {
    ctl capture --output "$1" "$3" >"$fakes/capture.log" 2>&1 &&
        [ "$(pixel "$3" "$2" 24)" != "$(pixel "$3" "$2" 300)" ]
}
keep() { [ -z "${SHOT:-}" ] || cp "$1" "$SHOT/$2"; }
black() { [ "$(pixel "$1" 100 300)" = "#000000" ]; }

if [ "$1" = outputs ]; then
session_start "1280x800,1280x800+1280+0" -- sh -c "$bin/chora-xp --output HEADLESS-1 --theme $repo/flavors/xp/theme & $bin/chora-tiling --output HEADLESS-2 & wait"
grant screen.capture input.inject methods.call
companions
two=$fakes/two.png
until_ok 'drawn HEADLESS-2 640 "$two" && ! black "$two"' 80 ||
    { keep "$two" two-screens-tiling.png; fail "the empty output of tiling is black: pixel at the bar $(pixel "$two" 640 24), in the body $(pixel "$two" 100 300); waybar says $(tail -n 3 "$fakes/waybar.log")"; }
keep "$two" two-screens-tiling.png
unmark
echo "[{\"pointer_move\": {\"position\": [55, 24], \"output\": \"HEADLESS-2\"}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null
marked '^fuzzel' || fail "a click on Apps of the empty output starts no launcher"
ok "the empty output of tiling beside xp shows the bar and the wallpaper, and its Apps door starts the launcher"
exit 0
fi

session_start 1280x800 -- "$bin/chora-construct"
grant screen.capture input.inject methods.call
ctl wait id:construct-look >/dev/null || fail "no space: $(tail -n 5 "$GATE_ENV")"
call() { ctl call construct "$@"; }
call pane.manager '{"manager": "xp", "size": [560, 400]}' >/dev/null || fail "pane.manager xp"
call pane.manager '{"manager": "tiling", "size": [560, 400]}' >/dev/null || fail "pane.manager tiling"
companions
out=$(call construct.list | jq -r '.panes[] | select(.manager == "tiling") | .output')
[ -n "$out" ] || fail "the pane of tiling names no output"
pane=$fakes/pane.png
until_ok 'drawn "$out" 200 "$pane" && ! black "$pane"' 80 ||
    { keep "$pane" pane-tiling.png; [ -z "${SHOT:-}" ] || ctl capture "$SHOT/panes-screen.png" >/dev/null 2>&1; fail "the pane of tiling on $out is black: pixel at the bar $(pixel "$pane" 200 24), in the body $(pixel "$pane" 100 300); waybar says $(tail -n 3 "$fakes/waybar.log")"; }
keep "$pane" pane-tiling.png
[ -z "${SHOT:-}" ] || ctl capture "$SHOT/panes-screen.png" >/dev/null
ok "the pane of tiling with no window shows the bar and the wallpaper on $out with no click"
