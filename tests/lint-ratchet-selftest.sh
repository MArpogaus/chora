#!/bin/sh
# Self-test of tools/lint-ratchet: over a fixture tree the three counts say
# what their names say (a rustfmt-wrapped attribute with its reason on a later
# line is not without reason), a planted new allow, no-reason or eprintln
# fails, a lower count asks to lower the list, the help names the rule of the
# list.
#
# Usage: tests/lint-ratchet-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
tool=$repo/tools/lint-ratchet
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

fail() {
    echo "FAIL: $*"
    failures=$((failures + 1))
}
listed() { awk -v k="$1" '$1 == k { print $2 }' "$work/repo/tools/lint-ratchet.list"; }
check_count() {
    got=$(listed "$1")
    [ "$got" = "$2" ] || fail "$label: $1 listed $got, want $2"
}
want_list() {
    check_count allow "$1"
    check_count no-reason "$2"
    check_count eprintln "$3"
}

mkdir -p "$work/repo/crates/chora/src" "$work/repo/plugins/p/src" "$work/repo/tools" || exit 2
cd "$work/repo" || exit 2
cat >crates/chora/src/lib.rs <<'RS'
#[allow(clippy::too_many_arguments, reason = "one-line, with a reason")]
fn one() {}

#[expect(
    clippy::cognitive_complexity,
    reason = "wrapped by rustfmt, the reason is on a later line"
)]
fn wrapped_with_reason() {}

#[allow(clippy::needless_range_loop)]
fn no_reason() {}

fn print() {
    eprintln!("one eprintln");
}
RS

label="fixture tree"
"$tool" --root "$work/repo" --update || fail "$label: --update failed"
want_list 3 1 1
"$tool" --root "$work/repo" >"$work/out" 2>&1 || fail "$label: the tree fails its own list: $(tail -2 "$work/out")"

label="wrapped attribute only"
cat >crates/chora/src/lib.rs <<'RS'
#[expect(
    clippy::cognitive_complexity,
    reason = "wrapped"
)]
fn f() {}
RS
"$tool" --root "$work/repo" --update
want_list 1 0 0

label="breach"
src=$(cat crates/chora/src/lib.rs)
printf '#[allow(dead_code)]\nfn g() {}\n' >>crates/chora/src/lib.rs
"$tool" --root "$work/repo" >"$work/out" 2>&1 && fail "$label: a new allow without reason passed"
grep -q '^FAIL: lint-ratchet: allow is 2, the list holds 1' "$work/out" || fail "$label: no allow FAIL line: $(tail -3 "$work/out")"
grep -q '^FAIL: lint-ratchet: no-reason is 1, the list holds 0' "$work/out" || fail "$label: no no-reason FAIL line: $(tail -3 "$work/out")"
printf '%s\n' "$src" >crates/chora/src/lib.rs
printf '#[allow(dead_code, reason = "new")]\nfn h() {}\n' >>crates/chora/src/lib.rs
"$tool" --root "$work/repo" >"$work/out" 2>&1 && fail "$label: a new allow with a reason passed without raising the list"
grep -q '^FAIL: lint-ratchet: allow is 2, the list holds 1' "$work/out" || fail "$label: no allow FAIL line for the reasoned allow: $(tail -3 "$work/out")"
grep -q '^FAIL: lint-ratchet: no-reason' "$work/out" && fail "$label: a reasoned allow breached no-reason"
sed -i 's/^allow 1$/allow 2/' tools/lint-ratchet.list
"$tool" --root "$work/repo" >"$work/out" 2>&1 || fail "$label: a reasoned allow failed with the list raised: $(tail -3 "$work/out")"
printf '%s\n' "$src" >crates/chora/src/lib.rs
printf 'fn p() { eprintln!("x"); }\n' >>crates/chora/src/lib.rs
"$tool" --root "$work/repo" >"$work/out" 2>&1 && fail "$label: a new eprintln passed"
grep -q '^FAIL: lint-ratchet: eprintln is 1, the list holds 0' "$work/out" || fail "$label: no eprintln FAIL line: $(tail -3 "$work/out")"
printf 'fn q() {}\n' >crates/chora/src/lib.rs
"$tool" --root "$work/repo" >"$work/out" 2>&1 || fail "$label: a fall failed"
grep -q 'lower the list: allow is 0, listed 2' "$work/out" || fail "$label: a fall does not ask to lower the list"

label="breach names its cause"
rm -rf crates plugins
mkdir -p crates/chora/src
for n in 1 2 3 4 5 6 7; do
    i=0
    while [ "$i" -lt "$n" ]; do
        echo "fn e${n}_$i() { eprintln!(\"x\"); }" >>"crates/chora/src/f$n.rs"
        i=$((i + 1))
    done
done
"$tool" --root "$work/repo" >"$work/out" 2>&1 && fail "$label: 28 eprintln over a list of 0 passed"
cause=$(grep '^  eprintln: list ' "$work/out" || true)
case $cause in
'  eprintln: list 0, tree 28; most in: '*) ;;
*) fail "$label: no cause line (counter, list, tree): $(cat "$work/out")" ;;
esac
for n in 7 6 5 4 3; do
    case $cause in *"crates/chora/src/f$n.rs ($n)"*) ;; *) fail "$label: the top files miss f$n.rs ($n): $cause" ;; esac
done
for n in 2 1; do
    case $cause in *"crates/chora/src/f$n.rs"*) fail "$label: the top 5 holds f$n.rs: $cause" ;; esac
done

label="help"
"$tool" --help >"$work/out"
tr '\n' ' ' <"$work/out" | grep -q 'raises .allow. in the list in the same commit' || fail "$label: the help does not say a reasoned new allow raises the list: $(cat "$work/out")"

[ "$failures" = 0 ] && echo "ok: lint-ratchet-selftest"
exit "$failures"
