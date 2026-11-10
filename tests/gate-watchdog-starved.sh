#!/bin/bash
# The frame watchdog on a starved host: chora-mc shows a scene of 300
# panels beside busy loops, four per CPU, each in a session of its own.
# The time that the core waits for a CPU is no cost of a scene, so the
# watchdog blames and disconnects nobody. With a budget of 1 us
# (CHORA_FRAME_BUDGET_US=1) every frame costs too much by itself, and
# the watchdog still blames chora-mc beside the same busy loops.
#
# Usage: tests/gate-watchdog-starved.sh   BIN: directory with the binaries
#   STARVE_S  seconds of busy loops per run (default 15)
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc
export PATH=$bin:$PATH
export CHORA_BIN=$bin
unset DBUS_SESSION_BUS_ADDRESS
seconds=${STARVE_S:-15}
work=$(mktemp -d /tmp/gate-starved.XXXXXX)
tmp_dirs+=("$work")
temp_config

mc() { ctl call mc "$@" >/dev/null 2>&1; }
panels=$(python3 -I -c 'import json
print(json.dumps([{"id": "p%d" % i, "cell": [i % 20, i // 20, 1, 1],
    "title": "T%d" % i, "lines": ["%d\tline" % i]} for i in range(300)]))')

starve() {
    local hogs=() end
    for _ in $(seq $(($(nproc) * 4))); do
        setsid bash -c 'while :; do :; done' &
        hogs+=($!)
    done
    pids+=("${hogs[@]}")
    end=$((SECONDS + seconds))
    while [ "$SECONDS" -lt "$end" ]; do
        mc scene.show '{"name": "many"}'
        sleep 2.5
        mc scene.show '{"name": "day"}'
        sleep 0.5
    done
    kill "${hogs[@]}" 2>/dev/null
}

run() {
    export CHORA_LOG=$work/$1.log
    env_file=$work/$1.env session_start 640x480 -- "$bin/chora-mc" --manage-windows
    for _ in $(seq 75); do
        mc scene.list '{}' && break
        sleep 0.2
    done
    mc scene.save "{\"name\": \"many\", \"grid\": [20, 15], \"panels\": $panels}" ||
        fail "chora-mc saves no scene of 300 panels"
    mc scene.save '{"name": "day", "grid": [1, 1], "panels": [
        {"id": "a", "cell": [0, 0, 1, 1], "title": "A", "lines": ["one"]}]}'
    starve
    kill "${pids[0]}" 2>/dev/null
    wait "${pids[0]}" 2>/dev/null
    pids=()
}
plain() { sed 's/\x1b\[[0-9;]*m//g' "$work/$1.log"; }
acts() { plain "$1" | grep -c "the watchdog \(blames\|disconnects\|drops\)"; }

run starved
[ "$(acts starved)" = 0 ] ||
    fail "the watchdog acts on a starved host: $(plain starved | grep "the watchdog" | head -n 3)"
plain starved | grep -q "the host is busy" ||
    fail "the core does not say that its frames waited for the CPU"
ok "a starved host makes the watchdog blame nobody"

CHORA_FRAME_BUDGET_US=1 run costly
plain costly | grep -q "the watchdog blames a client client=mc" ||
    fail "the watchdog blames nobody for frames over the budget: $(plain costly | tail -n 3)"
ok "frames that cost too much by themselves still blame their client"
