#!/bin/bash
# Repeats the cells of the matrix gate (tests/gate-handover-matrix.sh) N
# times and counts the two window-loss signals of the old backlog items
# (backlog.md, "## UX and looks" A13 and "## Flakes and gates" A17):
#
#   panes  A13: the cell `mc construct panes`, which drags a window from mc
#          onto the construct three times. Counted: a construct window with
#          `pane: none` in a round line (after the drag), per run.
#   cells  A17: the cgive and voice cells of all pairs. Counted: the cells
#          whose window ends in xp without the keyboard focus (focus is not
#          true), and the cgive and voice cells that are not pass.
#
# Usage: tests/repeat-handover.sh panes|cells N OUT
#   GATE: the gate script to run (default tests/gate-handover-matrix.sh)
#   LOAD: a shell command run beside the runs (default none): it starts
#         before the first run and is stopped after the last one, in its own
#         process group, so that a `cargo build` of it ends with it
#   CAP:  the cap of the gate in seconds (default 387, the gate's own)
# OUT gets run-N/ with the gate's output (gate.out), its log (matrix.log)
# and its exit status (rc), and summary.txt with one line per run and totals.
#
# The counts read the log lines of the gate: `mc construct panes round R:`
# holds the construct windows as JSON, and each cell line is
# `SRC DST PATH RESULT owner=.. focus=.. pane=..`.
set -uo pipefail

mode=${1:-}
n=${2:-}
out=${3:-}
case $mode in panes | cells) ;; *) echo "usage: $0 panes|cells N OUT" >&2; exit 2 ;; esac
case $n in '' | *[!0-9]*) echo "N must be a number of runs" >&2; exit 2 ;; esac
[ -n "$out" ] || { echo "usage: $0 panes|cells N OUT" >&2; exit 2; }

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
gate=${GATE:-$repo/tests/gate-handover-matrix.sh}
case $mode in
panes) only='mc construct panes' ;;
cells) only='[a-z]+ [a-z]+ (cgive|voice)' ;;
esac

mkdir -p "$out"
loadpid=
if [ -n "${LOAD:-}" ]; then
    setsid bash -c "$LOAD" >"$out/load.log" 2>&1 &
    loadpid=$!
fi

for i in $(seq "$n"); do
    d=$out/run-$i
    mkdir -p "$d"
    ONLY=$only SHOTS=$d CAP=${CAP:-387} "$gate" >"$d/gate.out" 2>&1
    echo $? >"$d/rc"
done

if [ -n "$loadpid" ]; then
    kill -- "-$loadpid" 2>/dev/null
    wait "$loadpid" 2>/dev/null
fi

summary=$out/summary.txt
if [ "$mode" = panes ]; then
    printf 'run rc rounds_none setup_none panes_result\n' >"$summary"
    tot=0
    for i in $(seq "$n"); do
        d=$out/run-$i
        rounds=$(grep '^mc construct panes round' "$d/matrix.log" 2>/dev/null)
        none=$(printf '%s\n' "$rounds" | grep -o '"owner":"construct","pane":"none"' | wc -l)
        setup_none=$(grep '^setup filler' "$d/matrix.log" 2>/dev/null | grep -o '"owner":"construct","pane":"none"' | wc -l)
        res=$(grep -E '^mc construct panes (pass|gap)$' "$d/matrix.log" 2>/dev/null | tail -n 1)
        printf '%s %s %s %s %s\n' "$i" "$(cat "$d/rc")" "$none" "$setup_none" "${res:-none}" >>"$summary"
        tot=$((tot + none))
    done
    printf 'total_rounds_none %s of %s runs\n' "$tot" "$n" >>"$summary"
else
    printf 'run rc xp_cells xp_no_focus cells_not_pass\n' >"$summary"
    tot_nf=0
    for i in $(seq "$n"); do
        d=$out/run-$i
        log=$d/matrix.log
        xp=$(awk '$2 == "xp" && ($3 == "cgive" || $3 == "voice") && $4 ~ /^(pass|kept|gap|skip)$/' "$log" 2>/dev/null | wc -l)
        nf=$(awk '$2 == "xp" && ($3 == "cgive" || $3 == "voice") && $0 !~ /focus=true/' "$log" 2>/dev/null | wc -l)
        np=$(awk '($3 == "cgive" || $3 == "voice") && $4 != "pass"' "$log" 2>/dev/null | wc -l)
        printf '%s %s %s %s %s\n' "$i" "$(cat "$d/rc")" "$xp" "$nf" "$np" >>"$summary"
        tot_nf=$((tot_nf + nf))
    done
    printf 'total_xp_no_focus %s of %s runs\n' "$tot_nf" "$n" >>"$summary"
fi
cat "$summary"
