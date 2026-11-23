#!/bin/bash
# The notice of a screenshot in the tiling flavor (ti-4): with chora-notify
# as the unit chora-tiling-notify starts it, the screenshot of the flavor
# (chora-tiling --launch screenshot) saves a file, chora-notify lists a
# notice that names it (`state.get` of notify), and the card stands at the
# bottom right: the pixel there differs from the one before. Both run on
# one session bus of their own, in a home of the gate.
# A launch whose script is missing ends with failure and chora-notify shows
# the reason (`chora-tiling --launch power` from a copy of the binary that
# finds no flavor folder). chora-notify shows one card at a time and the
# card of the screenshot shows 10 s while the failure notice waits in the
# queue (`waiting` of state.get), so the gate dismisses that card first.
#
# Usage: tests/gate-tiling-notice.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the capture tiling-screenshot-notice.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need notify-send
need grim
need jq
need dbus-run-session
need_bins chora choractl chora-tiling
export PATH=$bin:$PATH
export CHORA_FLAVOR_DIR=$repo/flavors/tiling
temp_config
home=$XDG_CONFIG_HOME/home
mkdir -p "$home"
session_start 1280x800 -- "$bin/chora-tiling" --no-companions
grant screen.capture methods.call
shot=$XDG_CONFIG_HOME/notice.png
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
before=$(pixel "$shot" 1000 700)

HOME=$home XDG_PICTURES_DIR=$home/Pictures LC_ALL=C setsid dbus-run-session -- sh -c '
    chora-notify &
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
    shown_notices | grep -q "Screenshot saved.*$(basename "$file")" || continue
    ctl capture --output HEADLESS-1 "$shot" >/dev/null || continue
    [ "$(pixel "$shot" 1000 700)" != "$before" ] && seen=1 && break
done
[ -n "$seen" ] || fail "no notice at the bottom right: the pixel 1000,700 is $(pixel "$shot" 1000 700), before $before; $(tail -n 3 "$XDG_CONFIG_HOME/notice.log")"
[ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-screenshot-notice.png"
ok "a notice of chora-notify names the file and stands at the bottom right after the screenshot"

dismiss_notices
lone=$(mktemp -d /tmp/gate-launch.XXXXXX)
tmp_dirs+=("$lone")
cp "$bin/chora-tiling" "$lone/"
if LC_ALL=C CHORA_FLAVOR_DIR=$lone/none CHORA_SHARED_DIR=$lone/none "$lone/chora-tiling" --launch power 2>"$lone/err"; then
    fail "a launch of a missing script ended with success"
fi
for _ in $(seq 10); do
    shown_notices | grep -q "Power menu not installed" && break
    sleep 0.5
done
shown_notices | grep -q "Power menu not installed" || fail "no notice of chora-notify for the missing script: $(cat "$lone/err"); $(shown_notices)"
ok "a launch that fails ends with failure and chora-notify shows the reason"
