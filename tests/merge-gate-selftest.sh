#!/bin/bash
# Self-test of tests/merge-gate.sh: it runs the gate against fake steps
# (MERGE_GATE_STEPS) and a fake git on PATH, so it needs no build.
#
# Usage: tests/merge-gate-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
gate=$repo/tests/merge-gate.sh
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir "$work/bin"
cat >"$work/bin/git" <<'FAKE'
#!/bin/bash
echo "$*" >>"$FAKE_GIT_LOG"
if [ "$1" = status ]; then
    [ -z "${FAKE_GIT_DIRTY:-}" ] || echo " M file"
fi
exit 0
FAKE
chmod +x "$work/bin/git"

failures=0
check() {
    if [ "$2" = "$3" ]; then
        echo "ok: $1"
    else
        echo "FAIL: $1: got '$2', want '$3'"
        failures=$((failures + 1))
    fi
}

run_gate() {
    : >"$work/git.log"
    env -u CHORA_GATE_LOCK_HELD CHORA_GATE_LOCK="$work/gate.lock" \
        PATH="$work/bin:$PATH" FAKE_GIT_LOG="$work/git.log" MERGE_GATE_STEPS="$work/steps" \
        MERGE_GATE_RUNS="$work/runs" "$gate" "$@" >"$work/out" 2>&1
    rc=$?
    pushes=$(grep -c '^push ' "$work/git.log")
}

printf 'one|true\ntwo|true\n' >"$work/steps"
run_gate
check "a passing run exits 0" "$rc" 0
check "a passing run without --push does not push" "$pushes" 0

run_gate --push session
check "a passing run with --push exits 0" "$rc" 0
check "a passing run with --push pushes once" "$pushes" 1
check "the push is HEAD to the branch" "$(grep -c '^push origin HEAD:session$' "$work/git.log")" 1

printf 'one|true\ntwo|exit 1\nthree|touch %s/third\n' "$work" >"$work/steps"
run_gate --push session
check "a step that exits 1 stops the gate" "$rc" 1
check "a step that exits 1 stops before the next step" "$(test -e "$work/third" && echo ran)" ""
check "a step that exits 1 does not push" "$pushes" 0
check "a failed step prints the word FAIL" "$(grep -c 'two.*FAIL' "$work/out")" 1

printf 'one|echo "FAIL: x"\n' >"$work/steps"
run_gate --push session
check "a step that prints FAIL: stops the gate" "$rc" 1
check "a step that prints FAIL: does not push" "$pushes" 0

printf 'one|echo "MISMATCH cell"\n' >"$work/steps"
run_gate --push session
check "a step that prints MISMATCH stops the gate" "$rc" 1
check "a step that prints MISMATCH does not push" "$pushes" 0

printf 'one|echo "test result: FAILED. 1 failed"\n' >"$work/steps"
run_gate
check "a step that prints test result: FAILED stops the gate" "$rc" 1

printf 'one|echo "no FAIL: at the start"\n' >"$work/steps"
run_gate
check "FAIL: inside a line does not stop the gate" "$rc" 0

printf 'one|true\ntwo|true\n' >"$work/steps"
FAKE_GIT_DIRTY=1 run_gate --push session
check "a dirty tree refuses" "$rc" 1
check "a dirty tree does not push" "$pushes" 0

run_gate --only one --push session
check "--only passes" "$rc" 0
check "--only never pushes" "$pushes" 0

run_gate --only nosuch
check "--only with an unknown step is a usage error" "$rc" 2

run_gate --help
check "--help exits 0" "$rc" 0

hold_lock() {
    rm -f "$work/held" "$work/released"
    (
        flock 9
        touch "$work/held"
        sleep 2
        date +%s.%N >"$work/released"
    ) 9>"$work/gate.lock" &
    holder=$!
    for _ in $(seq 50); do
        [ -e "$work/held" ] && break
        sleep 0.1
    done
}
after_release() {
    awk -v a="$(cat "$work/$1" 2>/dev/null || echo 0)" -v b="$(cat "$work/released" 2>/dev/null || echo 9e99)" \
        'BEGIN { print (a >= b) ? "after" : "before" }'
}

printf 'one|date +%%s.%%N >%s/started\n' "$work" >"$work/steps"
hold_lock
run_gate
wait "$holder"
check "a second gate passes after the first one" "$rc" 0
check "a second gate starts its steps after the first one ends" "$(after_release started)" after
check "a second gate prints that it waits" "$(grep -c '^merge-gate: waits for' "$work/out")" 1

printf 'one|timeout 5 bash -c %s\n' "'. tests/gate-lib.sh'" >"$work/steps"
run_gate
check "a gate script inside the gate does not wait for the lock of the gate" "$rc" 0

env -u CHORA_GATE_LOCK_HELD -u MERGE_GATE_STEPS CHORA_GATE_LOCK="$work/gate.lock" \
    MERGE_GATE_RUNS="$work/runs" "$gate" --only gate-lib >"$work/out" 2>&1
check "the step gate-lib runs the self-test of tests/gate-lib.sh" "$?" 0

[ "$failures" = 0 ] || {
    echo "FAIL: $failures checks of merge-gate-selftest"
    exit 1
}
echo "ok: merge-gate-selftest"
