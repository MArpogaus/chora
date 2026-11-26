#!/bin/bash
# Self-test of the red-first run (tests/merge-gate.sh --red-first-run): in a
# temp repo with a tiny crate, a fix whose test already passes on the fix's
# parent is red, a test that fails there is green, a gate cell is named and
# not run, `No test:` opts out, and the gate's default step passes the flag.
#
# Usage: tests/red-first-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
check() {
    if [ "$2" = "$3" ]; then
        echo "ok: $1"
    else
        echo "FAIL: $1 (got '$2', want '$3')"
        failures=$((failures + 1))
    fi
}

rr=$work/rr
export CARGO_TARGET_DIR=$work/target
export CARGO_NET_OFFLINE=true
mkdir -p "$rr/tests" "$rr/src"
cp "$repo/tests/merge-gate.sh" "$repo/tests/gate-lock.sh" "$repo/tests/red-first.sh" "$rr/tests/"
git -C "$rr" init -q -b main
git -C "$rr" config user.email t@t
git -C "$rr" config user.name t
printf '[package]\nname = "rfx"\nversion = "0.1.0"\nedition = "2021"\n' >"$rr/Cargo.toml"

rr_commit() {
    git -C "$rr" add -A
    git -C "$rr" commit -q -m "$@"
}
add_src() {
    printf 'pub fn add(a: i32, b: i32) -> i32 {\n    %s\n}\n' "$1" >"$rr/src/lib.rs"
}
add_test() {
    printf '\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {\n        assert_eq!(super::add(1, 2), 3);\n    }\n}\n' >>"$rr/src/lib.rs"
}
red() { (cd "$rr" && tests/merge-gate.sh --red-first-check --red-first-run --base base 2>&1); }
fresh() {
    git -C "$rr" reset -q --hard base
}

add_src 'a + b'
rr_commit "chore: init"
git -C "$rr" tag base

fresh
add_src 'a + b'
add_test
rr_commit "test(x): adds is 3 (sh-9)"
add_src 'a + b + 0'
rr_commit "fix(x): add stays right (sh-9)"
fix=$(git -C "$rr" rev-parse --short HEAD)
out=$(red)
check "a test that passes on the fix's parent is red" "$(red >/dev/null; echo $?)" 1
check "that red names the fix and the test" "$(grep -c "^red-first: $fix .*adds.* passes on the parent" <<<"$out")" 1

fresh
add_src 'a - b'
add_test
rr_commit "test(x): adds is 3 (sh-9, red)"
add_src 'a + b'
rr_commit "fix(x): add adds (sh-9)"
fix=$(git -C "$rr" rev-parse --short HEAD)
out=$(red)
check "a test that fails on the fix's parent is green" "$(red >/dev/null; echo $?)" 0
check "that green names the red test" "$(grep -c "^red-first: $fix .*adds.* red on the parent" <<<"$out")" 1

fresh
add_src 'a + b'
add_test
rr_commit "test(x): adds is 3 (sh-9)"
add_src 'a + b + 0'
git -C "$rr" add -A
git -C "$rr" commit -q -m "fix(x): add stays right (sh-9)" -m "No test: a rename"
check "a body line No test: opts out of the run" "$(red >/dev/null; echo $?)" 0

fresh
printf '#!/bin/bash\nexit 0\n' >"$rr/tests/gate-z.sh"
rr_commit "test(x): a gate cell (sh-9)"
add_src 'a + b + 0'
rr_commit "fix(x): add stays right (sh-9)"
fix=$(git -C "$rr" rev-parse --short HEAD)
out=$(red)
check "a test only in a gate script is not run and passes" "$(red >/dev/null; echo $?)" 0
check "that pass names the gate cell" "$(grep -c "^red-first: $fix gate cell, not run" <<<"$out")" 1

fresh
add_src 'a + b + 0'
add_test
rr_commit "fix(x): add stays right and its test"
fix=$(git -C "$rr" rev-parse --short HEAD)
out=$(red)
check "a test in the fix commit itself is named, not run" "$(grep -c "^red-first: $fix test in the fix commit, not run" <<<"$out")" 1

fresh
add_src 'a + b'
add_test
rr_commit "test(x): adds is 3 (sh-9)"
add_src 'a + b + 0'
rr_commit "fix(x): add stays right (sh-9)"
fresh
add_src 'a - b'
add_test
rr_commit "test(x): adds is 3 (sh-9, red)"
add_src 'a + b'
rr_commit "fix(x): add adds (sh-9)"
add_src 'a + b + 0'
printf '#!/bin/bash\nexit 0\n' >"$rr/tests/gate-z.sh"
rr_commit "fix(x): add stays right (sh-9)"
fix=$(git -C "$rr" rev-parse --short HEAD)
out=$(red)
check "a second fix of one id does not reuse the test of the first" "$(grep -c 'NOT RED' <<<"$out")" 0
check "that second fix names its own gate cell" "$(grep -c "^red-first: $fix gate cell, not run" <<<"$out")" 1

check "without --red-first-run the passing test is not run" "$(cd "$rr" && tests/merge-gate.sh --red-first-check --base base >/dev/null 2>&1; echo $?)" 0

check "the gate's red-first step passes --red-first-run" "$("$repo/tests/merge-gate.sh" --commands | grep -c '^red-first|.*--red-first-run')" 1

[ "$failures" = 0 ] || {
    echo "FAIL: $failures checks of red-first-selftest"
    exit 1
}
echo "ok: red-first-selftest"
