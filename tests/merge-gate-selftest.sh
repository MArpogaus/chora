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
[ "$1" != -C ] || shift 2
case $1 in
status)
    [ -z "${FAKE_GIT_DIRTY:-}" ] || echo " M file"
    ;;
rev-parse)
    case $* in
    *--is-shallow-repository*) echo "${FAKE_GIT_SHALLOW:-false}" ;;
    *refs/heads/*)
        [ -n "${FAKE_GIT_LOCAL:-}" ] || exit 1
        echo "$FAKE_GIT_LOCAL"
        ;;
    *) echo "${FAKE_GIT_ORIGIN:-}" ;;
    esac
    ;;
esac
exit 0
FAKE
chmod +x "$work/bin/git"
cat >"$work/bin/df" <<'FAKE'
#!/bin/bash
echo "Filesystem     1024-blocks      Used Available Capacity Mounted on"
echo "/dev/fake      262144000 100000000 ${FAKE_DF_FREE_KB:-100000000}      40% /"
FAKE
chmod +x "$work/bin/df"

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

env -u CHORA_GATE_LOCK_HELD -u MERGE_GATE_STEPS CHORA_GATE_LOCK="$work/gate.lock" MERGE_GATE_NESTED=1 \
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

meet() {
    # shellcheck disable=SC2016 # the shell of the step expands it
    printf '%s|touch %s/%s; for _ in $(seq 50); do [ -e %s/%s ] && exit 0; sleep 0.1; done; exit 1\n' \
        "$1" "$work" "$1" "$work" "$2"
}
rm -f "$work/a" "$work/b"
{
    meet a b
    meet b a
} >"$work/steps"
run_gate --jobs 2
check "--jobs 2 runs two steps at once" "$rc" 0
check "--jobs 2 prints one line per step" "$(grep -cE '^(a|b) +PASS' "$work/out")" 2
rm -f "$work/a" "$work/b"
run_gate
check "without --jobs the steps run one after the other" "$rc" 1
rm -f "$work/a" "$work/b"
{
    meet build test
    meet test build
} >"$work/steps"
run_gate --jobs 2
check "--jobs never runs the cargo steps at once" "$rc" 1
rm -f "$work/2d-guard" "$work/xp"
{
    meet 2d-guard xp
    meet xp 2d-guard
} >"$work/steps"
run_gate --jobs 2
check "--jobs runs 2d-guard alone, before the parallel steps" "$rc" 1
rm -f "$work/third"
printf 'one|exit 1\ntwo|sleep 0.5\nthree|touch %s/third\n' "$work" >"$work/steps"
run_gate --jobs 2 --push session
check "--jobs: a failed step fails the gate" "$rc" 1
check "--jobs: no step starts after a failed step" "$(test -e "$work/third" && echo ran)" ""
check "--jobs: a failed step does not push" "$pushes" 0
check "--jobs: a failed step prints the word FAIL" "$(grep -c '^one .*FAIL' "$work/out")" 1
printf 'one|true\ntwo|true\n' >"$work/steps"
run_gate --jobs 3 --push session
check "--jobs with --push pushes after every step passed" "$pushes" 1
run_gate --jobs 0
check "--jobs 0 is a usage error" "$rc" 2
run_gate --jobs x
check "--jobs x is a usage error" "$rc" 2

rr=$work/rr
mkdir -p "$rr/tests"
cp "$gate" "$rr/tests/merge-gate.sh"
cp "$repo/tests/gate-lock.sh" "$repo/tests/red-first.sh" "$rr/tests/"
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

hook() {
    printf '%s\n' "$@" >"$work/msg"
    (cd "$rr" && RED_FIRST_BASE=base tests/red-first.sh "$work/msg" 2>&1)
}
hook_rc() {
    hook "$@" >/dev/null
    echo $?
}
git -C "$rr" reset -q --hard base
rr_file h.rs h
git -C "$rr" add h.rs
check "the hook refuses a feat commit without a test" "$(hook_rc "feat(h): no test")" 1
check "the hook names the rule" "$(hook "fix(h): no test" | grep -c '^RED-FIRST MISSING: fix(h): no test')" 1
check "the hook takes a No test: line" "$(hook_rc "fix(h): excused" "" "No test: a pure move")" 0
for free in "docs: d" "ci: c" "chore: c" "test(h): t" "refactor(h): r" "Merge claude/x into y"; do
    check "the hook lets '$free' pass without a test" "$(hook_rc "$free")" 0
done
rr_file tests/gate-lib.sh t
git -C "$rr" add tests/gate-lib.sh
check "the hook does not count a staged helper" "$(hook_rc "fix(h): helper only")" 1
rr_file tests/gate-h.sh t
git -C "$rr" add tests/gate-h.sh
check "the hook takes a test staged with the fix" "$(hook_rc "fix(h): with its test")" 0
git -C "$rr" reset -q --hard base
rr_file lib.rs '#[test]
fn f() {}'
git -C "$rr" add lib.rs
check "the hook takes a staged #[test] line" "$(hook_rc "feat(l): with a unit test")" 0
git -C "$rr" reset -q --hard base
rr_file tests/gate-h.sh t
rr_commit "test(h): first" tests/gate-h.sh
rr_file h.rs h
git -C "$rr" add h.rs
check "the hook takes a test commit before it on the branch" "$(hook_rc "fix(h): after the test")" 0
check "the hook reads the subject below the comment lines" "$(hook_rc "# comment" "feat(h): after the test")" 0
git -C "$rr" reset -q --hard base

nb=$work/nb
new_hooked_repo() {
    rm -rf "$1"
    git init -q -b main "$1"
    git -C "$1" config user.email t@t
    git -C "$1" config user.name t
    # shellcheck disable=SC2016 # the hook expands its own argument
    printf '#!/bin/bash\nexec %q "$1"\n' "$repo/tests/red-first.sh" >"$1/.git/hooks/commit-msg"
    chmod +x "$1/.git/hooks/commit-msg"
}
nb_commit() {
    local msg=$1
    shift
    for path in "$@"; do
        mkdir -p "$(dirname "$nb/$path")"
        echo "$msg" >>"$nb/$path"
    done
    git -C "$nb" add -- "$@"
    env -u RED_FIRST_BASE git -C "$nb" commit -q -m "$msg" >"$work/nb.out" 2>&1
    echo $?
}
new_hooked_repo "$nb"
check "no base: an init commit passes" "$(nb_commit "chore: init" README)" 0
check "no base: a test commit passes" "$(nb_commit "test(z): red" tests/gate-z.sh)" 0
check "no base: a fix after its test commit passes" "$(nb_commit "fix(z): green" z.rs)" 0
env -u RED_FIRST_BASE git -C "$nb" commit -q --amend -m "fix(z): green, amended" >"$work/nb.out" 2>&1
check "no base: an amend of that fix passes" "$?" 0
(cd "$nb" && env -u RED_FIRST_BASE GIT_SEQUENCE_EDITOR='sed -i 1s/^pick/reword/' \
    GIT_EDITOR="sed -i '1s/.*/fix(z): reworded/'" git rebase -q -i HEAD~1 >"$work/nb.out" 2>&1)
check "no base: a rebase reword of that fix passes" "$?:$(git -C "$nb" log -1 --format=%s)" "0:fix(z): reworded"
env -u RED_FIRST_BASE git -C "$nb" commit -q --fixup HEAD --allow-empty >"$work/nb.out" 2>&1
check "no base: a fixup commit passes" "$?" 0
env -u RED_FIRST_BASE git -C "$nb" revert --no-edit HEAD~1 >"$work/nb.out" 2>&1
check "no base: a revert passes" "$?" 0
git -C "$nb" checkout -q -b side
check "no base: a side commit passes" "$(nb_commit "chore: side" side.txt)" 0
git -C "$nb" checkout -q main
env -u RED_FIRST_BASE git -C "$nb" merge -q --no-ff side -m "fix(m): a merge" >"$work/nb.out" 2>&1
check "no base: a merge with a fix subject passes" "$?" 0
cl=$work/clone
rm -rf "$cl"
git clone -q --depth 1 "file://$nb" "$cl"
git -C "$cl" config user.email t@t
git -C "$cl" config user.name t
cp "$nb/.git/hooks/commit-msg" "$cl/.git/hooks/"
echo y >"$cl/y.rs"
git -C "$cl" add y.rs
env -u RED_FIRST_BASE git -C "$cl" commit -q -m "fix(y): in a shallow clone" >"$work/nb.out" 2>&1
check "a shallow clone with nothing unpushed lets a fix pass" "$?" 0
check "the pass without a base names the merge gate" "$(grep -c '^red-first: .*merge gate' "$work/nb.out")" 1
new_hooked_repo "$nb"
check "unborn HEAD: a first feat commit passes" "$(nb_commit "feat(z): init" z.rs)" 0
check "unborn HEAD: the pass names the merge gate" "$(grep -c '^red-first: .*merge gate' "$work/nb.out")" 1
check "no base: a fix without any test on the branch passes" "$(nb_commit "fix(z): no test" z.rs)" 0
check "no base: that pass names the merge gate" "$(grep -c '^red-first: .*merge gate' "$work/nb.out")" 1
new_hooked_repo "$nb"
nb_commit "chore: init" README >/dev/null
bare=$work/origin.git
git init -q --bare "$bare"
git -C "$nb" push -q "$bare" main:work 2>/dev/null
nb=$work/single
git clone -q --depth 1 --single-branch --branch work "file://$bare" "$nb"
git -C "$nb" config user.email t@t
git -C "$nb" config user.name t
cp "$work/nb/.git/hooks/commit-msg" "$nb/.git/hooks/"
nb_commit "test(y): red" tests/gate-y.sh >/dev/null
git -C "$nb" push -q origin HEAD:work 2>/dev/null
nb_commit "docs(y): note" y.md >/dev/null
check "single-branch clone: a fix after a pushed test and a docs commit passes" "$(nb_commit "fix(y): green" y.rs)" 0
check "single-branch clone: that pass names the merge gate" "$(grep -c '^red-first: .*merge gate' "$work/nb.out")" 1
session=claude/autonomous-dev-session-3w-i33pg7
git -C "$work/nb" push -q "$bare" "main:$session" 2>/dev/null
nb=$work/session
git clone -q --branch "$session" "file://$bare" "$nb"
git -C "$nb" config user.email t@t
git -C "$nb" config user.name t
cp "$work/nb/.git/hooks/commit-msg" "$nb/.git/hooks/"
nb_commit "test(s): red" tests/gate-s.sh >/dev/null
git -C "$nb" push -q origin "HEAD:$session" 2>/dev/null
git -C "$nb" fetch -q origin
check "base branch: a fix after its pushed test passes" "$(nb_commit "fix(s): green" s.rs)" 0
check "base branch: that pass names the merge gate" "$(grep -c '^red-first: .*merge gate' "$work/nb.out")" 1
git -C "$nb" checkout -q -b claude/lane "origin/$session"
check "lane branch: a fix without a test on the lane is refused" "$(nb_commit "fix(s): lane" s.rs)" 1
nb=$work/nb

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
for tool in bash flock mktemp sed git tail grep cat sleep kill rm touch date dirname mkdir true sh env df awk timeout; do
    ln -s "$(command -v "$tool")" "$work/nopc/$tool"
done
rm -rf "$work/runs"
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
check "an unknown job leaves no log directory" "$(find "$work/ci-tmp" -mindepth 1 | wc -l)" 0
ci_local --help
check "--help leaves no log directory" "$(find "$work/ci-tmp" -mindepth 1 | wc -l)" 0
ci_local deny
check "a run without a failure removes its log directory" "$(find "$work/ci-tmp" -mindepth 1 | wc -l)" 0

runtime_case() {
    rm -rf "$work/plugins"
    mkdir "$work/plugins"
    printf '%b' "$1" >"$work/plugins/lib.rs"
    CHORA_GATE_LOCK_HELD=1 "$repo/tests/gate-0.8-plugin-runtime.sh" "$work/plugins" >/dev/null 2>&1
    echo $?
}
allowed_fn='#[allow(\n    clippy::disallowed_methods,\n    reason = "fake core"\n)]\npub fn fake() {\n    let c = conn::connect(p);\n}\n'
check "plugin runtime: a hit inside the allowed item passes" "$(runtime_case "$allowed_fn")" 0
check "plugin runtime: a hit without an allow fails" "$(runtime_case 'fn a() {\n    conn::connect(p);\n}\n')" 1
check "plugin runtime: a hit in the item after the allowed one fails" \
    "$(runtime_case "${allowed_fn}\nfn other() {\n    conn::connect(q);\n}\n")" 1
check "plugin runtime: an allowed statement covers only itself" \
    "$(runtime_case 'fn a() {\n    #[allow(clippy::disallowed_methods, reason = "x")]\n    let c = conn::connect(\n        p,\n    );\n    let d = conn::connect(q);\n}\n')" 1
check "plugin runtime: an allowed multi-line statement passes" \
    "$(runtime_case 'fn a() {\n    #[allow(clippy::disallowed_methods, reason = "x")]\n    let c = conn::connect(\n        p,\n    );\n}\n')" 0
check "plugin runtime: a comment naming the lint opens no allow" \
    "$(runtime_case 'fn a() {\n    // Tests here may not use clippy::disallowed_methods.\n    core.call("events.subscribe");\n}\n')" 1
check "plugin runtime: a string naming the lint opens no allow" \
    "$(runtime_case 'const L: &str = "clippy::disallowed_methods";\nfn a() {\n    conn::connect(p);\n}\n')" 1
check "plugin runtime: an allow over several lines passes" \
    "$(runtime_case 'fn a() {\n    #[allow(\n        clippy::disallowed_methods,\n        reason = "x"\n    )]\n    let c = conn::connect(p);\n}\n')" 0
check "plugin runtime: an allow with a trailing comment hides nothing after it" \
    "$(runtime_case 'fn a() {\n    #[allow(clippy::cast_precision_loss)] // a few items\n    let x = 1;\n}\nfn b() {\n    core.call("events.subscribe");\n}\n')" 1
check "plugin runtime: an allowed item with a trailing comment covers only itself" \
    "$(runtime_case '#[allow(clippy::disallowed_methods)] // fake core\nfn a() {\n    conn::connect(p);\n}\nfn b() {\n    core.call("events.subscribe");\n}\n')" 1
check "plugin runtime: an allowed item with a trailing comment passes" \
    "$(runtime_case '#[allow(clippy::disallowed_methods)] // fake core\nfn a() {\n    conn::connect(p);\n}\n')" 0

idr=$work/idr
good="Good One <good@example.org>"
mkdir -p "$idr/tests"
cp "$gate" "$idr/tests/merge-gate.sh"
cp "$repo/tests/gate-lock.sh" "$repo/tests/red-first.sh" "$idr/tests/"
git -C "$idr" init -q -b main
idcommit() {
    GIT_AUTHOR_NAME=${1%% <*} GIT_AUTHOR_EMAIL=${1##*<} GIT_COMMITTER_NAME=${2%% <*} GIT_COMMITTER_EMAIL=${2##*<} \
        git -C "$idr" commit -q --allow-empty -m "$3"
}
idcommit "Old One <old@example.org>" "Old One <old@example.org>" "chore: before the base"
git -C "$idr" tag base
ident() {
    MERGE_GATE_IDENTITY="Good One <good@example.org>" "$idr/tests/merge-gate.sh" --identity-check --base base 2>&1
}
idcommit "$good" "$good" "chore: all good"
check "identity: every commit with the expected identity is green" "$(ident >/dev/null; echo $?)" 0
idcommit "$good" "Claude <noreply@anthropic.com>" "chore: bad committer"
bad=$(git -C "$idr" rev-parse --short HEAD)
check "identity: a bad committer is red" "$(ident >/dev/null; echo $?)" 1
check "identity: the failure names the sha and the identity" \
    "$(ident | grep -c "^FAIL: $bad committer is Claude <noreply@anthropic.com>")" 1
git -C "$idr" reset -q --hard HEAD~1
idcommit "Claude <noreply@anthropic.com>" "$good" "chore: bad author"
check "identity: a bad author is red" "$(ident >/dev/null; echo $?)" 1
check "identity: the failure says author" "$(ident | grep -c ' author is Claude <noreply@anthropic.com>')" 1
git -C "$idr" reset -q --hard HEAD~1
git -C "$idr" checkout -q -b side base
idcommit "$good" "$good" "chore: on the side"
git -C "$idr" checkout -q main
GIT_COMMITTER_NAME=Claude GIT_COMMITTER_EMAIL=noreply@anthropic.com GIT_AUTHOR_NAME=Good\ One GIT_AUTHOR_EMAIL=good@example.org \
    git -C "$idr" merge -q --no-ff side -m "Merge side"
check "identity: a merge commit with a bad committer is red" "$(ident >/dev/null; echo $?)" 1
git -C "$idr" reset -q --hard HEAD~1
check "identity: a bad commit before the base does not count" "$(ident >/dev/null; echo $?)" 0
check "identity: the step follows red-first in the step list" \
    "$(grep -A1 '^red-first|' "$gate" | grep -c '^identity|')" 1
check "identity: the override is printed when MERGE_GATE_IDENTITY is set" \
    "$(ident | grep -c "^identity: MERGE_GATE_IDENTITY overrides the expected identity: $good\$")" 1
check "identity: no override line without MERGE_GATE_IDENTITY" \
    "$(env -u MERGE_GATE_IDENTITY "$idr/tests/merge-gate.sh" --identity-check --base base 2>&1 | grep -c 'overrides the expected identity')" 0
printf 'one|touch %s/ran\n' "$work" >"$work/steps"
rm -f "$work/ran"
FAKE_DF_FREE_KB=5242880 run_gate
check "df: 5 GB free on / fails the gate" "$rc" 1
check "df: the gate names the free space and the need" "$(grep -c 'only 5 GB free on /, 6 GB are needed' "$work/out")" 1
check "df: the gate prints df" "$(grep -c '^/dev/fake ' "$work/out")" 1
check "df: no step runs under the limit" "$(test -e "$work/ran" && echo ran)" ""
FAKE_DF_FREE_KB=6291456 run_gate
check "df: exactly 6 GB free passes" "$rc" 0
check "df: a pass prints no df line" "$(grep -c 'GB free on /' "$work/out")" 0
FAKE_DF_FREE_KB=5242880 run_gate --list
check "df: --list needs no free space" "$rc" 0
rm -f "$work/ran"
FAKE_GIT_SHALLOW=true run_gate
check "shallow: a shallow clone fails the gate" "$rc" 1
check "shallow: the gate says how to deepen it" "$(grep -c 'shallow clone; run git fetch --unshallow' "$work/out")" 1
check "shallow: no step runs in a shallow clone" "$(test -e "$work/ran" && echo ran)" ""
FAKE_GIT_SHALLOW=true run_gate --list
check "shallow: --list needs no full history" "$rc" 0
FAKE_GIT_LOCAL=aaaaaaaa11111111 FAKE_GIT_ORIGIN=bbbbbbbb22222222 run_gate
check "drift: a session branch that differs from origin warns" "$(grep -c 'WARNING: drift: claude/autonomous-dev-session-3w-i33pg7 is at aaaaaaaa, origin/claude/autonomous-dev-session-3w-i33pg7 at bbbbbbbb' "$work/out")" 1
check "drift: the warning does not fail the gate" "$rc" 0
check "drift: the gate fetches the branch first" "$(grep -c ' fetch -q origin claude/autonomous-dev-session-3w-i33pg7$' "$work/git.log")" 1
FAKE_GIT_LOCAL=aaaaaaaa11111111 FAKE_GIT_ORIGIN=aaaaaaaa11111111 run_gate
check "drift: equal heads print no warning" "$(grep -c 'WARNING' "$work/out")" 0
FAKE_GIT_ORIGIN=bbbbbbbb22222222 run_gate
check "drift: no local session branch prints no warning" "$(grep -c 'WARNING' "$work/out")" 0
check "gate-lib: the step runs merge-gate-selftest.sh" \
    "$(grep '^gate-lib|' "$gate" | grep -c 'tests/merge-gate-selftest.sh')" 1
check "gate-lib: the step runs cells-for-selftest.sh" \
    "$(grep '^gate-lib|' "$gate" | grep -c 'tests/cells-for-selftest.sh')" 1

printf 'one|true\nlauncher|echo "SKIP: fuzzel 1.9.2 < 1.11"\nthree|echo "ok: three"\n' >"$work/steps"
run_gate
check "skip: a run with a skipped step exits 0" "$rc" 0
check "skip: the step line says SKIP, not PASS" "$(grep -c '^launcher *SKIP ' "$work/out")" 1
check "skip: a step without a SKIP line still says PASS" "$(grep -c '^three *PASS ' "$work/out")" 1
check "skip: the summary names the cell and its reason" "$(grep -c '^skipped: launcher (fuzzel 1.9.2 < 1.11)$' "$work/out")" 1
run_gate --push session
check "skip: --push refuses a run with a skipped step" "$rc" 1
check "skip: --push with a skipped step does not push" "$pushes" 0
check "skip: the refusal names the cell and --allow-skip" "$(grep -c 'launcher.*--allow-skip' "$work/out")" 1
run_gate --push session --allow-skip other
check "skip: --allow-skip of another cell still refuses" "$rc" 1
check "skip: --allow-skip of another cell does not push" "$pushes" 0
run_gate --push session --allow-skip launcher
check "skip: --allow-skip of the cell lets the push through" "$rc" 0
check "skip: --allow-skip of the cell pushes once" "$pushes" 1
check "skip: an allowed skip is still listed" "$(grep -c '^skipped: launcher' "$work/out")" 1
run_gate --push session --allow-skip other,launcher
check "skip: --allow-skip takes a list" "$pushes" 1
run_gate --only one,launcher
check "skip: a partial run lists the skip too" "$(grep -c '^skipped: launcher' "$work/out")" 1
printf 'one|true\nfuzz|echo "SKIP: cell a: x"; echo "SKIP: cell b: y"\n' >"$work/steps"
run_gate
check "skip: two SKIP lines of one step give one SKIP step line" "$(grep -c '^fuzz *SKIP ' "$work/out")" 1
check "skip: the summary gives both reasons of the step" "$(grep -c '^skipped: fuzz (cell a: x; cell b: y)$' "$work/out")" 1
printf 'one|true\nbad|echo "SKIP: x"; exit 1\n' >"$work/steps"
run_gate
check "skip: a step that skips and fails is FAIL" "$(grep -c '^bad *FAIL ' "$work/out")" 1
check "skip: the header states the rule" "$([ "$(sed -n '1,/^set -u/p' "$gate" | grep -c -- '--allow-skip')" -ge 2 ] && echo yes)" yes
printf 'one|true\n' >"$work/steps"
run_gate --allow-skip
check "skip: --allow-skip needs a list" "$rc" 2

check "ci-check: the step runs before build in the step list" \
    "$("$gate" --list | grep -nx -e ci-check -e build | head -1 | cut -d: -f2)" ci-check
check "ci-check: tools/cut-check is the first command of the step" \
    "$(grep '^ci-check|' "$gate" | cut -d'|' -f2 | grep -c '^tools/cut-check ')" 1

[ "$failures" = 0 ] || {
    echo "FAIL: $failures checks of merge-gate-selftest"
    exit 1
}
echo "ok: merge-gate-selftest"
