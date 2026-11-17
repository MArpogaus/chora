#!/bin/bash
# The notice of a screenshot in the tiling flavor (ti-4): with mako as
# the unit chora-tiling-mako starts it (chora-tiling --launch mako), the
# screenshot of the flavor (chora-tiling --launch screenshot) saves a
# file, and a notice that names it stands at the bottom right, in the
# surface colour of Ink. Both run on one session bus of their own, in a
# home of the gate.
#
# Usage: tests/gate-tiling-notice.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the capture tiling-screenshot-notice.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need mako
need notify-send
need grim
need dbus-run-session
need_bins chora choractl chora-tiling
export PATH=$bin:$PATH
export CHORA_FLAVOR_DIR=$repo/flavors/tiling
temp_config
home=$XDG_CONFIG_HOME/home
mkdir -p "$home"
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture
shot=$XDG_CONFIG_HOME/notice.png
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
before=$(pixel "$shot" 1000 700)

HOME=$home XDG_PICTURES_DIR=$home/Pictures LC_ALL=C setsid dbus-run-session -- sh -c '
    chora-tiling --launch mako &
    sleep 2
    chora-tiling --launch screenshot
    sleep 30' >"$XDG_CONFIG_HOME/notice.log" 2>&1 &
run_pid $!
for _ in $(seq 40); do
    file=$(find "$home/Pictures/Screenshots" -name '*.png' 2>/dev/null | head -n 1)
    [ -n "$file" ] && break
    sleep 0.5
done
[ -n "$file" ] || fail "the screenshot saved no file: $(tail -n 3 "$XDG_CONFIG_HOME/notice.log")"
ok "the screenshot is saved as $(basename "$file")"

seen=
for _ in $(seq 20); do
    sleep 0.5
    ctl capture --output HEADLESS-1 "$shot" >/dev/null || continue
    read -r r g b _ < <(pixel "$shot" 1000 700)
    [ "$r $g $b" = "26 29 39" ] && seen=1 && break
done
[ -n "$seen" ] || fail "no notice at the bottom right: the pixel 1000,700 is $(pixel "$shot" 1000 700), before $before; $(tail -n 3 "$XDG_CONFIG_HOME/notice.log")"
[ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-screenshot-notice.png"
ok "a notice of mako stands at the bottom right after the screenshot"
