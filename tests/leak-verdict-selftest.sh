#!/bin/bash
# Self-test of the verdict of tests/gate-leak.sh (sh-35): recorded tables go
# through tests/leak-verdict.awk. Train118 (one bounded +7 MB heap step at
# round 6, flat after) must pass; the sh-33 red shape (+31703 kB, +8 fds, +4
# windows per round) and a steady +1.5 MB per round must fail.
#
# sh-38: the RSS of the core is not flat between the rounds even without a leak
# (the heap of the core sits on one of two levels, about 172 and 179 MB, and
# moves between them from round to round). Tables recorded in the lane
# container, no leak in any of them, must pass: idle6 is a run of 8 rounds
# that the median of the differences put at +1736 kB per round (bound 1024),
# r12-2 and r12-8 are runs of 12 rounds. A planted leak (a ramp added to the
# recorded table: 2.5 MB per round on 8 rounds, 1.5 MB per round on 12 rounds,
# the 4 MB texture of a window per round) must fail on each of them.
# shellcheck disable=SC2046,SC2086 # the tables are lists of numbers to split
dir=$(dirname "$0")
verdict() { awk -F'\t' -v warm=2 -v bound=1024 -v bw=2 -v bc=0 -f "$dir/leak-verdict.awk"; }

table() {
    local fds=${FDSTEP:-0} wins=${WINSTEP:-0} r=0 v
    printf 'round\tlook\trss_kb\tfds\twindows\tclients\n'
    for v; do
        printf '%d\tx\t%d\t%d\t%d\t0\n' "$r" "$v" $((40 + fds * r)) $((2 + wins * r))
        r=$((r + 1))
    done
}
status=0
expect() {
    local want=$1 name=$2 out got=pass
    shift 2
    out=$(table "$@" | verdict)
    grep -q '^FAIL:' <<<"$out" && got=fail
    if [ "$got" = "$want" ]; then echo "ok: $name -> $got"; else
        echo "FAIL: $name: want $want, got $got"; echo "$out"; status=1
    fi
}
expect pass "train118 (one bounded step)" 134612 173656 175752 175044 175380 176176 182956 182520 182960
expect fail "steady +1.5 MB/round" 100000 101500 103000 104500 106000 107500 109000 110500 112000
ramp() {
    local step=$1 round=0 v
    shift
    for v; do
        echo $((v + step * round))
        round=$((round + 1))
    done
}
idle6="134332 175144 176280 175416 177152 176320 180620 176500 183308"
r12_2="134400 175696 179072 176044 176324 176376 183260 183148 183192 180288 183172 183996 184020"
r12_8="135228 175216 176748 179224 181084 184620 183752 183696 184628 183780 183752 184608 183940"
expect pass "recorded idle6, 8 rounds, no leak (+1736 by the median of the differences)" $idle6
expect pass "recorded r12-2, 12 rounds, no leak" $r12_2
expect pass "recorded r12-8, 12 rounds, no leak" $r12_8
expect fail "recorded idle6 with a planted +2.5 MB/round" $(ramp 2500 $idle6)
expect fail "recorded r12-2 with a planted +1.5 MB/round" $(ramp 1500 $r12_2)
expect fail "recorded r12-8 with a planted +1.5 MB/round" $(ramp 1500 $r12_8)
FDSTEP=8 WINSTEP=4 expect fail "sh-33 red (+31703 kB, +8 fds, +4 windows per round)" \
    100000 131703 163406 195109 226812 258515 290218 321921 353624
exit $status
