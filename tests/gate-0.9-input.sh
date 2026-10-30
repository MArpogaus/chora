#!/bin/bash
# Gate check 6 of 0.9: a view sets tap and key repeat, and the core
# applies them. `choractl view load` sends the `input` of the view to
# `input.configure`; a Wayland client then gets the new `repeat_info`,
# and the core answers tap in its settings. The headless backend has no
# libinput device: the unit tests of crates/chora/src/devices.rs show
# that a touchpad gets tap, also one that comes later.
#
# Usage: tests/gate-0.9-input.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-probe
temp_config
work=$XDG_CONFIG_HOME
cat >"$work/input.yaml" <<'YAML'
name: input
input:
  keyboard: {repeat_rate: 40, repeat_delay: 300}
  touchpad: {tap: true}
YAML

export CHORA_BIN=$bin
session_start 400x300
repeat=$(timeout 10 "$bin/chora-probe" repeat-info) || fail "the probe: $repeat"
[ "$repeat" = "repeat-info: 25 200" ] || fail "the default repeat: $repeat"
timeout 10 "$bin/choractl" view load "$work/input.yaml" >"$work/load.log" 2>&1 ||
    fail "view load: $(cat "$work/load.log")"
repeat=$(timeout 10 "$bin/chora-probe" repeat-info) || fail "the probe: $repeat"
[ "$repeat" = "repeat-info: 40 300" ] || fail "a client got no new repeat_info: $repeat"
ok "a view set the key repeat, and a Wayland client got repeat_info 40 300"

# view load prints the answer of input.configure: the settings of the core.
settings=$(python3 -I -c 'import json, sys; print(json.dumps(json.load(sys.stdin)["input"], sort_keys=True))' \
    <"$work/load.log") || fail "no input in the answer of view load: $(cat "$work/load.log")"
[ "$settings" = '{"keyboard": {"repeat_delay": 300, "repeat_rate": 40}, "touchpad": {"tap": true}}' ] ||
    fail "the settings: $settings"
ok "the core holds tap for touchpads: $settings"
