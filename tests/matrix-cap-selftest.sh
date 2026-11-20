#!/bin/bash
# Self-test of the cap of tests/gate-handover-matrix.sh (MX-CAP): the cap
# follows the rule of its own comment, the slowest of the five timed runs
# that the comment lists plus 25 % (rounded up), and the three places that
# name the cap (the comment, the usage line, the default in the code) say
# the same number. A cap that was set by eye, or that a new measurement did
# not move, fails here and not in a train at the edge of the limit.
#
# Usage: tests/matrix-cap-selftest.sh [FILE]   FILE: the gate script (default
#   tests/gate-handover-matrix.sh)
# shellcheck disable=SC2016 # the patterns hold a literal $
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
file=${1:-$repo/tests/gate-handover-matrix.sh}
fails=0
fail() { echo "FAIL: $*"; fails=1; }

[ -f "$file" ] || { echo "FAIL: $file is missing"; exit 1; }

comment=$(sed -n 's/^# \{0,1\}//p' "$file" | tr '\n' ' ' | tr -s ' ')

cap_comment=$(sed -n 's/.*The cap is a guard[^:]*: \([0-9][0-9]*\) s is the slowest of five runs.*/\1/p' <<<"$comment")
runs_text=$(sed -n 's/.*is the slowest of five runs[^(]*(\([0-9, and]*\) s) plus 25 %.*/\1/p' <<<"$comment")
cap_usage=$(sed -n 's/^#   CAP: the most seconds of the run (default \([0-9][0-9]*\))$/\1/p' "$file")
cap_code=$(sed -n 's/^time_cap=\${CAP:-\([0-9][0-9]*\)}$/\1/p' "$file")

[ -n "$cap_comment" ] || fail "the comment gives no cap ('N s is the slowest of five runs')"
[ -n "$runs_text" ] || fail "the comment lists no five runs ('(a, b, c, d and e s) plus 25 %')"
[ -n "$cap_usage" ] || fail "the usage line has no 'CAP: ... (default N)'"
[ -n "$cap_code" ] || fail "the code has no 'time_cap=\${CAP:-N}'"

read -r -a runs <<<"$(tr -d ',' <<<"$runs_text" | sed 's/ and / /')"
[ "${#runs[@]}" = 5 ] || fail "the comment lists ${#runs[@]} runs, not five: $runs_text"

slowest=0
for r in "${runs[@]}"; do
    [[ $r =~ ^[0-9]+$ ]] || { fail "'$r' in the runs is not a number of seconds"; continue; }
    [ "$r" -le "$slowest" ] || slowest=$r
done

if [ "$slowest" -gt 0 ]; then
    want=$(((slowest * 125 + 99) / 100))
    for place in comment usage code; do
        var=cap_$place
        [ -z "${!var}" ] || [ "${!var}" = "$want" ] ||
            fail "the cap in the $place is ${!var} s, the rule (slowest $slowest s plus 25 %, rounded up) gives $want s"
    done
fi

[ "$fails" = 0 ] && echo "matrix-cap-selftest: ok (slowest ${slowest:-?} s, cap ${cap_code:-?} s)"
exit "$fails"
