#!/bin/bash
# Gate of sh-30: a chora-probe whose compositor goes away ends. The probe
# of `window --report` used to spin at a full core on the closed
# connection, for hours, and leave the machine loaded (F5).
#
# A headless session with tiling shows a probe window with --report; the
# session ends (the way a gate that fails or a train that is killed ends
# it). The probe must exit within 2 s, and while it runs it must not
# burn CPU.
#
# Usage: tests/gate-probe-hangup.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -u
. "$(dirname "$0")/gate-lib.sh"
. "$repo/tests/proc-lib.sh"
need_bins chora choractl chora-tiling chora-probe
temp_config
export PATH=$bin:$PATH

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
cpu_ticks() { awk '{ print $14 + $15 }' "/proc/$1/stat" 2>/dev/null; }

session_start 1280x720 -- "$bin/chora-tiling"
core=${pids[0]}
"$bin/chora-probe" window 200 200 200 --title "Probe window" --server-deco --report >"$XDG_CONFIG_HOME/probe.log" 2>&1 &
probe=$!
pids+=("$probe")
until_ok 'ctl query "app_id:chora-probe kind:surface" | jq -e "length > 0" >/dev/null 2>&1' ||
    fail "the probe window does not show: $(cat "$XDG_CONFIG_HOME/probe.log")"
running "$probe" || fail "the probe is gone before its compositor: $(cat "$XDG_CONFIG_HOME/probe.log")"

kill "$core"
for _ in $(seq 100); do
    kill -0 "$core" 2>/dev/null || break
    sleep 0.1
done
kill -0 "$core" 2>/dev/null && fail "the session does not end"
t0=$(cpu_ticks "$probe")
for _ in $(seq 20); do
    running "$probe" || break
    sleep 0.1
done
if running "$probe"; then
    sleep 1
    t1=$(cpu_ticks "$probe")
    fail "the probe still runs 2 s after its compositor went away, with $((${t1:-0} - ${t0:-0})) CPU ticks in the next second"
fi
ok "the probe ends when its compositor goes away"
