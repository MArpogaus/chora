#!/bin/bash
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling
need fuzzel
export PATH=$bin:$PATH
temp_config
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x720 -- "$bin/chora-tiling"
grant screen.capture input.inject
printf 'alpha\nbeta\ngamma\n' | WAYLAND_DEBUG=1 fuzzel --dmenu >/tmp/claude-0/fz.out 2>/tmp/claude-0/fz.log &
pids+=($!)
sleep 3
ctl capture /tmp/claude-0/shots/fz-before.png >/dev/null
echo '[{"pointer_move": {"position": [640, 260]}}]' | ctl inject >/dev/null
sleep 0.4
echo '[{"pointer_move": {"position": [642, 262]}}]' | ctl inject >/dev/null
sleep 0.4
echo '[{"button": {"state": "pressed", "hold": true}}]' | ctl inject >/dev/null
sleep 0.3
echo '[{"button": {"state": "released"}}]' | ctl inject >/dev/null
sleep 2
echo out: $(cat /tmp/claude-0/fz.out)
