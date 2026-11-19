# ig-fix

## r1 (2026-11-19, build lane)
Head 3a39659c on claude/ig-fix (base a9fd91f3); 2 commits, 2 files.
- ig-1 closed: the gate-lib step now also runs tests/merge-gate-selftest.sh, in parallel with gate-lib-selftest.sh and under-load-selftest.sh. A nested guard (MERGE_GATE_NESTED, set by the selftest case that runs the real `--only gate-lib`) stops the selftest from calling itself. Red first (55774fe0): the case "gate-lib: the step runs merge-gate-selftest.sh" failed 0 vs 1.
- ig-2 closed: with MERGE_GATE_IDENTITY set the identity step prints "identity: MERGE_GATE_IDENTITY overrides the expected identity: <value>"; selftest checks it (and its absence when unset). Red before the fix: 0 vs 1.
- Time of the gate-lib step: 66 s before, 66 s after (the selftest runs beside the other two).
- Gate `--jobs 3 --only pre-commit,ci-check,red-first,identity,gate-lib`: green on 3a39659c.
- A first try without the nested guard forked the gate recursively; killed, no residue.
