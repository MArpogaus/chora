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
for what in chora choractl chora-probe; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
work=$(mktemp -d /tmp/gate-0.9-input.XXXXXX)
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

# Empty params read the settings back.
settings=$(timeout 10 python3 -I - "$CHORA_SOCKET" <<'PY'
import json, socket, sys
s = socket.socket(socket.AF_UNIX)
s.connect(sys.argv[1])
f = s.makefile("rw")
for i, (method, params) in enumerate(
    [("hello", {"name": "gate"}), ("input.configure", None)]
):
    f.write(json.dumps({"jsonrpc": "2.0", "id": i, "method": method, "params": params}) + "\n")
    f.flush()
    answer = json.loads(f.readline())
print(json.dumps(answer["result"], sort_keys=True))
PY
) || fail "input.configure: $settings"
[ "$settings" = '{"keyboard": {"repeat_delay": 300, "repeat_rate": 40}, "touchpad": {"tap": true}}' ] ||
    fail "the settings: $settings"
ok "the core holds tap for touchpads: $settings"
rm -rf "$work"
