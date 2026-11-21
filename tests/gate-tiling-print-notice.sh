#!/bin/bash
# The screenshot of tiling shows itself: the key Super+Print, and the
# row Screenshot of the power menu that the power icon of the bar opens,
# each save a file and put a notice of chora-notify at the bottom right
# that stays at least 3 s, and chora-notify lists it. chora-notify runs
# as its unit starts it, on a session bus of the gate. No stand-in for
# notify-send: the real one runs.
#
# Usage: tests/gate-tiling-print-notice.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the captures tiling-print-notice.png
#          and tiling-power-screenshot-notice.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need notify-send
need grim
need jq
need dbus-daemon
need waybar
need fuzzel
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling
home=$(mktemp -d /tmp/gate-print.XXXXXX)
tmp_dirs+=("$home")
export PATH=$bin:$PATH
export HOME=$home
export XDG_PICTURES_DIR=$home/Pictures
export LC_ALL=C
bus=$(dbus-daemon --session --print-address --fork 2>/dev/null | head -n 1 | cut -d, -f1)
[ -n "$bus" ] || fail "no session bus"
export DBUS_SESSION_BUS_ADDRESS=$bus
export CHORA_TEST_KEEP_BUS=1
unset CHORA_FLAVOR_DIR
temp_config
session_start 1280x800 -- "$bin/chora-tiling" --no-companions
CHORA_FLAVOR_DIR=$flavor setsid sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
run_pid $!
grant screen.capture input.inject methods.call
start_notify "$home/notify.log"
shot=$home/shot.png
for _ in $(seq 40); do
    sleep 0.5
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || continue
    [ "$(pixel "$shot" 640 24)" != "$(pixel "$shot" 640 300)" ] && break
done
keys() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
click() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null || fail "the click at $1"; }
empty=$(pixel "$shot" 1000 700 | cut -d' ' -f1-3)
notice_here() {
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || return 1
    [ "$(pixel "$shot" 1000 700 | cut -d' ' -f1-3)" != "$empty" ] && shown_notices | grep -q "Screenshot saved"
}
files() { find "$XDG_PICTURES_DIR/Screenshots" -name "*.png" 2>/dev/null | wc -l; }
wait_notice() {
    for _ in $(seq 24); do
        sleep 0.5
        notice_here && return 0
    done
    return 1
}
stays_3s() {
    local until=$((SECONDS + 3))
    while [ $SECONDS -lt $until ]; do
        notice_here || return 1
        sleep 0.5
    done
}
wait_gone() {
    for _ in $(seq 40); do
        notice_here || return 0
        sleep 0.5
    done
    return 1
}

keys Super+Print
wait_notice || fail "Super+Print: no notice at the bottom right, the pixel 1000,700 is $(pixel "$shot" 1000 700)"
stays_3s || fail "Super+Print: the notice was gone within 3 s"
[ "$(files)" -ge 1 ] || fail "Super+Print wrote no file"
[ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-print-notice.png"
ok "Super+Print saves a file and shows a notice for 3 s"
wait_gone || fail "the notice stays on"

rm -rf "$XDG_PICTURES_DIR/Screenshots"
click 1238 24
sleep 2
echo '[{"text": {"text": "Screenshot"}}]' | ctl inject - >/dev/null || fail "typing Screenshot"
settle
keys Return
wait_notice || fail "the power menu row Screenshot: no notice at the bottom right, the pixel 1000,700 is $(pixel "$shot" 1000 700)"
stays_3s || fail "the power menu row Screenshot: the notice was gone within 3 s"
[ "$(files)" -ge 1 ] || fail "the power menu row Screenshot wrote no file"
[ -z "${SHOTS:-}" ] || cp "$shot" "$SHOTS/tiling-power-screenshot-notice.png"
ok "a click on the power icon, the row Screenshot: a file and a notice"

windows=$(ctl query 'kind:window' 2>/dev/null | jq 'length')
[ "${windows:-0}" = 0 ] || fail "$windows windows are left"
ok "no window lost"
