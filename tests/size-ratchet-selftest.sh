#!/bin/bash
# Self-test of tools/size-ratchet: a planted growth of a listed file fails,
# an extraction passes, a new file over the limit fails, a test module does
# not count.
#
# Usage: tests/size-ratchet-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
tool=$repo/tools/size-ratchet
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

fail() {
    echo "FAIL: $*"
    failures=$((failures + 1))
}
expect() {
    local want=$1 label=$2
    shift 2
    "$@" >"$work/out" 2>&1
    local got=$?
    [ "$got" = "$want" ] || fail "$label: exit $got, want $want: $(tail -3 "$work/out")"
}
expect_fail() {
    local label=$1 pattern=$2
    shift 2
    "$@" >"$work/out" 2>&1
    local got=$?
    [ "$got" = 1 ] || fail "$label: exit $got, want 1: $(tail -3 "$work/out")"
    grep -q "^FAIL: size-ratchet: $pattern" "$work/out" || fail "$label: no FAIL line matching '$pattern': $(tail -3 "$work/out")"
}
lines() { seq 1 "$1" | sed 's/.*/let _x = 1;/'; }

mkdir -p "$work/repo/crates/a/src" && cd "$work/repo" || exit 2
git init -q . && git config user.email t@t && git config user.name t
{ lines 1600; printf '#[cfg(test)]\nmod tests {\n'; lines 900; printf '}\n'; } >crates/a/src/big.rs
lines 100 >crates/a/src/small.rs
"$tool" --root "$work/repo" --update
git add -A && git commit -qm base && base=$(git rev-parse HEAD)

expect 0 "head as listed" "$tool" --root "$work/repo" --base "$base"
grep -q '^1600 crates/a/src/big.rs$' tools/size-ratchet.list || fail "list does not hold 1600 production lines of big.rs"

{ lines 1601; printf '#[cfg(test)]\nmod tests {\n'; lines 900; printf '}\n'; } >crates/a/src/big.rs
expect_fail "planted growth" '.*big.rs: grew from 1600 to 1601' "$tool" --root "$work/repo" --base "$base"

sed -i 's/^1600 /1700 /' tools/size-ratchet.list
expect_fail "growth with the list raised" '.*big.rs: grew' "$tool" --root "$work/repo" --base "$base"
git checkout -q tools/size-ratchet.list crates/a/src/big.rs

sed -i 's/^1600 /1700 /' tools/size-ratchet.list
expect 0 "stale list entry above the base" "$tool" --root "$work/repo" --base "$base"
git checkout -q tools/size-ratchet.list

{ lines 1200; printf '#[cfg(test)]\nmod tests {\n'; lines 900; printf '}\n'; } >crates/a/src/big.rs
lines 400 >crates/a/src/extracted.rs
expect 0 "extraction" "$tool" --root "$work/repo" --base "$base"
grep -q 'lower the list' "$work/out" || fail "extraction does not ask to lower the list"
git checkout -q crates/a/src/big.rs && rm crates/a/src/extracted.rs

lines 1501 >crates/a/src/new.rs
expect_fail "new file over the limit" '.*new.rs: 1501 production lines, over 1500 and not listed' "$tool" --root "$work/repo" --base "$base"
rm crates/a/src/new.rs

lines 2000 >crates/a/src/tests.rs
mkdir -p crates/a/tests && lines 2000 >crates/a/tests/it.rs
expect 0 "test files do not count" "$tool" --root "$work/repo" --base "$base"

lines 1700 >crates/a/src/aaa.rs
lines 1800 >crates/a/src/zzz.rs
"$tool" --root "$work/repo" --update
paths=$(grep -v '^#' tools/size-ratchet.list | awk '{print $2}')
[ "$paths" = "$(printf '%s\n' "$paths" | LC_ALL=C sort)" ] || fail "list is not sorted by path: $paths"
cp tools/size-ratchet.list "$work/before"
"$tool" --root "$work/repo" --update
cmp -s tools/size-ratchet.list "$work/before" || fail "--update of an unchanged tree changed the list"
lines 1750 >crates/a/src/aaa.rs
"$tool" --root "$work/repo" --update
changed=$(diff "$work/before" tools/size-ratchet.list | grep -c '^[<>]')
[ "$changed" = 2 ] || fail "--update changed $changed lines for one changed file, want 2"
rm crates/a/src/aaa.rs crates/a/src/zzz.rs

[ "$failures" = 0 ] || exit 1
echo "ok: size-ratchet-selftest"
