#!/bin/bash
# Process checks by pid for the gate scripts (sourced, not run).
# running PID: the process lives and is no zombie.
# remove_after_exit DIR PID...: waits up to 5 s until every PID has ended,
# then removes DIR; a client that exits can still write into DIR.
# live_child PARENT NAME: some child NAME of PARENT is running; prints
# its pid (a restarted child can sit beside its zombie).
running() { [ -n "$1" ] && [ "$(ps -o stat= -p "$1" | cut -c1)" != Z ] && kill -0 "$1" 2>/dev/null; }
live_child() {
    local pid
    for pid in $(pgrep -P "$1" -x "$2"); do
        running "$pid" && echo "$pid" && return 0
    done
    return 1
}
remove_after_exit() {
    local directory=$1 pid waited=0
    shift
    for pid in "$@"; do
        while running "$pid" && [ "$waited" -lt 50 ]; do
            sleep 0.1
            waited=$((waited + 1))
        done
    done
    rm -rf "$directory"
}
