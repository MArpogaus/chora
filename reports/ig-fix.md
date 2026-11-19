# ig-fix

## r1 (2026-11-19, build lane)
Head 3a39659c on claude/ig-fix (base a9fd91f3); 2 commits, 2 files.
- ig-1 closed: the gate-lib step now also runs tests/merge-gate-selftest.sh, in parallel with gate-lib-selftest.sh and under-load-selftest.sh. A nested guard (MERGE_GATE_NESTED, set by the selftest case that runs the real `--only gate-lib`) stops the selftest from calling itself. Red first (55774fe0): the case "gate-lib: the step runs merge-gate-selftest.sh" failed 0 vs 1.
- ig-2 closed: with MERGE_GATE_IDENTITY set the identity step prints "identity: MERGE_GATE_IDENTITY overrides the expected identity: <value>"; selftest checks it (and its absence when unset). Red before the fix: 0 vs 1.
- Time of the gate-lib step: 66 s before, 66 s after (the selftest runs beside the other two).
- Gate `--jobs 3 --only pre-commit,ci-check,red-first,identity,gate-lib`: green on 3a39659c.
- A first try without the nested guard forked the gate recursively; killed, no residue.

## PM review r1 (2026-11-19 21:40 UTC; PM, not the builder)
Diff a9fd91f3..3a39659c, 2 files, red first 55774fe0. ig-1, ig-2 closed with selftest checks (read: the nested case sets MERGE_GATE_NESTED=1; the guard skips only the recursive call; both exit codes are checked).
- igr-1 MAJOR (merge): the gate-lib line conflicts with gate-pick 31cfe52d (train 74), which appends `&& tests/cells-for-selftest.sh` to the same line. r2: merge the session branch after train 74 and keep all four selftests in the step.
- igr-2 MINOR: MERGE_GATE_NESTED is not in the usage text of merge-gate.sh (the env list names MERGE_GATE_IDENTITY, MERGE_GATE_STEPS).
Verdict (computed): open MAJOR -> FIX. r2 after train 74 is green.

## r2 (2026-11-19, build lane)
Head b717a5f2 on claude/ig-fix; check commit f5950114 (red: the gate-lib line had no cells-for-selftest before the merge), merge b717a5f2.
- igr-1 closed: merged gate-pick; the gate-lib step runs gate-lib-selftest, under-load-selftest and cells-for-selftest in one background group, merge-gate-selftest beside it, both exit codes checked, nested guard kept. Selftest greps cells-for-selftest.sh on the gate-lib line.
- igr-2 closed: usage text names MERGE_GATE_NESTED.
- Gate `--jobs 3 --only pre-commit,ci-check,red-first,identity,gate-lib`: green (gate-lib 65 s, ci-check 81 s).

## PM review r2 (22:19 UTC; PM not the builder)
Base a9e7084d (train 74), head b717a5f2, 2 files +13/-2. Red first: f5950114 before the merge.
- igr-1 closed: the gate-lib line runs all four selftests, `wait $a && [ $b = 0 ]` checks both exit codes, nested guard kept.
- igr-2 closed: the usage text names MERGE_GATE_NESTED.
- No new finding.
Verdict (computed): MERGE-OK (no open MAJOR). Harness branch: train 76 right after 75 (the gate lock is held).
