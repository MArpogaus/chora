#!/bin/bash
# Self-test of the `ratchets` step of tests/merge-gate.sh: on a scratch worktree
# of HEAD the step passes; with one added wait line in a tests/*.sh file it
# fails with a line that names sleep-ratchet. The added line is built from two
# halves so that this file adds no line to the count it tests.
#
# Usage: tests/gate-ratchets-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'git -C "$repo" worktree remove --force "$work/wt" >/dev/null 2>&1; rm -rf "$work"' EXIT
failures=0
fail() {
    echo "FAIL: $*"
    failures=$((failures + 1))
}

git -C "$repo" worktree add --detach "$work/wt" HEAD >/dev/null 2>&1 || {
    echo "FAIL: no scratch worktree"
    exit 1
}
word=sl$(printf ee)p
run() { (cd "$work/wt" && MERGE_GATE_RUNS=$work CHORA_GATE_LOCK=$work/lock tests/merge-gate.sh --only ratchets --base HEAD >"$work/out" 2>&1); }

run || fail "clean tree: the ratchets step fails: $(tail -5 "$work/out")"
grep -q '^ratchets .*PASS' "$work/out" || fail "clean tree: no PASS line for ratchets: $(tail -5 "$work/out")"

printf '%s 1\n' "$word" >>"$work/wt/tests/zz-ratchet-fixture.sh"
if run; then
    fail "added wait: the ratchets step passed"
fi
grep -q 'FAIL: sleep-ratchet' "$work/out" || fail "added wait: no line names sleep-ratchet: $(tail -8 "$work/out")"

[ "$failures" = 0 ] && echo "gate-ratchets-selftest: ok"
exit "$failures"
