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
# 3. flag: no gate-started companion at all (COMPANIONS=0 is the rule of
#    this part): `chora-tiling --with-companions` starts waybar.sh and
#    wallpaper.sh itself, so its output shows the bar and the wallpaper.
# 3b. bare (rhm-1): the documented start, a plain `chora-tiling --output
#    NAME` beside xp with no flag and no gate-started companion, shows the
#    bar within 5 s and its Apps door starts the launcher.
# 4. guard: the gate starts the companions first, as the units of a session
#    do; a `chora-tiling --with-companions` that follows starts no second
#    bar and no second wallpaper (one waybar.sh, one waybar, one
#    wallpaper.sh in the session). It waits for the pid file of the panel,
#    so that the guard is what it meets and not a race of two starts.
#
# Usage: tests/gate-tiling-empty-output.sh [outputs|panes|flag|guard]   (default:
#   all four, each in a session of its own)   BIN: directory with the binaries
#   SHOT  a directory that keeps the captures of both parts
#   COMPANIONS  0 starts no waybar.sh and no wallpaper.sh: the gate then
#         fails on the black output, which is what a session without the
#         companions (a bare chora-tiling) shows
# waybar, swaybg, jq and the binaries are needed.
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
case "${1:-}" in
"") "$0" outputs || exit 1; "$0" panes || exit 1; "$0" flag || exit 1; "$0" bare || exit 1; exec "$0" guard ;;
outputs | panes | flag | bare | guard) ;;
*) echo "usage: $0 [outputs|panes|flag|bare|guard]" >&2; exit 2 ;;
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

if [ "$1" = flag ]; then
session_start "1280x800,1280x800+1280+0" -- sh -c "$bin/chora-xp --output HEADLESS-1 --theme $repo/flavors/xp/theme & $bin/chora-tiling --with-companions --output HEADLESS-2 & wait"
grant screen.capture input.inject methods.call
bare=$fakes/bare.png
until_ok 'drawn HEADLESS-2 640 "$bare" && ! black "$bare"' 80 ||
    { keep "$bare" flag-tiling.png; fail "a tiling started with --with-companions is black: pixel at the bar $(pixel "$bare" 640 24), in the body $(pixel "$bare" 100 300)"; }
keep "$bare" flag-tiling.png
ok "a tiling started with --with-companions shows the bar and the wallpaper with no companion of the gate"
exit 0
fi

if [ "$1" = bare ]; then
session_start "1280x800,1280x800+1280+0" -- sh -c "$bin/chora-xp --output HEADLESS-1 --theme $repo/flavors/xp/theme & $bin/chora-tiling --output HEADLESS-2 & wait"
grant screen.capture input.inject methods.call
plain=$fakes/plain.png
until_ok 'drawn HEADLESS-2 640 "$plain" && ! black "$plain"' 20 ||
    { keep "$plain" bare-tiling.png; fail "a bare chora-tiling paints nothing within 5 s: pixel at the bar $(pixel "$plain" 640 24), in the body $(pixel "$plain" 100 300)"; }
keep "$plain" bare-tiling.png
unmark
echo "[{\"pointer_move\": {\"position\": [55, 24], \"output\": \"HEADLESS-2\"}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null
marked '^fuzzel' || fail "a click on Apps of a bare chora-tiling starts no launcher"
ok "a bare chora-tiling shows the bar within 5 s and its start door starts the launcher"
exit 0
fi

if [ "$1" = guard ]; then
session_start "1280x800,1280x800+1280+0" -- sh -c "$bin/chora-xp --output HEADLESS-1 --theme $repo/flavors/xp/theme & i=0; while [ \$i -lt 60 ] && [ ! -f \"\$XDG_RUNTIME_DIR/chora-tiling/panel.pid\" ]; do sleep 0.5; i=\$((i + 1)); done; $bin/chora-tiling --with-companions --output HEADLESS-2 & wait"
grant screen.capture input.inject methods.call
COMPANIONS=1 companions
guarded=$fakes/guarded.png
until_ok 'drawn HEADLESS-2 640 "$guarded" && ! black "$guarded"' 80 ||
    { keep "$guarded" guard-tiling.png; fail "the output of tiling is black beside gate-started companions: pixel at the bar $(pixel "$guarded" 640 24)"; }
keep "$guarded" guard-tiling.png
let_a_second_panel_start() { sleep 5; }
let_a_second_panel_start
count() { session_pids "$@" | wc -l; }
[ "$(session_pids -f "^$bin/chora-tiling --with-companions" | wc -l)" = 1 ] || fail "the tiling with --with-companions does not run"
[ "$(count -f "^sh .*waybar\.sh")" = 1 ] || fail "--with-companions beside a running panel left $(count -f "^sh .*waybar\.sh") waybar.sh in the session, not one"
[ "$(count -x waybar)" = 1 ] || fail "--with-companions beside a running panel left $(count -x waybar) waybars in the session, not one"
[ "$(count -f "^sh .*wallpaper\.sh")" = 1 ] || fail "--with-companions beside a running wallpaper left $(count -f "^sh .*wallpaper\.sh") wallpaper.sh in the session, not one"
ok "a tiling started with --with-companions beside running companions starts no second bar and no second wallpaper"
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
