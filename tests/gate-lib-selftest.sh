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
# A cell that ends in any way leaves no compositor of its run: it fails,
# it fails in a subshell, the timeout ends it, SIGKILL ends it, the group of
# the cell is killed with SIGKILL (the stop of merge-gate after its grace).
#
# Its load_scaled_limit (sh-37): a timing bound that a loaded host breaks
# (gate-mc-scale: a settings.set beside 300 panels took 9316 ms, bound 5000 ms)
# is the floor times how much slower a probe is now than at its best in the same
# cell, at least the floor, so a slow host raises it and an idle one keeps it.
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
if [ "${queued_seconds:-0}" -lt 2 ] || [ "${queued_elapsed:-9}" -gt 1 ]; then
    echo "FAIL: gate_elapsed counts the wait for the gate lock: shell time ${queued_seconds:-none} s, gate_elapsed ${queued_elapsed:-none} s (want at most 1 after a 3 s wait: SECONDS counts whole seconds)"
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
mkdir -p "$fake/target/debug"
printf '#!/bin/sh\necho "[]"\n' >"$fake/target/debug/choractl"
chmod +x "$fake/target/debug/choractl"
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
leak=$work/leak
mkdir -p "$leak/own" "$leak/other"
cp "$(command -v sleep)" "$leak/chora-probe"
PATH=$leak:$PATH XDG_RUNTIME_DIR=$leak/other chora-probe 300 &
bystander=$!
for ending in 'exit 0' 'fail "a cell that fails"'; do
    rm -f "$leak/direct" "$leak/orphan"
    XDG_RUNTIME_DIR=$leak/own CHORA_GATE_LOCK=$leak/gate.lock timeout --foreground -k 1 15 bash -c '
. "$1/tests/gate-lib.sh"
temp_config
"$2/chora-probe" 30 &
echo $! >"$2/direct"
( "$2/chora-probe" 30 & echo $! >"$2/orphan" )
sleep 0.3
eval "$3"' gate "$repo" "$leak" "$ending" >/dev/null 2>&1
    for which in direct orphan; do
        pid=$(cat "$leak/$which" 2>/dev/null)
        for _ in $(seq 20); do
            kill -0 "$pid" 2>/dev/null || break
            sleep 0.1
        done
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            echo "FAIL: the $which chora-probe $pid outlives a cell that ends with: $ending"
            kill -KILL "$pid"
            failures=$((failures + 1))
        fi
    done
done
if ! kill -0 "$bystander" 2>/dev/null; then
    echo "FAIL: the gate ended a chora-probe of another runtime directory"
    failures=$((failures + 1))
fi
kill "$bystander" 2>/dev/null
ghost=$work/ghost
mkdir -p "$ghost"
printf '#!/bin/bash\necho $$ >>"$(dirname "$0")/pids"\nmkdir -p "$XDG_RUNTIME_DIR/chora"\ntouch "$XDG_RUNTIME_DIR/chora/wayland-1.sock"\nsleep 300 & wait\n' >"$ghost/chora"
printf '#!/bin/sh\necho "[]"\n' >"$ghost/choractl"
chmod +x "$ghost/chora" "$ghost/choractl"
cat >"$ghost/cell.sh" <<'CELL'
. "$1/tests/gate-lib.sh"
temp_config
session_start 800x600
echo ready >"$2/ready"
case $3 in
fail) fail "a cell that fails" ;;
subshell) (fail "a failure in a subshell"); sleep 300 ;;
hang) sleep 300 ;;
kill) kill -KILL $$ ;;
group) sleep 300 ;;
esac
CELL
leaks=0
for ending in fail subshell hang kill group; do
    rm -f "$ghost/ready"
    export BIN=$ghost CHORA_BIN=$ghost
    case $ending in
    hang) CHORA_GATE_LOCK=$ghost/lock.$ending timeout -k 1 4 bash "$ghost/cell.sh" "$repo" "$ghost" "$ending" >/dev/null 2>&1 ;;
    group)
        (CHORA_GATE_LOCK=$ghost/lock.$ending exec setsid bash "$ghost/cell.sh" "$repo" "$ghost" "$ending" >/dev/null 2>&1) &
        cellpid=$!
        until [ -e "$ghost/ready" ]; do sleep 0.1; done
        kill -KILL -- "-$cellpid"
        wait "$cellpid" 2>/dev/null
        ;;
    *) CHORA_GATE_LOCK=$ghost/lock.$ending timeout -k 1 20 bash "$ghost/cell.sh" "$repo" "$ghost" "$ending" >/dev/null 2>&1 ;;
    esac
    mapfile -t ghosts <"$ghost/pids"
    for _ in $(seq 30); do
        kill -0 "${ghosts[@]}" 2>/dev/null || break
        sleep 0.1
    done
    if kill -0 "${ghosts[@]}" 2>/dev/null; then
        echo "FAIL: a compositor outlives a cell that ends by $ending: $(ps -o pid,pgid,args -p "${ghosts[@]}" | tail -n 1)"
        kill -KILL "${ghosts[@]}" 2>/dev/null
        leaks=$((leaks + 1))
    fi
    rm -f "$ghost/pids"
done
unset BIN CHORA_BIN
failures=$((failures + leaks))
echo "leak cases: $((5 - leaks))/5 leave no compositor"
stub=$work/fuzzel-stub
mkdir -p "$stub/old" "$stub/new" "$stub/none"
printf '#!/bin/sh\necho "fuzzel version: 1.9.2 +cairo +png +svg(nanosvg) -assertions"\n' >"$stub/old/fuzzel"
printf '#!/bin/sh\necho "fuzzel version: 1.11.1 +cairo +png +svg(nanosvg) -assertions"\n' >"$stub/new/fuzzel"
printf '#!/bin/sh\necho "fuzzel version: 1.12.0 +cairo"\n' >"$stub/none/fuzzel"
chmod +x "$stub"/*/fuzzel
fuzzel_gate() { PATH=$stub/$1:$PATH CHORA_GATE_LOCK_HELD=1 env -u CI bash -c ". \"\$1/tests/gate-lib.sh\"; $2; echo went-on" gate "$repo" 2>&1; }
out=$(fuzzel_gate old 'need_fuzzel 1.11')
if [ "$out" != "SKIP: fuzzel 1.9.2 < 1.11" ]; then
    echo "FAIL: need_fuzzel 1.11 with fuzzel 1.9.2 does not end in 'SKIP: fuzzel 1.9.2 < 1.11': $out"
    failures=$((failures + 1))
fi
out=$(fuzzel_gate new 'need_fuzzel 1.11')
if [ "$out" != went-on ]; then
    echo "FAIL: need_fuzzel 1.11 with fuzzel 1.11.1 does not go on: $out"
    failures=$((failures + 1))
fi
out=$(fuzzel_gate none 'need_fuzzel 1.11')
if [ "$out" != went-on ]; then
    echo "FAIL: need_fuzzel 1.11 with fuzzel 1.12.0 does not go on: $out"
    failures=$((failures + 1))
fi
out=$(fuzzel_gate old 'fuzzel_older 1.11 && echo older; echo "$fuzzel_version"')
if [ "$out" != "$(printf 'older\n1.9.2\nwent-on')" ]; then
    echo "FAIL: fuzzel_older 1.11 with fuzzel 1.9.2 is not true with fuzzel_version 1.9.2: $out"
    failures=$((failures + 1))
fi
out=$(fuzzel_gate new 'fuzzel_older 1.11 || echo not-older')
if [ "$out" != "$(printf 'not-older\nwent-on')" ]; then
    echo "FAIL: fuzzel_older 1.11 with fuzzel 1.11.1 is true: $out"
    failures=$((failures + 1))
fi
out=$(fuzzel_gate new 'cell_skip launcher-click-row "fuzzel 1.9.2 < 1.11"')
if [ "$out" != "$(printf 'SKIP: cell launcher-click-row: fuzzel 1.9.2 < 1.11\nwent-on')" ]; then
    echo "FAIL: cell_skip does not print one SKIP line and go on: $out"
    failures=$((failures + 1))
fi
out=$(fuzzel_gate new 'cells_end 0 "launcher-click-row"')
if [ "$out" != "SKIP: no subtest ran (skipped: launcher-click-row)" ]; then
    echo "FAIL: a cell list with no subtest run does not end in SKIP: $out"
    failures=$((failures + 1))
fi
out=$(fuzzel_gate new 'cells_end 3 ""')
if [ "$out" != "$(printf 'ok: 3 cells ran\nwent-on')" ]; then
    echo "FAIL: a cell list that ran does not end in ok: $out"
    failures=$((failures + 1))
fi
out=$(fuzzel_gate new 'cells_end 3 "launcher-click-row"')
if [ "$out" != "$(printf 'ok: 3 cells ran\nwent-on')" ]; then
    echo "FAIL: a cell list that ran 3 and skipped 1 does not end in ok: $out"
    failures=$((failures + 1))
fi
out=$(CHORA_GATE_LOCK_HELD=x bash -c '. "$1/tests/gate-lib.sh"; (sleep 1; touch "$2/late") & until_ok "[ -e \"$2/late\" ]" 20 && echo came' gate "$repo" "$work" 2>&1)
[ "$out" = came ] || {
    echo "FAIL: until_ok does not wait for a state that comes after 1 s: $out"
    failures=$((failures + 1))
}
out=$(CHORA_GATE_LOCK_HELD=x bash -c '. "$1/tests/gate-lib.sh"; until_ok false 2 || echo gave-up' gate "$repo" 2>&1)
[ "$out" = gave-up ] || {
    echo "FAIL: until_ok does not give up on a state that never comes: $out"
    failures=$((failures + 1))
}
lsl() { CHORA_GATE_LOCK_HELD=x bash -c '. "$1/tests/gate-lib.sh"; load_scaled_limit "$2" "$3" "$4"' gate "$repo" "$@" 2>&1; }
idle=$(lsl 5000 69 44)
[ "$idle" = 5000 ] || {
    echo "FAIL: load_scaled_limit 5000 69 44 gives '$idle', want 5000 (idle probe jitter keeps the floor)"
    failures=$((failures + 1))
}
[ "${idle:-0}" -lt 5500 ] 2>/dev/null || {
    echo "FAIL: a 2x regression (5.5 s) must fail an idle host's bound, which is '$idle' ms"
    failures=$((failures + 1))
}
loaded=$(lsl 5000 600 200)
[ "$loaded" = 7500 ] || {
    echo "FAIL: load_scaled_limit 5000 600 200 gives '$loaded', want 7500 (a probe 3x slower: 1.5x the floor)"
    failures=$((failures + 1))
}
capped=$(lsl 5000 4000 100)
[ "$capped" = 9000 ] || {
    echo "FAIL: load_scaled_limit 5000 4000 100 gives '$capped', want 9000 (cap below the core's 10 s)"
    failures=$((failures + 1))
}
[ "$failures" = 0 ] || exit 1
echo "ok: gate-lib-selftest"
