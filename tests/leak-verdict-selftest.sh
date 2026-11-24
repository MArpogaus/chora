#!/bin/bash
# Self-test of the verdict of tests/gate-leak.sh (sh-35): recorded tables go
# through tests/leak-verdict.awk. Train118 (one bounded +7 MB heap step at
# round 6, flat after) must pass; the sh-33 red shape (+31703 kB, +8 fds, +4
# windows per round) and a steady +1.5 MB per round must fail.
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
FDSTEP=8 WINSTEP=4 expect fail "sh-33 red (+31703 kB, +8 fds, +4 windows per round)" \
    100000 131703 163406 195109 226812 258515 290218 321921 353624
exit $status
