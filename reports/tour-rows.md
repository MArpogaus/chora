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

## Review r1

Seeded review of claude/tour-rows-r1, base 62a04be03, head 1ac365f1f (reviewer: cloud lane, opus; read-only on code). Step 0 green (`git fetch --unshallow`, `cargo check --workspace`). No merge gate run (the lane's gate: EXIT 0).

**Verdict: FIX** (rtr1-1 BLOCK, rtr1-2 MAJOR open; rtr1-3 is product for the PM).

### Checks

1. Red first: confirmed. `python3 -m pytest tests/test_tour_diff.py tests/test_tour.py -q` fails at a3678a023 (`CutText::test_a_text_that_loses_lines_is_a_cut`), b6984edf9 (`OldBuild::test_a_chapter_keeps_the_captures_of_another_chapter_in_the_same_folder`), 0624beaf9 (`PaperclipBlink::test_a_blink_is_no_pixel_diff_and_the_body_still_is`); 1ac365f1f: 196 passed, 19 subtests passed.
2. Tool: a `cut` is absorbed only by a `cut`/`*` row naming the node (`named()` falls back to nodeless rows, which make the act `open`, not `change`); the intended.toml has no `*` and no `cut` row. The eye mask (16, 10, 48 x 30 of `hud-figure`) covers both eyes in every state of flavors/xp/assets/assistant/paperclip.svg (eyes at x 22.5..58.5, y 16..36 of the 80 x 104 image; the ±6° tilt of `listening` leaves at most a 1 px column, under the 12 px blob limit); the body, brows and tilt still compare. run.py `clear_captures` keeps only stills another chapter plans (`tour.planned`); a renamed act of any chapter is still cleared. Slips: rtr1-4.
3. Rows: 672 rows, 44 distinct shas in the causes, all exist (`git cat-file -e`). No row names a `-lines` node or a recipe panel rect (only the `-rule` nodes, 6fa86c16/675ebd55); no `*` row, one nodeless row (h2 `added`, open). Every named node (5824 names) is in a json of its act and diffed by its kind in my take, except `xp-ctx-1-0` of h2 (open rows, no baseline). The `371f0fa1` rows are 49, not 35 (x1..x12c, and also c1..c5d, h1..h1d), all `added xp-desktop`: 371f0fa18 is the only commit in 733967aa..62a04be0 that adds the id (`const DESKTOP: &str = "xp-desktop"`, plugins/chora-xp/src/wm.rs:91), the root of xp's wallpaper and icons that T7 split from the window root; an effect of that commit, not a hidden change (the c/h rows cite captures/x1.Q.png, a nit). Pixel boxes: c5e [0, 0, 1280, 720] matches judge r2b (whole frame, co-40); the xp window box [274, 84, 816, 549] covers a window that moves right by about 88 px (x7 side by side), as judge r1 says. The m8 box is wider than its judge verdict: rtr1-1.
4. Counts: no take files were pushed, so I took one: head 1ac365f1f (v9) and a worktree of 733967aa (v8, `TOUR_BASELINE=1`), each chapter its own TOUR_OUT, captures merged, construct's `baseline-none.json` beside the base. `tests/tour-diff.py --intended demo/tour/intended.toml`: **1 same, 86 change, 0 open, 12 regression, 11 none** (report: 1, 90, 0, 8, 11). The extra four: m3b, m7c, m8, m8b (rtr1-2). Chapters: xp 30 green 1 known; tiling 28 green; mc 21 green 1 red (dark act 4); construct 4 green 14 red (I2/I6, co-44/45); handover 11 green; v8 handover BROKE after h1d as before.
5. mc act 4 "the launcher is not open": reproduced on 1ac365f1f in a fresh TOUR_OUT holding only mc (nothing for `clear_captures` to clear), and the check is `pgrep -f "chora-mc --launch fuzzel"` (a process, not a file): not the harness change of this diff. Product: rtr1-3.

### Findings

- rtr1-1 BLOCK demo/tour/intended.toml:3707 (m8 pixel row) the box [464, 199, 352, 322] with "12f97cd3 Programs is the kit list" absorbs the launcher's absence: on this head v9 m8.Q.png shows no launcher at all (act 4 red), v8 shows fuzzel at exactly that box, and judge r2a (e39448ad, launcher open) gave [500, 225, 270, 290] (rv9ka-2); m8 counts `change` where it is a regression. Fix: box m8 as the judge did ([500, 225, 270, 290], or none until the launcher opens again), so m8 stays a regression until rtr1-3 is fixed.
- rtr1-2 MAJOR demo/tour/intended.toml (m3b, m7c, m8, m8b rows; the row at :353 says "the blob lands in m1 or m1c by take") the counts do not reproduce: my take gives 86 change, 12 regression; in m3b and m8b the KEYS block of the column (`mc-r0/r1-right-keys-*`, `-keys-title`, `-title-rule`) moves 28 px (y 406 in v8 -> 378) and in m3b, m7c, m8, m8b the hairlines [888, 378, 336, 1], [937, 406, 287, 1] blob, with no row: whether the v8 column still shows its notice depends on the take, and the rows fit one take; the take files are not on the branch. Fix: name those nodes and the two hairline boxes (675ebd55) in every mc act whose v8 column can hold the notice, then two takes must agree (keep the take's diff json under the report media).
- rtr1-3 MAJOR (product, for the PM; not this diff, no fix here) mc dark act 4 (scene m8): a click on Programs in the column opens no launcher on 62a04be0/1ac365f1 (3 of 3 takes: the lane's 2, mine 1), green 22/22 on e39448ad. Evidence: v9 m8.Q.png shows no launcher, the pointer mark is on `mc-r0-right-doors-programs` (888, 644, 65 x 28); candidates in e39448ad..62a04be0 on that path: cd277ee40 (rpx1-10: the launcher is now `chora-mc --launch fuzzel --config FILE`, a file rewritten when the settings change; this is the only Programs act and it runs in the dark theme) and 761c0b29/798fdc33 (column rects). The written config passes `fuzzel --check-config` (1.11.1). The gate is blind to it: `launcher-click` covers tiling and construct only (tests/merge-gate.sh:24). Fix: a lane bisects e39448ad..62a04be0 with `run.py mc` and a light/dark launcher cell.
- rtr1-4 MINOR tests/tour-diff.py:105 `cut_of` misses three kinds of cut: a text without `line_height` in the json (the field is skipped when None, crates/chora-scene/src/node.rs:632; the core default 1.2, crates/chora/src/text.rs:36; 200 of 5152 text nodes in my take), a text that wraps by `max_width` (lines are counted by `\n` only), and a text cut in both builds that v9 cuts more (`not cut_of(ka[k])`). None changes this take (a 1.2 default adds no cut). Fix: default `line_height` to 1.2 and report a cut when v9 is shorter than v8 by a line even if both are cut.
- rtr1-5 NIT tests/tour-diff.py:72 the eye box is in fixed pixels of an 80 x 104 figure; a scaled figure gets a misplaced mask (a false regression, nothing hidden). Fix: scale the box by the rect's width/80 and height/104.

## r2

Base 1ac365f1f, head f992fe6ac (claude/tour-rows-r2). Commits: f5a4d670a red (test(tour): cut_of default line height, a text cut twice, rtr1-4), 1c6e66e72 fix (cut_of), f992fe6ac rows (m8 box, mc KEYS and page rows). Diffstat: demo/tour/intended.toml +290 -2, tests/tour-diff.py and tests/test_tour_diff.py (cut_of, cut_more; 2 tests). rtr1-5 (eye mask) not done (optional).

- rtr1-1: m8 box is [500, 225, 270, 290]; m8 counts a regression in all 3 takes (the launcher, rtr1-3 / MC-LAUNCHER).
- rtr1-2: the KEYS nodes (mc-r0/r1-right-keys-N-key, -key-hit, -key-mark, -words, -keys-title, -title-rule, as rect) and the two hairline boxes [888,378,336,1], [937,406,287,1] are named in all 22 mc acts; m6 (region slot flip, take 2) and m9b (agent bands, take 1) and the t7 notice card box [896,560,368,104] added from the takes.
- rtr1-4: cut_of defaults line_height to 1.2 and a text cut in both builds that v9 cuts by a line more is a cut; a wrapped text (lines by max_width) is still not seen.

### Takes (rows of the head; v9 on the head, v8 = this harness against the 733967aa binaries, TOUR_BASELINE=1, each chapter its own TOUR_OUT; the 733967aa tree has no demo/tour, so run.py is the head's with TOUR_REPO on the worktree)

| take | change | same | none | regression | regressions |
|---|---|---|---|---|---|
| 1 | 83 | 1 | 11 | 15 | m1 m1b m1c m2 m2b m3 m4 m4b m8, t3b t3c t3d t3e t3f t3g |
| 2 | 89 | 1 | 11 | 9 | m1 m1b m1c m2 m2b m3 m4 m4b m8 |
| 3 | 88 | 1 | 11 | 10 | m1 m1b m1c m2 m2b m3 m4 m4b m8, t3c |

The mc acts now reproduce in all three takes (m3b, m7c, m8b hold, m6 and m9b no longer flip). The counts are NOT equal: the take differs in the tiling acts t3b..t3g, a take-timing diff of the tiling left tile (v9 types the "chora" text into it, v8 shows an empty tile at the still; take 1 six acts, take 3 one act, take 2 none): the typed content is a pixel blob in the tile, no row names it, and I did not mask the tile (it would hide a real change there). Finding rtr2-1 (harness, for the PM): the tiling still waits for the typed text, or a row boxes the tile for t3b..t3g.

Finding rtr2-2 (product, new): m1 m1b m1c m2 m2b m3 m4 m4b regress in every take with a : when the card of chora-notify stands, the page ends 105 px higher (608 to 503, db20c304 / 3e355bb2) and the list panel of the recipe page is 29 px high for 4 lines (mc-s-ingredients-panels-list-lines 112 -> 29; mc-s-sauce-panels-side-lines 56 -> 29). Left unnamed on purpose (a real cut); the review take did not have the card up there.

Diff jsons: media/2026-11-25/tour-rows-r2/take1-diff.json, take2-diff.json, take3-diff.json.

Gate: CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 (no --push), all 50-odd cells PASS, last lines: "e2e PASS 263s ... all steps passed; no --push, nothing is pushed". pytest tests/test_tour_diff.py tests/test_tour.py: 198 passed. pgrep -fa 'chora --headless' empty.
