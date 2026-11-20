# gate-silent

## r1

Lane: cloud, branch claude/gate-silent from f5ac3500 (probe-leak). Brief briefs/gate-silent-r1.md.
Head **baac5dcf** (pushed). Authors: Marcel Arpogaus on all commits.

Commits:

| sha | what |
|---|---|
| b9205740 | red: gate-lib-selftest (need_fuzzel, fuzzel_older, cell_skip, cells_end) and merge-gate-selftest (SKIP line, summary, --push refusal, --allow-skip) |
| 546674e6 | MC-47-CELL: mc-47-cell.patch applied as is (git am) |
| 5ad34491 | fix: SKIP in merge-gate.sh; shared fuzzel check in gate-lib; guards in launcher-click, xp-fields (Run box) and the mc-sofa cell |
| baac5dcf | fix: the --allow-skip hint is a comma list (found by the dry run), selftest in the same commit |

Rule (in the header of tests/merge-gate.sh): a step that passed but has a line `SKIP: <reason>` in its log prints SKIP, not PASS; the
summary lists `skipped: <step> (<reasons>)`; `--push` refuses while any step is SKIP unless `--allow-skip STEP,...` names it.
Under CI a skip still fails (gate-lib skip and cell_skip). A cell of several cells (mc-sofa launcher-click-row) is skipped alone
with `SKIP: cell <name>: <reason>`; a script whose cells all skip ends in SKIP (cells_end), not ok.
Not matched: the `SKIP name: reason` lines of e2e-sequence.sh (they have their own list, tests/e2e-skips.txt).

Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,gate-lib,ci-check,build,test,clippy,xp-fields,launcher-click,mc-sofa`

Run 1, fuzzel 1.11.1 on PATH (all PASS, no skipped line; mc-sofa log has `ok: launcher-click-row (dark)`, `ok: 13 cells ran`):

```
pre-commit PASS     6s
gate-lib   PASS    84s
ci-check   PASS   131s
build      PASS   189s
test       PASS   249s
clippy     PASS    27s
xp-fields  PASS    33s
mc-sofa    PASS    49s
launcher-click PASS    17s
partial run: nothing is pushed
```

Run 2, stub fuzzel 1.9.2 first on PATH (the three cells SKIP, the rest PASS):

```
xp-fields  SKIP    16s
launcher-click SKIP     0s
mc-sofa    SKIP    47s
skipped: xp-fields (fuzzel 1.9.2 < 1.11)
skipped: launcher-click (fuzzel 1.9.2 < 1.11)
skipped: mc-sofa (cell launcher-click-row: fuzzel 1.9.2 < 1.11)
partial run: nothing is pushed
```

Dry run of --push (MERGE_GATE_STEPS with the real commands of the three cells, stub 1.9.2, --push claude/gate-silent-dryrun): rc 1, nothing pushed
(`git ls-remote` shows no such branch):

```
merge-gate: refusing to push: xp-fields launcher-click mc-sofa skipped; pass --allow-skip xp-fields,launcher-click,mc-sofa to push with a host gap you know
```

Notes for the PM: a lane container (fuzzel 1.9.2) now gets SKIP instead of red-by-host, and the train needs `--allow-skip` for those cells or
fuzzel >= 1.11 on PATH (PM host). A full-gate `--push` on the PM host with 1.11.1 is unchanged. Not touched: flakes, matrix cap, lint lists.
