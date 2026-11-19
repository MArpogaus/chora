#!/bin/bash
# Self-test of tests/gate-lib.sh. Its EXIT trap: a gate that ends, also
# with a failure, ends at once and leaves none of its programs behind,
# also not one that it stopped with SIGSTOP to force a late answer. Its
# lock: a gate script waits while another gate holds the lock, and says
# so; the time of a gate (gate_elapsed) starts when it has the lock, not
# in the queue for it. Its session_start with the env file of an earlier session of the
# same name (gate-construct-pull.sh runs `pull behind` twice): with the
# new session held 1 s before its redirect truncates the file, as on a
# busy machine (F21), it takes the socket of the new session, not the
# gone one of the old file.
#
# Usage: tests/gate-lib-selftest.sh
# shellcheck disable=SC2016 # the inner gate expands its own arguments
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
export CHORA_GATE_LOCK=$work/gate.lock
unset CHORA_GATE_LOCK_HELD

timeout -k 1 10 bash -c '
. "$1/tests/gate-lib.sh"
trap gate_cleanup EXIT
runs() { until [ "$(ps -o comm= -p "$1")" = sleep ]; do sleep 0.05; done; }
sleep 60 &
pids+=($!)
echo $! >"$2/running"
sleep 60 &
pids+=($!)
echo $! >"$2/stopped"
runs "$!"
runs "$(cat "$2/running")"
kill -STOP "$!"
fail "the gate fails while a program stands still"
' gate "$repo" "$work" >/dev/null 2>&1
status=$?
failures=0
if [ "$status" = 124 ]; then
    echo "FAIL: the EXIT trap of the gate hangs"
    failures=1
fi
for program in running stopped; do
    pid=$(cat "$work/$program")
    for _ in $(seq 20); do
        kill -0 "$pid" 2>/dev/null || break
        sleep 0.1
    done
    if kill -0 "$pid" 2>/dev/null; then
        echo "FAIL: the $program program $pid outlives the gate"
        kill -KILL "$pid"
        failures=$((failures + 1))
    fi
done
(
    flock 9
    touch "$work/held"
    sleep 2
    date +%s.%N >"$work/released"
) 9>"$CHORA_GATE_LOCK" &
for _ in $(seq 50); do
    [ -e "$work/held" ] && break
    sleep 0.1
done
timeout 10 bash -c '. "$1/tests/gate-lib.sh"; date +%s.%N >"$2/started"' gate "$repo" "$work" >"$work/out" 2>&1
wait
if ! awk -v a="$(cat "$work/started")" -v b="$(cat "$work/released")" 'BEGIN { exit !(a >= b) }'; then
    echo "FAIL: a gate script runs while another gate holds the lock"
    failures=$((failures + 1))
fi
if ! grep -q '^gate: waits for' "$work/out"; then
    echo "FAIL: a gate script that waits for the lock does not say so: $(cat "$work/out")"
    failures=$((failures + 1))
fi
(
    flock 9
    touch "$work/held2"
    sleep 3
) 9>"$CHORA_GATE_LOCK" &
for _ in $(seq 50); do
    [ -e "$work/held2" ] && break
    sleep 0.1
done
queued=$(timeout 15 bash -c '. "$1/tests/gate-lib.sh"; echo "$SECONDS $(gate_elapsed)"' gate "$repo" 2>/dev/null | tail -n 1)
wait
read -r queued_seconds queued_elapsed <<<"$queued"
if [ "${queued_seconds:-0}" -lt 2 ] || [ "${queued_elapsed:-9}" != 0 ]; then
    echo "FAIL: gate_elapsed counts the wait for the gate lock: shell time ${queued_seconds:-none} s, gate_elapsed ${queued_elapsed:-none} s (want 0 after a 3 s wait)"
    failures=$((failures + 1))
fi
mkdir "$work/own" "$work/other"
cp "$(command -v sleep)" "$work/gl-probe"
XDG_RUNTIME_DIR=$work/other "$work/gl-probe" 30 &
other=$!
XDG_RUNTIME_DIR=$work/own "$work/gl-probe" 30 &
own=$!
sleep 0.3
found=$(CHORA_GATE_LOCK_HELD=1 XDG_RUNTIME_DIR=$work/own bash -c '. "$1/tests/gate-lib.sh"; session_pids -x gl-probe' gate "$repo" 2>&1)
CHORA_GATE_LOCK_HELD=1 XDG_RUNTIME_DIR=$work/none bash -c '. "$1/tests/gate-lib.sh"; session_pids -x gl-probe' gate "$repo" >/dev/null 2>&1
none=$?
kill "$own" "$other" 2>/dev/null
if [ "$found" != "$own" ]; then
    echo "FAIL: session_pids finds $found, not only the program of its own runtime directory $own (other $other)"
    failures=$((failures + 1))
fi
if [ "$none" != 1 ]; then
    echo "FAIL: session_pids exits $none, not 1, when its runtime directory has no such program"
    failures=$((failures + 1))
fi
bash -c '"$1" 0 & "$1" 30 & exec "$1" 30' restarted "$work/gl-probe" &
restarted=$!
bash -c '"$1" 0 & exec "$1" 30' dead "$work/gl-probe" &
dead=$!
sleep 0.3
bash -c '. "$1/tests/proc-lib.sh"; live_child "$2" gl-probe' gate "$repo" "$restarted" >/dev/null 2>&1
beside_zombie=$?
bash -c '. "$1/tests/proc-lib.sh"; live_child "$2" gl-probe' gate "$repo" "$dead" >/dev/null 2>&1
only_zombie=$?
pkill -P "$restarted"
pkill -P "$dead"
kill "$restarted" "$dead" 2>/dev/null
if [ "$beside_zombie" != 0 ]; then
    echo "FAIL: live_child exits $beside_zombie, not 0, for a running child beside a zombie of the same name"
    failures=$((failures + 1))
fi
if [ "$only_zombie" != 1 ]; then
    echo "FAIL: live_child exits $only_zombie, not 1, when the only child of the name is a zombie"
    failures=$((failures + 1))
fi
skip_status() { env "$@" bash -c '. "$1/tests/gate-lib.sh"; skip "no tool"' gate "$repo" >"$work/skip.out" 2>&1; echo $?; }
if [ "$(skip_status -u CI)" != 0 ]; then
    echo "FAIL: a skip without CI does not exit 0"
    failures=$((failures + 1))
fi
if [ "$(skip_status CI=true)" = 0 ]; then
    echo "FAIL: a skip under CI exits 0"
    failures=$((failures + 1))
fi
if ! grep -q 'no tool' "$work/skip.out"; then
    echo "FAIL: a skip under CI does not print its reason: $(cat "$work/skip.out")"
    failures=$((failures + 1))
fi
slow_client_dir=$work/slow-client
mkdir "$slow_client_dir"
bash -c 'trap "kill \$!; sleep 1; echo late >\"\$0/late\"; exit 0" TERM; sleep 60 & wait' "$slow_client_dir" &
slow_client=$!
sleep 0.3
(
    . "$repo/tests/proc-lib.sh"
    kill "$slow_client"
    remove_after_exit "$slow_client_dir" "$slow_client"
)
if [ -e "$slow_client_dir" ]; then
    echo "FAIL: remove_after_exit left the directory of a client that was still exiting"
    failures=$((failures + 1))
fi
if kill -0 "$slow_client" 2>/dev/null && [ "$(ps -o stat= -p "$slow_client" | cut -c1)" != Z ]; then
    echo "FAIL: remove_after_exit returned while the client still ran"
    failures=$((failures + 1))
fi
kill -KILL "$slow_client" 2>/dev/null
stubborn_dir=$work/stubborn
mkdir "$stubborn_dir"
bash -c 'trap "" TERM; while :; do sleep 0.1; done' &
stubborn=$!
sleep 0.3
. "$repo/tests/proc-lib.sh"
kill "$stubborn"
timeout 12 bash -c '. "$1/tests/proc-lib.sh"; remove_after_exit "$2" "$3"' x "$repo" "$stubborn_dir" "$stubborn"
if [ -e "$stubborn_dir" ]; then
    echo "FAIL: remove_after_exit left the directory of a client that ignores TERM"
    failures=$((failures + 1))
fi
if running "$stubborn"; then
    echo "FAIL: remove_after_exit did not end a client that ignores TERM"
    failures=$((failures + 1))
fi
kill -KILL "$stubborn" 2>/dev/null
fake=$work/fake
mkdir -p "$fake/tests"
cp "$repo/tests/gate-lib.sh" "$repo/tests/gate-lock.sh" "$fake/tests/"
printf '#!/bin/bash\necho XDG_RUNTIME_DIR=/new\necho WAYLAND_DISPLAY=/new/wayland-1\necho CHORA_SOCKET=/new/chora/wayland-1.sock\nexec sleep 5\n' \
    >"$fake/tests/headless-session.sh"
chmod +x "$fake/tests/headless-session.sh"
printf 'CHORA_SOCKET=/old/chora/wayland-1.sock\n' >"$work/again.env"
again=$(CHORA_GATE_LOCK=$work/again.lock timeout -k 1 30 bash -c '
. "$1/tests/gate-lib.sh"
set -T
trap "case \$BASH_COMMAND in pids+=*) kill -STOP \$!; (command sleep 1; kill -CONT \$!) & ;; esac" DEBUG
env_file=$2 session_start
trap - DEBUG
echo "$CHORA_SOCKET"' gate "$fake" "$work/again.env" 2>&1 | tail -n 1)
if [ "$again" != /new/chora/wayland-1.sock ]; then
    echo "FAIL: session_start took the socket ${again:-none} of an old env file, not the one of its new session"
    failures=$((failures + 1))
fi
[ "$failures" = 0 ] || exit 1
echo "ok: gate-lib-selftest"
