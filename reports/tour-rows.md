# tour-rows

## r1 (F7, RC 11): intended.toml after judges r2a and r2b, two tool fixes

Branch claude/tour-rows-r1, base 62a04be03, head 1ac365f1f. Step 0 green (`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`).

Diffstat (62a04be03..1ac365f1f): demo/tour/intended.toml (rewritten rows, 452 -> 672 rows), demo/tour/run.py +12, tests/test_tour.py +10, tests/test_tour_diff.py +74, tests/tour-diff.py +31. No product code.

### Tool fixes (red, then fix; each pair its own commits)

- rv9ka-3 (a3678a023 red, fb57d59a4 fix): tour-diff reports a text node whose rect is shorter than its lines times its line height as a `cut` diff. Only a row of kind `cut` or `*` that names the node can absorb it; a row of another kind leaves it unnamed (regression), a nodeless row leaves the act open. Red: `AssertionError: ('cut', 'list-lines') not found in [('rect', 'list-lines')]` and `AssertionError: 'open' != 'regression'`.
- rv9kb-1 (0624beaf9 red, 390c82454 fix): the eyes of `hud-figure` (rect + 16, 10, 48 x 30) are masked in the pixel diff. Red: `AssertionError: False is not true : {'same': False, 'fraction': 0.000412, 'blobs': [(1213, 593, 19, 20)]}`. The body of the figure is still compared (a second test).
- run.py (b6984edf9 red, ef4c7c66f fix): a chapter clears only the captures that no other chapter plans (`tour.planned`); a stale still of the chapter itself is still cleared (rv-2 test). Red: `AssertionError: Lists differ: [] != ['m1.Q.json', 'm1.Q.png']`.

### intended.toml (1ac365f1f)

Rows come from the two judge reports and a fresh take, one chapter per TOUR_OUT, on the head (v9) and on a fresh worktree of 733967aa (v8, `TOUR_BASELINE=1`). Every row lists nodes (scene kinds) or boxes (pixel) that were in a json of the take, with the corrected cause: column 675ebd55, key text b8872054, rules 6fa86c16, page end db20c304 / 3e355bb2, other page fa7faabd, Helpers page ffe5e815, hud 6a5cfc1d, start card f5bdc61f (r2a: start-help-1 gone is the same cause), xp chat 62708d35 / 098980b5 (xp-28, not 4fec75c3), notice card 62708d35 + 29281054 (not 675ebd55), Help scroll 9a6506d6, tiles fa3e20a6 + 1f9a5180 (t9), keys panel a611c08a + fa076e7b, notice c9ed1cfd (ti-22), construct chips cfc805d4, panes d9b2c4f1. Rows that would cover rv9ka-1 (`-lines` and their panel rects on the recipe pages) are not there. Tests of test_tour_diff.py that encoded older rules changed with the rows: a pixel row may name up to 8 causes (the bar of tiling has six), mc-59 and co-63 are no longer "unproven findings" (judges r2a and r2b answer them), the x2/x2b/x8 card is a closed rect row (62708d35) and the blink has no box.

### Counts (take on 62a04be03, v9 against the v8 replay; 110 captures)

| | same | change | open | regression | none |
|---|---|---|---|---|---|
| take r2 (old rows) | 1 | 0 | 88 | 10 | |
| this take, old rows, new tool | 1 | 0 | 77 | 21 | 11 |
| this take, new rows | 1 | 90 | 0 | 8 | 11 |

The 21 regressions with the old rows: h1..h1d (3 mc nodes without a row, judge r2a), m1..m4b (cut), m4c..m8b (added nodes without a row), x11 (the Turn Off buttons of rxm1-1, new commits since take r2).

### Still a regression (8): m1, m1b, m1c, m2, m2b, m3, m4, m4b

Each of them: `cut` (and `rect`) of the recipe page text (`mc-s-ingredients-panels-list-lines`, `mc-s-boil-panels-side-lines`, `mc-s-sauce-panels-side-lines`, with their panel rects): rv9ka-1, the page end loses whole lines (29 px for 56 or 112). Left unnamed on purpose; MC-PAGE-END r1 fixes the product and the rows follow. The same take also shows the new `-more` nodes (4c288e3f, rmm1-4: the page says more follows) as a closed row; the cut stays.

Not open or red in the chapters: mc act 4 (Programs click: "the launcher is not open") was red in both of my mc takes (2 of 2) on this head and green (22 of 22) in the judges' takes on e39448ad; I did not look into it (product, out of scope). xp act 31 and construct 14 of 18 are `known`/red as in take r2 (I2, I6: co-44, co-45). v8 handover breaks at Send To after h1d (4 captures), the h2.. captures are `baseline: none` as before.

Not touched (out of scope): rv9ka-1, -2, rv9kb-2..4, ratchet counts, no new sleep.

### Gate

See the last lines below.

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push, no step filter) on 1ac365f1f: every cell PASS, last lines:
```
launcher-click PASS    17s /tmp/merge-gate.ibUOTS/launcher-click.log
voice-acp  PASS    12s /tmp/merge-gate.ibUOTS/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.ibUOTS/voice-events.log
agent      PASS    40s /tmp/merge-gate.ibUOTS/agent.log
tiling-keys-more PASS    42s /tmp/merge-gate.ibUOTS/tiling-keys-more.log
acts       PASS  2044s /tmp/merge-gate.ibUOTS/acts.log
tiling-windows PASS   187s /tmp/merge-gate.ibUOTS/tiling-windows.log
tiling-system PASS    16s /tmp/merge-gate.ibUOTS/tiling-system.log
mc-scale   PASS    15s /tmp/merge-gate.ibUOTS/mc-scale.log
handover   PASS    26s /tmp/merge-gate.ibUOTS/handover.log
watchdog-starved PASS    51s /tmp/merge-gate.ibUOTS/watchdog-starved.log
xp-run     PASS    14s /tmp/merge-gate.ibUOTS/xp-run.log
demo       PASS   230s /tmp/merge-gate.ibUOTS/demo.log
leak       PASS   474s /tmp/merge-gate.ibUOTS/leak.log
tiling-bar PASS   186s /tmp/merge-gate.ibUOTS/tiling-bar.log
tiling-theme PASS    18s /tmp/merge-gate.ibUOTS/tiling-theme.log
mc-host    PASS    19s /tmp/merge-gate.ibUOTS/mc-host.log
tiling-disc PASS    73s /tmp/merge-gate.ibUOTS/tiling-disc.log
e2e        PASS   278s /tmp/merge-gate.ibUOTS/e2e.log
all steps passed; no --push, nothing is pushed
```
`pgrep -fa "chora --headless"` empty.
