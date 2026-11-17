#!/bin/bash
# The Keys door of the tiling bar (ti-3): a click with the mouse on the
# word Keys shows the key list, and Escape closes it. The click is an
# injected pointer click on the waybar of a headless session, at the
# word that stands left of the power icon; the gate looks for it from
# the right edge of the bar in steps of 12 px, since the width of the
# island follows its font.
#
# Usage: tests/gate-tiling-bar-keys.sh   BIN: directory with the binaries
#   SHOTS  a directory that keeps the capture tiling-bar-keys.png
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling
export PATH=$bin:$PATH
export CHORA_FLAVOR_DIR=$flavor
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
CHORA_FLAVOR_DIR=$flavor setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
run_pid $!
grant screen.capture input.inject
shot=$XDG_CONFIG_HOME/bar.png
for _ in $(seq 40); do
    sleep 0.5
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || continue
    [ "$(pixel "$shot" 640 24)" != "$(pixel "$shot" 640 300)" ] && break
done
[ "$(pixel "$shot" 640 24)" != "$(pixel "$shot" 640 300)" ] || fail "no bar"

listed() { ctl query id:tiling-keys-panel 2>/dev/null | jq -e 'length > 0' >/dev/null; }
listed && fail "the key list shows before a click"
at=
for x in $(seq 1226 -12 1130); do
    echo "[{\"pointer_move\": {\"position\": [$x, 24]}}, {\"click\": {\"button\": 272}}]" | ctl inject - >/dev/null ||
        fail "the click at $x"
    for _ in $(seq 10); do
        listed && at=$x && break 2
        sleep 0.2
    done
done
[ -n "$at" ] || fail "no click between 1130 and 1226 on the bar shows the key list"
ok "a click on Keys at x $at shows the key list"
if [ -n "${SHOTS:-}" ]; then
    sleep 1
    ctl capture --output HEADLESS-1 "$SHOTS/tiling-bar-keys.png" >/dev/null || fail "the capture"
fi
echo '[{"shortcut": {"keys": "Escape"}}]' | ctl inject >/dev/null || fail "inject Escape"
for _ in $(seq 20); do
    listed || break
    sleep 0.2
done
listed && fail "Escape keeps the key list"
ok "Escape closes the key list"
