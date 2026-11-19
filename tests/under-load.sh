#!/bin/bash
# Runs a gate command N times while busy loops hold the CPUs, as the merge
# gate under --jobs 3 does on a lane container, and counts the runs that
# passed. The reproducer of a load flake: a cell that reads a state at a
# fixed time fails here where it passes alone. The busy loops are ended
# by pid when the script ends.
#
# Usage: tests/under-load.sh [--runs N] [--cpus N] [--timeout S] [--keep DIR]
#                            -- COMMAND...
#   --runs N   how many runs of COMMAND (default 10)
#   --timeout S  a run that takes longer than S seconds is ended (TERM,
#              KILL 10 s later) and fails (default 1800)
#   --cpus N   how many busy loops (default: the number of CPUs)
#   --keep DIR the log of every run goes to DIR/run-I.log (default: a
#              temporary directory, printed at the end)
# Exit status: 0 when every run passed, 1 else. One line per run:
# `run I: PASS|FAIL S s`, and a failed run prints its FAIL lines; a run
# ended at its limit says `timed out after S s`.
set -u
runs=10
cpus=$(nproc)
limit=1800
keep=""
while [ $# -gt 0 ]; do
    case $1 in
    --runs) runs=$2; shift 2 ;;
    --cpus) cpus=$2; shift 2 ;;
    --timeout) limit=$2; shift 2 ;;
    --keep) keep=$2; shift 2 ;;
    --) shift; break ;;
    *) echo "under-load.sh: unknown argument $1" >&2; exit 2 ;;
    esac
done
[ $# -gt 0 ] || { echo "under-load.sh: no command" >&2; exit 2; }
[ -n "$keep" ] || keep=$(mktemp -d /tmp/under-load.XXXXXX)
mkdir -p "$keep"
busy=()
end_busy() {
    [ "${#busy[@]}" = 0 ] || kill "${busy[@]}" 2>/dev/null
    wait "${busy[@]}" 2>/dev/null
}
trap end_busy EXIT
for _ in $(seq "$cpus"); do
    sh -c 'while :; do :; done' &
    busy+=($!)
done
passed=0
for i in $(seq "$runs"); do
    start=$(date +%s)
    timeout -k 10 "$limit" "$@" >"$keep/run-$i.log" 2>&1
    status=$?
    if [ "$status" = 0 ] && ! grep -q '^FAIL' "$keep/run-$i.log"; then
        passed=$((passed + 1))
        echo "run $i: PASS $(($(date +%s) - start)) s"
    elif [ "$status" = 124 ] || [ "$status" = 137 ]; then
        echo "run $i: FAIL $(($(date +%s) - start)) s, timed out after $limit s"
    else
        echo "run $i: FAIL $(($(date +%s) - start)) s"
        grep -h 'FAIL' "$keep/run-$i.log" | head -n 5 | sed 's/^/  /'
    fi
done
echo "under load ($cpus busy loops): $passed of $runs passed; logs in $keep"
[ "$passed" = "$runs" ]
