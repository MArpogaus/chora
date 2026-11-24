#!/bin/sh
# Self-test of tools/sleep-ratchet: over a fixture tree, a count above the
# list fails and names the list value, the tree value and the 5 files that add
# the most; a count below only asks to lower the list; --update writes the
# count of the tree.
#
# The fixture lines are built from two halves so that this file adds no line to
# the count it tests.
#
# Usage: tests/sleep-ratchet-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
tool=$repo/tools/sleep-ratchet
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
fail() {
    echo "FAIL: $*"
    failures=$((failures + 1))
}
word=sl$(printf ee)p
line() { printf '    %s %s\n' "$word" "$1"; }

mkdir -p "$work/repo/tests" "$work/repo/tools" || exit 2
cd "$work/repo" || exit 2

label="count"
{
    echo '#!/bin/sh'
    line 1
    echo "echo ${word}ing"
    echo "x=\$(${word} 2)"
    echo "my_${word} 3"
} >tests/a.sh
printf '%s\n' "$(line 0.2)" >tests/b.sh
"$tool" --root "$work/repo" --update || fail "$label: --update failed"
got=$(awk '$1 == "sleep" { print $2 }' tools/sleep-ratchet.list 2>/dev/null)
[ "$got" = 3 ] || fail "$label: listed '$got', want 3 (line, command substitution, second file)"
"$tool" --root "$work/repo" >"$work/out" 2>&1 || fail "$label: the tree fails its own list: $(tail -2 "$work/out")"

label="over the list"
line 5 >>tests/b.sh
"$tool" --root "$work/repo" >"$work/out" 2>&1 && fail "$label: one more line passed"
grep -q '^FAIL: sleep-ratchet: ' "$work/out" || fail "$label: no FAIL line: $(cat "$work/out")"
grep -q 'list 3, tree 4; most in: tests/a.sh (2), tests/b.sh (2)' "$work/out" ||
    grep -q 'list 3, tree 4; most in: tests/b.sh (2), tests/a.sh (2)' "$work/out" ||
    fail "$label: no cause line with list, tree and files: $(cat "$work/out")"

label="top 5"
rm -f tests/*.sh
n=1
while [ "$n" -le 7 ]; do
    i=0
    while [ "$i" -lt "$n" ]; do
        line "$i" >>"tests/f$n.sh"
        i=$((i + 1))
    done
    n=$((n + 1))
done
line 0 >tests/g.sh
"$tool" --root "$work/repo" >"$work/out" 2>&1 && fail "$label: 29 lines over a list of 3 passed"
cause=$(grep "^  ${word}: list " "$work/out" || true)
case $cause in
"  ${word}: list 3, tree 29; most in: "*) ;;
*) fail "$label: no cause line: $(cat "$work/out")" ;;
esac
for n in 7 6 5 4 3; do
    case $cause in *"tests/f$n.sh ($n)"*) ;; *) fail "$label: the top files miss f$n.sh: $cause" ;; esac
done
case $cause in *"tests/f2.sh"* | *"tests/f1.sh"*) fail "$label: the top 5 holds a small file: $cause" ;; esac

label="under the list"
rm -f tests/*.sh
line 1 >tests/a.sh
"$tool" --root "$work/repo" >"$work/out" 2>&1 || fail "$label: a fall failed: $(cat "$work/out")"
grep -q "lower the list: ${word} is 1, listed 3; run tools/sleep-ratchet --update" "$work/out" ||
    fail "$label: a fall does not ask to lower the list: $(cat "$work/out")"

label="update does not raise"
rm -rf tests
mkdir tests
line 1 >tests/a.sh
"$tool" --root "$work/repo" --update >/dev/null 2>&1 || fail "$label: a fall or equal count failed"
line 2 >>tests/a.sh
"$tool" --root "$work/repo" --update >"$work/out" 2>&1 && fail "$label: --update raised the list"
grep -q 'ratchet-raise' "$work/out" || fail "$label: no hint to raise by hand: $(cat "$work/out")"
got=$(awk '$1 == "sleep" { print $2 }' tools/sleep-ratchet.list)
[ "$got" = 1 ] || fail "$label: the list holds '$got', want 1 (unchanged)"

label="sub and python"
rm -rf tests tools/sleep-ratchet.list
mkdir -p tests/sub
line 1 >tests/a.sh
line 2 >tests/sub/x.sh
printf '    time.%s(0.5)\n' "$word" >tests/b.sh
"$tool" --root "$work/repo" --update >/dev/null 2>&1 || fail "$label: --update failed"
got=$(awk '$1 == "sleep" { print $2 }' tools/sleep-ratchet.list)
[ "$got" = 1 ] || fail "$label: sleep is '$got', want 1 (the audit count, tests/*.sh only)"
got=$(awk '$1 == "sleep-all" { print $2 }' tools/sleep-ratchet.list)
[ "$got" = 3 ] || fail "$label: sleep-all is '$got', want 3 (tests/*.sh, a subdirectory, time.sleep)"
"$tool" --root "$work/repo" >"$work/out" 2>&1 || fail "$label: the tree fails its own list"
line 3 >>tests/sub/x.sh
"$tool" --root "$work/repo" >"$work/out" 2>&1 && fail "$label: a new wait in a subdirectory passed"
grep -q 'sleep-all: list 3, tree 4; most in: tests/sub/x.sh (3)' "$work/out" || fail "$label: no cause line: $(cat "$work/out")"

[ "$failures" = 0 ] && echo "ok: sleep-ratchet-selftest"
exit "$failures"
