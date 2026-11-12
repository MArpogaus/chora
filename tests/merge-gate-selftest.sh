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

: >"$work/marks"
for n in build test clippy xp hud cube; do printf '%s|echo %s >>%s/marks\n' "$n" "$n" "$work"; done >"$work/steps"
run_gate --worker xp,hud
check "--worker runs build, test, clippy and the named steps" "$(tr '\n' ' ' <"$work/marks")" "build test clippy xp hud "
check "--worker exits 0" "$rc" 0
run_gate --worker ''
check "--worker with no steps runs the three base steps" "$(wc -l <"$work/marks")" 8
run_gate --worker xp --push session
check "--worker with --push is refused" "$rc" 2
run_gate --worker nosuch
check "--worker with an unknown step is refused" "$rc" 2

rr=$work/rr
mkdir -p "$rr/tests"
cp "$gate" "$rr/tests/merge-gate.sh"
cp "$repo/tests/gate-lock.sh" "$rr/tests/"
git -C "$rr" init -q -b main
git -C "$rr" config user.email t@t
git -C "$rr" config user.name t
git -C "$rr" commit -q --allow-empty -m "chore: init"
git -C "$rr" tag base
red() { "$rr/tests/merge-gate.sh" --red-first-check --base base 2>&1; }
echo a >"$rr/a.rs"
git -C "$rr" add a.rs
git -C "$rr" commit -q -m "feat(a): no test before it"
check "a feat commit without a test before it is red" "$(red | grep -c '^RED-FIRST MISSING: .* feat(a): no test before it$')" 1
git -C "$rr" commit -q --amend -m "feat(a): excused" -m "No test: a pure move"
check "a body line No test: excuses the commit" "$(red >/dev/null; echo $?)" 0
git -C "$rr" commit -q --amend -m "feat(a): and its test"
echo t >"$rr/tests/gate-a.sh"
git -C "$rr" add tests/gate-a.sh
git -C "$rr" commit -q --amend --no-edit
check "a feat commit that changes a test file is green" "$(red >/dev/null; echo $?)" 0
git -C "$rr" reset -q --hard base
echo t >"$rr/tests/gate-a.sh"
git -C "$rr" add tests/gate-a.sh
git -C "$rr" commit -q -m "test(a): first"
echo b >"$rr/b.rs"
git -C "$rr" add b.rs
git -C "$rr" commit -q -m "fix(b): test came first"
check "a test commit before a fix commit is green" "$(red >/dev/null; echo $?)" 0
echo c >"$rr/c.rs"
git -C "$rr" add c.rs
git -C "$rr" commit -q -m "refactor(c): no test needed"
check "a refactor commit needs no test" "$(red >/dev/null; echo $?)" 0

rr_commit() {
    git -C "$rr" add -- "${@:2}"
    git -C "$rr" commit -q -m "$1"
}
rr_file() {
    mkdir -p "$(dirname "$rr/$1")"
    printf '%s\n' "$2" >"$rr/$1"
}

git -C "$rr" reset -q --hard base
rr_file b.rs b
rr_commit "fix(b): the fix comes first" b.rs
rr_file tests/gate-b.sh t
rr_commit "test(b): the test comes second" tests/gate-b.sh
check "a fix before its test is red" "$(red | grep -c '^RED-FIRST MISSING: .* fix(b): the fix comes first$')" 1

for helper in tests/gate-lib.sh tests/ci-local.sh tests/gate-lock.sh; do
    git -C "$rr" reset -q --hard base
    rr_file "$helper" t
    rr_commit "test(h): a helper changed" "$helper"
    rr_file c.rs c
    rr_commit "fix(c): after a helper" c.rs
    check "a change of $helper is not a test" "$(red | grep -c '^RED-FIRST MISSING: .* fix(c): after a helper$')" 1
done

for real in tests/gate-x.sh tests/x-selftest.sh tests/test_x.py crates/k/tests/x.rs; do
    git -C "$rr" reset -q --hard base
    rr_file "$real" t
    rr_commit "test(h): a real test" "$real"
    rr_file c.rs c
    rr_commit "fix(c): after a test" c.rs
    check "a change of $real is a test" "$(red >/dev/null; echo $?)" 0
done

git -C "$rr" reset -q --hard base
rr_file lib.rs 'fn f() {}'
rr_commit "chore: a source file" lib.rs
rr_file lib.rs '#[test]
fn f() {}'
rr_commit "test(l): a #[test] line only" lib.rs
rr_file c.rs c
rr_commit "fix(c): after a #[test] line" c.rs
check "a diff that only adds a #[test] line counts" "$(red >/dev/null; echo $?)" 0

git -C "$rr" reset -q --hard base
git -C "$rr" checkout -q -b side
rr_file s.txt s
rr_commit "chore: side" s.txt
git -C "$rr" checkout -q main
git -C "$rr" merge -q --no-ff side -m "fix(m): a merge without a test"
check "a merge commit in the range is not checked" "$(red >/dev/null; echo $?)" 0
git -C "$rr" branch -q -D side

bin2=$work/bin2
mkdir "$bin2"
cat >"$bin2/pre-commit" <<'FAKE'
#!/bin/bash
echo "SKIP=${SKIP:-}" >"$FAKE_PRE_COMMIT_LOG"
FAKE
chmod +x "$bin2/pre-commit"
gate_only() {
    env -u CHORA_GATE_LOCK_HELD -u MERGE_GATE_STEPS CHORA_GATE_LOCK="$work/gate.lock" \
        MERGE_GATE_RUNS="$work/runs" "$gate" --base HEAD --only pre-commit >"$work/out" 2>&1
}
FAKE_PRE_COMMIT_LOG=$work/pc.log PATH="$bin2:$PATH" gate_only
check "the pre-commit step skips the hooks that CI runs elsewhere" "$(cat "$work/pc.log")" "SKIP=cargo-clippy,lychee,vale"
mkdir "$work/nopc"
for tool in bash flock mktemp sed git tail grep cat sleep kill rm touch date; do
    ln -s "$(command -v "$tool")" "$work/nopc/$tool"
done
PATH="$work/nopc" gate_only
check "the pre-commit step fails when pre-commit is missing" "$?" 1
check "the pre-commit step says that pre-commit is missing" "$(grep -c '^FAIL: pre-commit is not installed' "$work/runs"/*/pre-commit.log)" 1

cl=$repo/tests/ci-local.sh
nobin=$work/nobin
mkdir "$nobin"
for tool in bash sed mktemp dirname tail rm cat; do
    ln -s "$(command -v "$tool")" "$nobin/$tool"
done
ci_local() {
    PATH="$nobin" TMPDIR="$work/ci-tmp" "$cl" "$@" >"$work/out" 2>&1
}
mkdir "$work/ci-tmp"
ci_local deny machete pre-commit
check "a job whose tool is missing is SKIP and the run exits 0" "$?" 0
check "three missing tools give three SKIP lines" "$(grep -c ' SKIP ' "$work/out")" 3
check "a missing tool is never a FAIL" "$(grep -c ' FAIL ' "$work/out")" 0
ci_local deny nosuch
check "an unknown job is a usage error" "$?" 2
check "an unknown job stops before any job runs" "$(grep -c ' SKIP \| FAIL ' "$work/out")" 0
check "an unknown job leaves no log directory" "$(ls "$work/ci-tmp" | wc -l)" 0
ci_local --help
check "--help leaves no log directory" "$(ls "$work/ci-tmp" | wc -l)" 0
ci_local deny
check "a run without a failure removes its log directory" "$(ls "$work/ci-tmp" | wc -l)" 0

[ "$failures" = 0 ] || {
    echo "FAIL: $failures checks of merge-gate-selftest"
    exit 1
}
echo "ok: merge-gate-selftest"
