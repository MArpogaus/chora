#!/bin/bash
# The core's last words: no end of the core is silent. A faulting signal
# leaves a backtrace in the log, a signal that ends the core leaves its
# name and its sender, and session.quit leaves the reason. A core that
# was stopped for longer than CHORA_HANG_ABORT_S goes on after SIGCONT.
#
# Usage: tests/last-words.sh   CHORA_BIN: directory with the binaries
set -u
repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${CHORA_BIN:-$repo/target/debug}
work=$(mktemp -d -t last-words.XXXXXX)
session=
failed=0
fail() {
    echo "FAIL: $*"
    failed=1
}
ok() { echo "ok: $*"; }
stop() {
    [ -n "$session" ] && kill "$session" 2>/dev/null
    wait "$session" 2>/dev/null
    session=
}
trap 'stop; rm -rf "$work"' EXIT

start() {
    log=$work/$1.log
    CHORA_BIN=$bin CHORA_LOG=$log "$repo/tests/headless-session.sh" 800x600 >"$work/$1.env" 2>&1 &
    session=$!
    for _ in $(seq 60); do
        grep -q CHORA_SOCKET "$work/$1.env" && break
        sleep 0.5
    done
    core=$(pgrep -P "$session" -x chora)
    [ -n "$core" ] || {
        echo "FAIL: no core"
        exit 1
    }
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$work/$1.env")
    export CHORA_SOCKET
}
core_gone() { timeout 5 tail --pid="$core" -f /dev/null; }

start segv
kill -SEGV "$core"
core_gone
if grep -q "chora: SIGSEGV at address" "$log" && grep -q "backtrace follows" "$log"; then
    ok "a fault leaves a backtrace"
else
    fail "no last words after SIGSEGV: $(tail -n 3 "$log")"
fi
stop

start term
kill -TERM "$core"
core_gone
if grep -q "chora: SIGTERM from pid $$ ends chora" "$log"; then
    ok "SIGTERM leaves its name and its sender"
else
    fail "no last words after SIGTERM: $(tail -n 3 "$log")"
fi
stop

start quit
timeout 10 "$bin/choractl" quit >/dev/null
core_gone
if grep -q "chora ends" "$log" && grep "chora ends" "$log" | grep -q "session.quit"; then
    ok "session.quit leaves the reason of the end"
else
    fail "no reason after session.quit: $(tail -n 3 "$log")"
fi
stop

CHORA_HANG_ABORT_S=2 start stopped
kill -STOP "$core"
sleep 4
kill -CONT "$core"
sleep 2
if kill -0 "$core" 2>/dev/null && timeout 5 "$bin/choractl" query >/dev/null 2>&1; then
    ok "a core that was stopped goes on after SIGCONT"
else
    fail "the core is gone after SIGSTOP and SIGCONT: $(tail -n 3 "$log")"
fi
stop
exit "$failed"
