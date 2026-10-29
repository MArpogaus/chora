#!/bin/bash
# Gate check 3 of 0.7: chora starts on the tty backend with the vkms card
# and shows a foot window under chora-tiling. A capture shows its pixels.
# The CI job `vkms` in .github/workflows/ci.yaml runs this as root, with
# vkms as the only card and seatd running. Elsewhere the answer is SKIP:
# chora on the tty backend would take the seat of the machine.
#
# Usage: tests/gate-0.7-3.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl chora-tiling; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need foot
test "$(id -u)" = 0 && test -d /sys/module/vkms || skip "needs root and the vkms module"
rt=$(mktemp -d /tmp/gate3.XXXXXX)
log=$rt/chora.log
env -u WAYLAND_DISPLAY -u DISPLAY XDG_RUNTIME_DIR="$rt" \
    "$bin/chora" -- sh -c "foot & exec '$bin/chora-tiling'" >"$log" 2>&1 &
pids+=($!)
trap 'kill "${pids[@]}" 2>/dev/null; wait' EXIT
export CHORA_SOCKET=$rt/chora/wayland-1.sock
wait_for() {
    for _ in $(seq 60); do
        "$@" >/dev/null 2>&1 && return 0
        sleep 0.5
    done
    fail "timeout: $*; $(tail -n 5 "$log")"
}
wait_for test -S "$CHORA_SOCKET"
grep -q 'the primary GPU.*card' "$log" || fail "no primary GPU in $log"
grant screen.capture
foot_up() { ctl query app_id:foot | grep -q '"foot"'; }
wait_for foot_up
# The background of foot fills the middle of the output.
shown() {
    ctl capture "$rt/shot.png" &&
        test "$(pixel "$rt/shot.png" 500 400)" != "0 0 0 255"
}
wait_for shown
ok "chora on vkms shows foot: $(pixel "$rt/shot.png" 500 400)"
