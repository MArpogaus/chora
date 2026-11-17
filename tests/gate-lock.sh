#!/bin/bash
# The lock of the gates: one gate runs at a time on a machine. A gate
# that finds the lock held prints `NAME: waits for LOCK` and waits. A
# helper process outside the jobs of the gate holds the lock until the
# shell of the gate ends, so that no program that the gate left behind
# keeps it and the wait of the gate does not wait for it. The gate exports
# CHORA_GATE_LOCK_HELD, so that the gate scripts that it runs, also in
# parallel, do not wait for their own gate.
#
# Usage: . tests/gate-lock.sh; gate_lock NAME
#   CHORA_GATE_LOCK  the lock file (default:
#                    ${XDG_RUNTIME_DIR:-/tmp}/chora-gate.lock)
# shellcheck disable=SC2016 # the helper shell expands its own arguments
gate_lock() {
    [ -z "${CHORA_GATE_LOCK_HELD:-}" ] || return 0
    local lock=${CHORA_GATE_LOCK:-${XDG_RUNTIME_DIR:-/tmp}/chora-gate.lock}
    local ready
    ready=$(mktemp -u "${TMPDIR:-/tmp}/chora-gate-ready.XXXXXX")
    flock -n "$lock" true || echo "$1: waits for $lock"
    (
        flock "$lock" sh -c 'touch "$1" && exec tail --pid="$2" -f /dev/null' gate-lock "$ready" "$$" \
            </dev/null >/dev/null 2>&1 &
        echo "$!" >"$ready.pid"
    )
    until [ -e "$ready" ]; do
        kill -0 "$(cat "$ready.pid")" 2>/dev/null || return 1
        sleep 0.1
    done
    rm -f "$ready" "$ready.pid"
    CHORA_GATE_LOCK_HELD=$lock
    export CHORA_GATE_LOCK_HELD
}
