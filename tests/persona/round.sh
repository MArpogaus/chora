#!/bin/bash
# The launcher of a round of persona sessions: one snapshot of the binaries
# (one sha for the whole round), at most two sessions at once (4 CPUs), the
# captures and actions of each session kept, the checker on.
#
# Usage: tests/persona/round.sh snapshot            copy the binaries of target/debug
#        tests/persona/round.sh start NAME FLAVOR[+FLAVOR] [WxH]
#                                   waits for a free slot, starts the session
#                                   and prints PERSONA_ENV=<file>; the checker
#                                   has its first snapshot
#        tests/persona/round.sh probe NAME
#                                   starts a chora-probe window with
#                                   --report in the session of NAME, on the
#                                   socket name (WAYLAND_DISPLAY, not the
#                                   path), fails when it does not stay up and
#                                   prints PROBE_REPORT=<file> for
#                                   check.py end --probe (T5)
#        tests/persona/round.sh stop NAME
#        tests/persona/round.sh smoke              each look: start, one action,
#                                   check.py end T7, stop
# Environment:
#   ROUND_DIR  the folder of the round (default /tmp/persona-round)
#   ROUND_KEEP the folder for the captures and actions of each NAME
#              (default $ROUND_DIR/keep)
set -u
repo=$(cd "$(dirname "$0")/../.." && pwd)
dir=${ROUND_DIR:-/tmp/persona-round}
keep=${ROUND_KEEP:-$dir/keep}
mkdir -p "$dir/run" "$keep"
cmd=${1:-}
name=${2:-}

snapshot() {
    rm -rf "${dir:?}/bin"
    mkdir -p "$dir/bin"
    local b
    for b in "$repo"/target/debug/chora*; do
        [[ -f $b && -x $b && $b != *.d ]] && cp "$b" "$dir/bin/"
    done
    git -C "$repo" rev-parse HEAD >"$dir/sha"
    echo "binaries of $(cut -c1-8 "$dir/sha") in $dir/bin"
}

start() {
    local flavor=${3:?flavor} size=${4:-1280x800} out=$dir/run/$name.out s pid env_file
    [[ -x $dir/bin/chora ]] || snapshot
    [[ ! -e $keep/$name ]] || { echo "name $name already used" >&2; exit 2; }
    while :; do
        for s in 1 2; do
            setsid bash -c "exec 9>$dir/run/slot$s.lock; flock -n 9 || exit 75
                exec env CHORA_BIN=$dir/bin PERSONA_KEEP=$keep/$name PERSONA_CHECK=1 \\
                    CHORA_TEST_FLAGS=--test-injected-press-is-real \\
                    $repo/tests/persona/session.sh $flavor $size" >"$out" 2>&1 </dev/null &
            pid=$!
            sleep 2
            if ! kill -0 $pid 2>/dev/null; then
                wait $pid
                [[ $? -eq 75 ]] && continue
                echo "session failed:" >&2; cat "$out" >&2; exit 1
            fi
            for _ in $(seq 120); do
                env_file=$(sed -n 's/^PERSONA_ENV=//p' "$out" | head -1)
                [[ -n $env_file ]] && break
                kill -0 $pid 2>/dev/null || { echo "session failed:" >&2; cat "$out" >&2; exit 1; }
                sleep 1
            done
            [[ -n $env_file ]] || { echo "session did not come up:" >&2; cat "$out" >&2; exit 1; }
            echo "$env_file" >"$dir/run/$name.env"
            sleep 3
            PERSONA_ENV=$env_file python3 -I "$repo/tests/persona/check.py" start
            echo "PERSONA_ENV=$env_file"
            exit 0
        done
        sleep 20
    done
}

probe() {
    local env_file session_dir socket report pid
    env_file=$(cat "$dir/run/$name.env" 2>/dev/null) || { echo "no session $name" >&2; exit 1; }
    session_dir=$(dirname "$env_file")
    socket=$(sed -n 's/^WAYLAND_DISPLAY=//p' "$session_dir/headless.out")
    [[ -n $socket ]] || { echo "session $name has no WAYLAND_DISPLAY in headless.out" >&2; exit 1; }
    socket=$(basename "$socket")
    report=$dir/run/$name.probe
    : >"$report"
    # shellcheck disable=SC2016 # the inner shell expands its own $$ and arguments
    WAYLAND_DISPLAY=$socket XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$env_file") \
        setsid bash -c 'echo $$ >"$1"; shift; exec "$@"' _ "$report.pid" \
        "$dir/bin/chora-probe" window 200 200 200 --report >"$report" 2>"$report.err" </dev/null &
    sleep 3
    pid=$(cat "$report.pid" 2>/dev/null || true)
    if [[ -z $pid ]] || ! kill -0 "$pid" 2>/dev/null; then
        echo "the probe did not stay up (WAYLAND_DISPLAY=$socket): $(cat "$report.err" 2>/dev/null)" >&2
        exit 1
    fi
    echo "PROBE_REPORT=$report"
}

stop() {
    local env_file
    env_file=$(cat "$dir/run/$name.env" 2>/dev/null) || { echo "no session $name" >&2; exit 1; }
    if [[ -f $dir/run/$name.probe.pid ]]; then
        kill "$(cat "$dir/run/$name.probe.pid")" 2>/dev/null || true
        rm -f "$dir/run/$name.probe.pid"
    fi
    "$repo/tests/persona/session.sh" --stop "$env_file" && echo "stopped $name"
}

smoke() {
    local look env_file n=0 out
    for look in tiling xp mc construct; do
        name=smoke-$look-$$
        env_file=$(start start "$name" "$look" | sed -n 's/^PERSONA_ENV=//p')
        [[ -n $env_file ]] || { echo "FAIL: $look: no session" >&2; return 1; }
        export PERSONA_ENV=$env_file PERSONA_CHECK=1
        python3 -I "$repo/tests/persona/check.py" start
        python3 -I "$repo/tests/persona/act.py" move 100 100 >/dev/null || { echo "FAIL: $look: act.py move" >&2; return 1; }
        grep -q '"action": "move"' "$(sed -n 's/^PERSONA_LOG=//p' "$env_file")" || { echo "FAIL: $look: no action in the log" >&2; return 1; }
        [[ -s $(dirname "$env_file")/check/history.jsonl ]] || { echo "FAIL: $look: the checker saw no action" >&2; return 1; }
        out=$(python3 -I "$repo/tests/persona/check.py" end T7) || { echo "FAIL: $look: check.py end T7: $out" >&2; return 1; }
        echo "$look: $(echo "$out" | tr '\n' ' ')"
        stop
        n=$((n + 1))
    done
    echo "smoke: $n looks, each with a session, an action, the checker and check.py end"
}

case $cmd in
snapshot) snapshot ;;
start) [[ -n $name ]] && start "$@" ;;
probe) [[ -n $name ]] && probe ;;
stop) [[ -n $name ]] && stop ;;
smoke) smoke ;;
*) sed -n '2,/^set -u/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
