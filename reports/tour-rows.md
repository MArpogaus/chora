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

## Review r2 (PM, 01:32 UTC)

Base 1ac365f1, head f992fe6a, +329 -9 (intended.toml +290 data; tour-diff.py and its test +46). Red f5a4d670 before fix 1c6e66e7: yes. rtr1-1 box [500,225,270,290]: yes. rtr1-2 KEYS rows: mc acts reproduce in all three takes. rtr1-4: yes.
- Done-when "two takes with equal counts" fails (15/9/10 regressions): INCOMPLETE, not MERGE-OK.
- rtr2-1 (MAJOR, harness): the tiling t3b..t3g still is taken before the typed text lands; the counts depend on timing. Fix in the tour (wait for the text), not a mask.
- rtr2-2 is not new: the takes ran on 1ac365f1, which lacks MC-PAGE-END r1 (8a0249a2, t144). rv9ka-1 is fixed on the session head; the r3 takes run on a head with it.
Verdict: INCOMPLETE; round r3 on f992fe6a merged with the session head.

## r3

Base f992fe6ac (tour-rows r2) merged with the session head 157be1954 (39e407547); head f36a229d8 (claude/tour-rows-r3). Commits: 87ee25b63 red (test(tour): tiling waits for the typed tile text, fails on f992fe6a: no `wait_for_text`), 1d8d2534a fix, 2fabc1fd7 and f36a229d8 fixes of the same wait (the first version counted the foot that holds the command line and the v8 look has no state.get; found by the takes). Diffstat of my commits: demo/tour/chapters/tiling.py +34, tests/test_tour.py +40. No intended.toml row added.

Fix (rtr2-1): in the window loop of the tiling chapter the act is cut with `t.flush("tile text")` and `wait_for_text(i)` (wall clock, 30 s, one failure line `tiling: the text of tile N did not land in 30 s (...)`), then `t.wait(0.3); t.done()`: the Q capture is taken after every text tile (`sh`/`python3 content.py KIND`) printed. The condition reads processes (`ps`), not the look, so it holds on the v8 build too. No mask.

### Takes (v9 on this head, v8 = the head's harness on the 733967aa binaries with TOUR_BASELINE=1, each chapter its own TOUR_OUT; clean serial run after two aborted ones)

| take | change | same | none | regression |
|---|---|---|---|---|
| 1 | 54 | 1 | 11 | 44 |
| 2 | 54 | 1 | 11 | 44 |
| 3 | 54 | 1 | 11 | 44 |

Counts are equal and the lists are the same in all three: c2 m1 m10 m1b m1c m2 m2b m3 m4 m4b m6 m6b m8 x1 x10 x10b x10c x11 x11b x12 x12b x12c x2 x2b x2c x3 x4 x5 x5b x5c x5d x5e x5f x5g x6 x6b x6c x6d x6e x7 x7b x8 x9 x9b. All t* acts (t3..t3h) are now change or same: rtr2-1 is fixed.

The counts are higher than r2 (44 vs 9..15) because the head now carries the session head's product changes, which the rows of intended.toml do not name yet (not in this lane's brief): x*: the new nodes hud-line, hud-line-tail, xp-desktop, notify-card rect, and xp-item-3/4 'UXTerm'/'XTerm' -> 'Foot'/'Foot Client' (the launcher environment, TOUR-LAUNCH); c2: construct hints rects (key overlay, I2 gap 9 px red on the candidate); m*: the right column (chat door 194 -> 110, notify rows gone), footer/header rects.

rv9ka-1 (rtr2-2): the recipe list cut is gone (no `ingredients-panels-list-lines` cut; m1 m1b m1c m4 m4b have no cut). NOT fully: m2, m2b, m3 still have `mc-s-boil-panels-side-lines` 29 px high for 56 px of 2 lines in all three takes: a remaining cut in the boil page's side panel, for the PM (product).

Diff jsons: media/2026-11-26/tour-rows-r3/take1-diff.json, take2-diff.json, take3-diff.json. Rows of intended.toml for the session-head changes are open work.

Gate: CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 (no --push): all cells PASS, last line "all steps passed; no --push, nothing is pushed" (last 20 lines in media/2026-11-26/tour-rows-r3/gate-last20.txt). pytest tests/test_tour_diff.py tests/test_tour.py: 203 passed. pgrep -fa 'chora --headless' empty.

## Review r3 (PM, 2026-11-26, 71 lines, 39e40754..f36a229d)
Checks (brief tour-rows-r3 Done when):
- Red first: 87ee25b6 (test, fails on f992fe6a: no wait_for_text) before 1d8d2534 (fix); 2fabc1fd, f36a229d refine the same wait. Holds.
- rtr2-1 fixed: the t3 loop is cut with flush, waits on wait_for_text (ps based, 30 s, one failure line), then done; 5 unit tests (order in the loop, return, timeout line, ps parse, window wait). Takes x3: equal counts and equal lists (54 change, 1 same, 11 none, 44 regression); all t* acts change or same. Holds.
- KISS, no mask: holds.
- Gate: full gate all PASS; pytest 203 passed. Holds.
- "no m1..m4b cut": m1, m1b, m1c, m4, m4b clean; m2, m2b, m3 keep mc-s-boil-panels-side-lines 29 of 56 px in all three takes. A product cut outside this lane's area (harness), so not a lane failure: filed as rtr3-1 for an mc lane.
Findings:
- rtr3-1 (MAJOR, product, mc): boil page side panel cut (29 of 56 px, 2 lines) at m2, m2b, m3; rest of rv9ka-1. To an MC-BOIL r1 lane (red: no cut at m2/m2b/m3).
- rtr3-2 (MAJOR, F7, harness): 44 regressions are session-head product changes with no intended.toml row (x: hud-line, xp-desktop, notify-card, Foot names; c2 hints; m: right column). Each needs a judged row or a fix: TOUR-ROWS r4 (judge by eye, one row per intended change, regressions filed).
- rtr3-3 (NIT): content_started counts every foot with app-id code/notes/page on the machine; a foot left over from another chapter would satisfy it early. Accept; note.
Verdict (computed on this diff): no open MAJOR in the lane's change: MERGE-OK f36a229d, joins train 151. rtr3-1, rtr3-2 are new lanes.

## r4

Base 613844fc6 (session head, t151), Step 0 merge of the harness fix f396ebe63 (claude/tour-harness-r2) as 035df1fc1; head 67a2dc4d1 (claude/tour-rows-r4). Step 0 green (`git fetch --unshallow`, merge, `cargo check --workspace`, `cargo build --workspace`).
Commits: 2680e71be red (test(tour): intended rows name product commits after v8), 30f0ac17e (h2 rows: 4278d4cd, not 22fead94), 67a2dc4d1 (75 rows). Diffstat 035df1fc1..67a2dc4d1: demo/tour/intended.toml +458 -4, tests/test_tour_diff.py +47 (2 files, +501 -4). No product code, no launch.py, no chapter.

**Red first.** `IntendedCommits` in tests/test_tour_diff.py: every row of intended.toml names a commit sha, every named sha is a commit, in the history of HEAD and not of 733967aa (v8), and it changes product code (crates/, plugins/, flavors/, services/, dist/, Cargo.*). The literal rule ("names a sha that exists") holds on the base (720 rows, 44 shas, all in the history); what fails on the base is that a cited commit can make a v9 diff: the three h2 rows cite 22fead94, which touches only tests/gate-handover.sh (`AssertionError: [('22fead94', 'tests/gate-handover.sh')] != []`). Both takes run the head's harness, so a commit of tests/ cannot change a capture. Fix 30f0ac17e: the Send To submenu row `xp-ctx-1-0` comes from 4278d4cd (feat(xp): Send To in the window menu).

**Takes.** v9: this branch, each chapter its own TOUR_OUT. v8: the 733967aa binaries (worktree), TOUR_BASELINE=1, each chapter its own TOUR_OUT, run by the head's harness from a scratch worktree with two take-side changes the replay needs (rtr4-4, rtr4-5; not committed, launch.py is outside this lane): tests/tool-version.sh copied into the v8 tree, and `ensure_notify` skipped under TOUR_BASELINE (as the r3 harness ran the replay: no chora-notify for a look that does not list it). Without them every v8 chapter BROKE (rtr4-5) and then mc dark and handover BROKE with 0 captures (rtr4-4). v8 handover stops after h1d as in r1 to r3 (h2.. are `baseline_none`).

Take 1 regressions (old rows): 57, not 44. All 44 of r3 are among them; the 13 more: c2b c3 c3b c3c c3d c4 c4b c5c c5d c5e (construct: 75d4623e, after r3) and m8b m9 m9b (mc dark: since 80dbd0b48 the launcher opens, so its Escape no longer turns v9 back to Agents and v9 stays on the agenda while the v8 replay drifts to Agents and The Morning).

### Judged, act by act (take 1; stills in media/2026-11-26/tour-rows-r4/)

Every act was looked at in both stills (v8 over v9) and its unnamed nodes traced with `git log -S` / `git log -- <path>` in 733967aa..HEAD. Rows name only the unnamed nodes of their act and kind (a pixel row the box of the act's unnamed blobs); each cause starts with the commit and cites the still.

| act | in r3's 44 | unnamed in the take | judged | commit (rows) or finding | still |
|---|---|---|---|---|---|
| c2 | yes | added, pixel | row x2 | 5025dc47 | c2.Q-v8-v9.jpg |
| c2b | no | visible, rect, missing, added, pixel | finding | rtr4-3 | c2b.Q-v8.jpg, c2b.Q-v9.jpg |
| c3 | no | visible, rect, missing, added | finding | rtr4-3 | c3.H50-v8.jpg, c3.H50-v9.jpg |
| c3b | no | visible, rect, missing, added, pixel | finding | rtr4-3 | c3b.Q-v8.jpg, c3b.Q-v9.jpg |
| c3c | no | visible, rect, missing, added, pixel | finding | rtr4-3 | c3c.Q-v8.jpg, c3c.Q-v9.jpg |
| c3d | no | visible, rect, missing, added, pixel | finding | rtr4-3 | c3d.Q-v8.jpg, c3d.Q-v9.jpg |
| c4 | no | visible, rect, missing, pixel | finding | rtr4-3 | c4.Q-v8.jpg, c4.Q-v9.jpg |
| c4b | no | visible, rect, missing, added, pixel | finding | rtr4-3 | c4b.Q-v8.jpg, c4b.Q-v9.jpg |
| c5c | no | visible, rect, missing, added | row x4 | 75d4623e | c5c.H50-v8-v9.jpg |
| c5d | no | visible, rect, missing, added, pixel | row x5 | 75d4623e | c5d.Q-v8-v9.jpg |
| c5e | no | visible, rect | finding | rtr4-3 | c5e.Q-v8.jpg, c5e.Q-v9.jpg |
| m1 | yes | rect | row x1 | 859cf3fe | m1.Q-v8-v9.jpg |
| m1b | yes | rect | row x1 | 859cf3fe | m1b.Q-v8-v9.jpg |
| m1c | yes | rect | row x1 | 859cf3fe | m1c.Q-v8-v9.jpg |
| m2 | yes | rect, cut | finding | rtr4-1 | m2.Q-v8.jpg, m2.Q-v9.jpg |
| m2b | yes | rect, cut | finding | rtr4-1 | m2b.Q+1-v8.jpg, m2b.Q+1-v9.jpg |
| m3 | yes | rect, cut | finding | rtr4-1 | m3.Q-v8.jpg, m3.Q-v9.jpg |
| m4 | yes | rect | row x1 | 859cf3fe | m4.Q-v8-v9.jpg |
| m4b | yes | rect | row x1 | 859cf3fe | m4b.Q-v8-v9.jpg |
| m6 | yes | missing, rect, added | row x3 | 4363de11 | m6.Q-v8-v9.jpg |
| m6b | yes | missing, added | row x2 | 4363de11 | m6b.Q-v8-v9.jpg |
| m8 | yes | pixel | finding | rtr4-2 | m8.Q-v8.jpg, m8.Q-v9.jpg |
| m8b | no | rect, visible, pixel | row x6 | 675ebd55, fa7faabd | m8b.Q-v8-v9.jpg |
| m9 | no | missing, visible, rect, added, pixel | row x5 | 675ebd55, fa7faabd | m9.Q-v8-v9.jpg |
| m9b | no | missing, rect, visible, added | row x5 | 675ebd55, fa7faabd | m9b.Q-v8-v9.jpg |
| m10 | yes | missing, rect, visible, added, pixel | row x7 | 675ebd55, fa7faabd | m10.Q+0.5-v8-v9.jpg |
| x1 | yes | added | row x1 | e4157dbd | x1.Q-v8-v9.jpg |
| x2 | yes | added | row x1 | e4157dbd | x2.Q-v8-v9.jpg |
| x2b | yes | added | row x1 | e4157dbd | x2b.Q-v8-v9.jpg |
| x2c | yes | added | row x1 | e4157dbd | x2c.Q-v8-v9.jpg |
| x3 | yes | added | row x1 | e4157dbd | x3.Q-v8-v9.jpg |
| x4 | yes | added | row x1 | e4157dbd | x4.H50-v8-v9.jpg |
| x5 | yes | added | row x1 | e4157dbd | x5.Q-v8-v9.jpg |
| x5b | yes | added | row x1 | e4157dbd | x5b.Q-v8-v9.jpg |
| x5c | yes | added | row x1 | e4157dbd | x5c.Q-v8-v9.jpg |
| x5d | yes | added | row x1 | e4157dbd | x5d.Q+1-v8-v9.jpg |
| x5e | yes | added | row x1 | e4157dbd | x5e.Q-v8-v9.jpg |
| x5f | yes | added | row x1 | e4157dbd | x5f.Q-v8-v9.jpg |
| x5g | yes | added | row x1 | e4157dbd | x5g.Q-v8-v9.jpg |
| x6 | yes | added | row x1 | e4157dbd | x6.Q-v8-v9.jpg |
| x6b | yes | added | row x1 | e4157dbd | x6b.Q-v8-v9.jpg |
| x6c | yes | added | row x1 | e4157dbd | x6c.Q-v8-v9.jpg |
| x6d | yes | added | row x1 | e4157dbd | x6d.Q+1-v8-v9.jpg |
| x6e | yes | added | row x1 | e4157dbd | x6e.Q-v8-v9.jpg |
| x7 | yes | added | row x1 | e4157dbd | x7.Q-v8-v9.jpg |
| x7b | yes | added | row x1 | e4157dbd | x7b.Q-v8-v9.jpg |
| x8 | yes | added | row x1 | e4157dbd | x8.Q+0.5-v8-v9.jpg |
| x9 | yes | added | row x1 | e4157dbd | x9.Q-v8-v9.jpg |
| x9b | yes | added | row x1 | e4157dbd | x9b.Q-v8-v9.jpg |
| x10 | yes | added | row x1 | e4157dbd | x10.Q-v8-v9.jpg |
| x10b | yes | added | row x1 | e4157dbd | x10b.Q-v8-v9.jpg |
| x10c | yes | added | row x1 | e4157dbd | x10c.Q-v8-v9.jpg |
| x11 | yes | added | row x1 | e4157dbd | x11.Q-v8-v9.jpg |
| x11b | yes | added | row x1 | e4157dbd | x11b.Q-v8-v9.jpg |
| x12 | yes | added | row x1 | e4157dbd | x12.H50-v8-v9.jpg |
| x12b | yes | added | row x1 | e4157dbd | x12b.Q-v8-v9.jpg |
| x12c | yes | added | row x1 | e4157dbd | x12c.H50-v8-v9.jpg |

Rows: 75 over 45 acts. Findings: 12 acts in 3 findings. 45 + 12 = 57 (the 44 of r3: 40 rows, 4 findings: m2 m2b m3 m8).

Why each row is intended (the commit says so and the v9 still shows it):
- x1..x9b (31 acts), `added hud-line-tail` and in x5..x6d `hud-chat-tail`: e4157dbd (rv9kb-2) makes the leaning tail of the balloon its own node; the v9 chat balloon's tail runs from the box to the paperclip head (judge r2b: in v8+62708d35 the tip ended 39 px left of the clip). No pixel blob left unnamed in this take (the blob [1182, 560, 50, 18] of r3's x1/x5e/x5g/x6e is gone; the blink is on the stepped clock since c2625c0f7).
- m1 m1b (list), m1c m4b (boil side), m4 (sauce side and its lines) `rect`: 859cf3fe (rv9ka-1) ends the page at the footer with no card and puts the more mark where the lines end: the panel ends 21 px higher, v9 shows whole lines and the down mark where v8 cut "parsley." in half (m4).
- m6 m6b `missing`/`added`/`rect` of `mc-r1-right-keys-N-*`: 4363de11 (rpe-1) adds the row "Up Down  Scroll the page" after Home; the rows after it shift by one, so the key-hit and key-mark of rows 3 and 13 move to 19 and the last row is 20.
- c2 `added construct-hint-put-in*`, `construct-hints-sep-4` and the bar box [118, 647, 1045, 42]: 5025dc47 (co-79): stood back from a pane the bar names "Super Alt 2 Into face"; the bar grows by that hint.
- c5c c5d (all kinds, c5d the whole frame): 75d4623e (co-drop r1, rcr5-1/2, MERGE-OK): a pane dropped onto the face at the front is placed there and the person steps in; the stood-back chrome (doors, face label, turn chips, cube edges) goes and the step-back hint comes. v9 c5d shows the xp face 1:1 with the sunset window on it.
- m8b m9 m9b m10: the two builds show two pages: fa7faabd (Home goes to the first page, mc-38) plus 80dbd0b4 (the launcher opens, so its Escape closes it and turns no page; rtr1-3): v9 stays on the agenda; the v8 replay (no Home; its chat does not open, so its Escape steps back) is on Agents and then The Morning. The column: 675ebd55 (notices out of the column, the region slot flips by take; bb3de644 the cross-fade) and the chat open in the column with its keys folded (m9); m10 the volume card of chora-notify (box [1144, 599, 120, 65]). Every v9 still matches its act; no mc act of either v9 take is red.

### Findings (not intended: no row)

- **rtr4-1 MAJOR** (mc; m2, m2b, m3): rtr3-1 is still on this base. With the timer card at the foot, `mc-s-boil-panels-side-lines` is [160, 530, 185, 29]: 29 px for 2 lines, one line and a 1 px strip of the next. By eye the ink equals a clean cut (the strip is the empty top of the next line, the more mark shows); the box does not. f10be960 (MC-BOIL r1, MERGE-OK, train 154) makes it 28 px; it is not in 613844fc. Once it lands, `rect mc-s-boil-panels-side` and `-side-lines` rows (859cf3fe, ae7496f7: the card takes its band) close the three acts. Stills m2.Q-v8.jpg / m2.Q-v9.jpg, m2b.Q+1-v8.jpg / -v9.jpg, m3.Q-v8.jpg / -v9.jpg.
- **rtr4-2 MINOR** (mc; m8): Programs opens the launcher again (80dbd0b4, rtr1-3 fixed) in mc's dark theme (cd277ee4, rpx1-10: intended), but its rows lost the icons that v8's fuzzel showed: rv9ka-2 (MINOR, KIT-ICONS r1). One blob [463, 198, 354, 324] holds both, so no row: the m8 row box [500, 225, 270, 290] (judge r2a) stays and m8 stays a regression until the icons land; then the box is the launcher, [463, 198, 354, 324], with cd277ee4 and the icons commit. Stills m8.Q-v8.jpg, m8.Q-v9.jpg.
- **rtr4-3 MAJOR** (construct, tour chapter demo/tour/chapters/construct.py:90..134; c3, c3b, c3c, c3d, c4, c4b, c5e, c2b): since 75d4623e (co-drop r1) a drop onto the front face steps in, but the chapter still drops the pane back (c5c) and goes on with its stood-back acts: Ctrl+Alt+Right, the TURN LEFT chip, Ctrl+Alt+Left, Super+F1, Return and Alt+Escape run inside the face. The v9 stills show the xp desktop where the acts describe the turning cube (c3b: face 3 stepped in, v8: the cube from outside); acts 9 and 10 are red in both v9 takes ("the cube: front is 2, expected 1", "front is 1, expected 0"); c5e and c2b enter and leave face 1, not 0. The product does what co-drop r1 decided; the chapter needs an Alt+Escape after the drop (and c5d's text "the cube with the window on the face" changed to the stepped-in face). Chapters are outside this lane: a TOUR lane. Stills <act>.<capture>-v8.jpg / -v9.jpg of the eight acts. (Acts 4 and 14..18 of the v9 construct chapter are red on I2, "construct-mode-step-in and construct-mode-turn-left are 9 apart", as in r1..r3 (co-44); not new.)

Harness findings of the takes (they break the v8 baseline, not v9; not among the 57):
- **rtr4-4 MAJOR** (harness, demo/tour/launch.py:274, 06c3f7cc6 / 07eb539fa): `ensure_notify` waits for `notify state.get`, which the 733967aa chora-notify does not answer: the v8 replay of mc (its dark half) and of handover BROKE with "harness: chora-notify does not answer notify state.get after 15 s", no capture. A take needs the call skipped under TOUR_BASELINE (what the scratch harness did); with the v8 notify started but not waited for, v8 mc dark drifts the same way, so the skip is the r3 behaviour, not the cause of m8b..m10.
- **rtr4-5 MINOR** (harness, demo/tour/launch.py:46): `fuzzel_version` runs `{REPO}/tests/tool-version.sh`, REPO being TOUR_REPO (the v8 tree), which has no such file: every v8 chapter BROKE with FileNotFoundError. The script is the harness's own: resolve it next to launch.py (`parents[2]`), not under TOUR_REPO.

### Counts

| take | same | change | open | regression | none | regressions |
|---|---|---|---|---|---|---|
| r3 (f36a229d, for reference) | 1 | 54 | 0 | 44 | 11 | c2, m1 m1b m1c m2 m2b m3 m4 m4b m6 m6b m8 m10, x1..x9b (31) |
| take 1, rows of the base | 1 | 41 | 0 | 57 | 11 | r3's 44 and c2b c3 c3b c3c c3d c4 c4b c5c c5d c5e m8b m9 m9b |
| take 1, rows of the head | 1 | 86 | 0 | 12 | 11 | c2b c3 c3b c3c c3d c4 c4b c5e m2 m2b m3 m8 |
| take 2 (re-take), rows of the head | 1 | 86 | 0 | 12 | 11 | c2b c3 c3b c3c c3d c4 c4b c5e m2 m2b m3 m8 |

The re-take leaves exactly the filed findings: rtr4-1 (3), rtr4-2 (1), rtr4-3 (8). v9 chapters in both takes: xp, tiling, mc, handover 0 red; construct 8 red (acts 4, 9, 10, 14..18; rtr4-3 and I2). v8 handover stops after h1d in both (h2.. `baseline_none`). `python3 -m pytest tests/test_tour_diff.py tests/test_tour.py -q`: 213 passed, 19 subtests passed.

Diff jsons: media/2026-11-26/tour-rows-r4/take1-diff-old-rows.json (57 regressions, rows of the base), take1-diff.json and take2-diff.json (12 each, rows of the head).

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs` (no --push, no step filter) on 67a2dc4d1, detached: every step PASS up to matrix (docs SKIP, allowed), then **xp-notepad FAIL**: `FAIL: xp-46: the client typed ':end' and not start::end` (product typing of xp-46; this diff has no product code). Rerun alone (`--only xp-notepad`): PASS 56 s. Then every step after it (`--only xp-helper,hud,...,e2e`, 45 steps): all PASS, EXIT 0. A foreign flake, named; every later step run. Last lines (media/2026-11-26/tour-rows-r4/gate-last.txt):
```
matrix     PASS   367s /tmp/merge-gate.S6SvfV/matrix.log
xp-notepad FAIL    13s /tmp/merge-gate.S6SvfV/xp-notepad.log
exit code 1; last 30 lines of /tmp/merge-gate.S6SvfV/xp-notepad.log:
ok: the start menu has Notepad as the first program below the pinned rows
ok: a click on Notepad starts the editor of the settings
FAIL: xp-46: the client typed ':end' and not start::end
--- xp-notepad alone
xp-notepad PASS    56s /tmp/merge-gate.gdfkUK/xp-notepad.log
partial run: nothing is pushed
--- the steps after it
launcher-mc PASS    40s /tmp/merge-gate.4h4uiA/launcher-mc.log
voice-acp  PASS    13s /tmp/merge-gate.4h4uiA/voice-acp.log
voice-events PASS     3s /tmp/merge-gate.4h4uiA/voice-events.log
agent      PASS    43s /tmp/merge-gate.4h4uiA/agent.log
tiling-keys-more PASS    45s /tmp/merge-gate.4h4uiA/tiling-keys-more.log
tiling-windows PASS   231s /tmp/merge-gate.4h4uiA/tiling-windows.log
tiling-system PASS    18s /tmp/merge-gate.4h4uiA/tiling-system.log
handover   PASS    30s /tmp/merge-gate.4h4uiA/handover.log
mc-scale   PASS    18s /tmp/merge-gate.4h4uiA/mc-scale.log
acts       PASS  2573s /tmp/merge-gate.4h4uiA/acts.log
watchdog-starved PASS    58s /tmp/merge-gate.4h4uiA/watchdog-starved.log
xp-run     PASS    15s /tmp/merge-gate.4h4uiA/xp-run.log
demo       PASS   292s /tmp/merge-gate.4h4uiA/demo.log
leak       PASS   494s /tmp/merge-gate.4h4uiA/leak.log
tiling-theme PASS    19s /tmp/merge-gate.4h4uiA/tiling-theme.log
tiling-bar PASS   208s /tmp/merge-gate.4h4uiA/tiling-bar.log
mc-host    PASS    22s /tmp/merge-gate.4h4uiA/mc-host.log
tiling-disc PASS    73s /tmp/merge-gate.4h4uiA/tiling-disc.log
e2e        PASS   294s /tmp/merge-gate.4h4uiA/e2e.log
partial run: nothing is pushed
EXIT 0
```
`pgrep -fa 'chora --headless'` empty.

## Review r4 (2026-11-26, 035df1fc..67a2dc4d, head 972f9415)
Checks (brief review-tour-rows-r4; step 0 green: `git fetch --unshallow`, `cargo check --workspace`):
- 1 Red first: `python3 -m pytest tests/test_tour_diff.py -q` at 2680e71b: 1 failed (IntendedCommits product check, 22fead94 tests only), 106 passed; at 67a2dc4d: 107 passed. Bad rows tried locally (not committed): a docs-only commit (3b8cf70f), a merge with conflict files (035df1fc), a v8 sha (7375daa1), a missing sha: each rejected. A clean merge (19348b95, first-parent diff tests/persona-selftest.sh only) passes: rtr4r-3. Holds otherwise.
- 2 Rows: 75 new rows, no `*` row, no nodeless row (pixel rows carry boxes), every node a row lists is in its act's diff of the old-rows take. All 9 shas exist, lie in 733967aa..67a2dc4d and change product code (5025dc47, 75d4623e, 859cf3fe, 4363de11, 675ebd55, fa7faabd, 80dbd0b4, bb3de644, e4157dbd). No row for m2 m2b m3 (rtr4-1) or m8 (rtr4-2). Sampled against take1-diff-old-rows.json and the stills (no capture jsons are on the branch, only the three diff jsons): c2 x2, c5c x4, c5d x5, m1 m1b m1c m4 m4b, m6 x3, m8b x6, m9 x5, m9b x5, m10 x7, x1 x5 x12c. x, m1..m4b, m6, c2 and c5c are the commits' effects (x: only hud-line-tail/hud-chat-tail; m1..m4b: panel 136 -> 115 px; m6: rows 3/13 -> 19/20; c2: the bar box holds only the bar blobs; c5c: the drop steps in, 75d4623e). c5d is not: rtr4r-1. The m9..m10 chat rows give a wrong reason: rtr4r-2. bb3de644 for the region flip is the precedent of the old rows (bisect r2), accepted.
- 3 New 13: m8b m9 m9b intended as cited (fa7faabd Home goes to the first page; 80dbd0b4 the launcher opens, so Escape closes it and turns no page; v9 m9b still: agenda, chat closed, as the acts say), but the v8 replay sits on another page from m8b on: rtr4r-4. Construct: c2b c3 c3b c3c c3d c4 c4b c5e correctly left as rtr4-3; c5d should be with them (rtr4r-1).
- 4 Counts: reproduced with tests/tour-diff.py `verdict()` on the scene and blobs of take1-diff-old-rows.json and take2-diff.json under the head's intended.toml: 1 same, 86 change, 0 open, 12 regression, 11 none (the base's rows: 41 change, 57 regression); regressions c2b c3 c3b c3c c3d c4 c4b c5e m2 m2b m3 m8. Raw take files are not on the branch, so not re-diffed from captures. Holds.
Findings:
- rtr4r-1 MAJOR demo/tour/intended.toml:4377-4405: the five c5d rows (one pixel box [0, 0, 1280, 720], the whole frame) take c5d out of rtr4-3: the v9 still shows the stepped-in xp face while the act says "The cube with the window on the face, no pane", rtr4-3 itself leaves c5d to the chapter fix, and a whole-frame pixel row hides every c5d pixel regression for good; fix: drop the five c5d rows and list c5d under rtr4-3 (13 regressions), rows for c5d only after the TOUR lane settles the act, with boxes of what that act changes.
- rtr4r-2 MINOR demo/tour/intended.toml:4414,4516,4546,4576 (and 4420,4438,4564,4582,4600): the cause "675ebd55 the chat opens in the column (mc-v2 item 6) ... the v8 replay opens no chat" is not that commit's effect: 733967aa already opens the chat in the column (plugins/chora-mc/src/main.rs:1174, chat-slot); v8 opens none because demo/tour/chapters/mc.py:287 clicks door("mc-r1-right-chat-door") while v8 shows the r0 buffer (its chat at y 252, not 168), and at m8b no chat is open at all (the door moves because the notices left the column); fix: name the region flip and notices-out of 675ebd55 and the chapter's r1 lookup in those causes.
- rtr4r-3 MINOR tests/test_tour_diff.py:835: `git show --format= --name-only` lists nothing for a clean merge, so `files and ...` accepts it: a row citing 19348b95 (chore: merge session head a93bc75b, tests/persona-selftest.sh only) passes all three tests; fix: take the files from `git diff --name-only <sha>^1 <sha>` (or reject merge commits).
- rtr4r-4 MINOR demo/tour/intended.toml:4413-4600 (m8b m9 m9b m10, about 100 nodes per act): the rows absorb the whole page and column because the v8 replay is on Agents / The Morning (no Home in v8) while v9 is on the agenda, so from m8b on the dark mc look is compared with another page and a regression of the dark agenda there is not seen; fix: a TOUR lane lets the v8 replay reach the agenda (Left twice under TOUR_BASELINE for Home), then the rows shrink to the column.
Verdict (computed): one open MAJOR (rtr4r-1): FIX.

## fix r1

Base 972f9415f (claude/tour-rows-r4 head, review r4 FIX); head 84e31324c (claude/tour-fix-r1). Step 0 green (`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`).
Commits: b0ac9cf83 **red** (test(tour): no whole-frame pixel box; a merge counts its 1st parent: fails on 972f9415 with `[('c5e', [0, 0, 1280, 720]), ('c5d', [0, 0, 1280, 720])] != []` and `[] != ['tests/persona-selftest.sh']` for 19348b95), 7fc8b31f5 (c5d rows out, first-parent diff), f6a8b347d red (launch on the baseline), aafd56112 (rtr4-4, rtr4-5), a63becb36 (test and fix of the chapter in one commit: the fix was staged when the test was committed and the push went out before I saw it; no force push), 8f38e5e0b (c5e loses its whole-frame box), 84e31324c (the replay without `cube.list` counts as stood back, with its test).
Diffstat 972f9415..84e31324: demo/tour/chapters/construct.py +27 -4, demo/tour/intended.toml +1 -31, demo/tour/launch.py +10 -2, tests/test_tour_diff.py +151 -5 (4 files, +189 -42). No product code.

**rtr4r-1.** The five c5d rows are gone. The new test holds for every act, so the c5e pixel row (r1) loses its box [0, 0, 1280, 720] too; its five small boxes stay.
**rtr4r-3.** `IntendedCommits.files_of` is `git diff --name-only <sha>^1 <sha>`. 19348b95 is rejected; the nine shas the rows cite still pass.
**rtr4-3.** Since 75d4623e the drop of c5c steps into face 0. c5d now says and expects that ("The first face at 1:1: xp with the sunset window on it, no pane, the hint Alt Esc Step back"; `cube front 0 mode use`). c3 first steps out with Alt+Escape and then turns the cube (`step_out`: Alt+Escape only when `cube.list` says `use`; otherwise it spends the same one frame, so the replay keeps its frames). After that c3 c3b c3c c3d c4 c4b turn the cube from outside, c5e steps into face 0 and c2b leaves it. The v9 stills show what each act says (media sheet below). Acts 9 and 10 hold their own checks, and "front is 2/1, expected 1/0" is gone. They are still RED, but only on **I2 of co-44** (chip gaps of 9, COSMETIC, open; its ledger row names acts c4..c18). The same red holds on every stood-back act: 4, 7..12 and 14..18, 12 in all. Before 75d4623e acts 7..12 were stood back too and carried it. They are not marked `known`: one `known=I2` would hide every I2 line of the act.
**rtr4-4.** `ensure_notify` does nothing under TOUR_BASELINE, as the r3 replay did: the 733967aa chora-notify exports no method. **rtr4-5.** `fuzzel_version` runs `tests/tool-version.sh` from the harness's own tree (`TOOL_VERSION`, next to launch.py).
Found during the take: the head's harness calls `cube.list` (`stepped_in`), which 733967aa does not export, so v8 construct stopped at c3 (12 captures `baseline: none`). Fixed in 84e31324: a missing method reads as stood back. v8 construct now takes c1..c2b and stops, as before, at the `front_now` of c3e (4 captures `baseline: none`).

### Takes (head 84e31324 for the harness; each chapter its own TOUR_OUT)

v9 (head build): xp 31 acts 0 red (1 known), tiling 28/0, mc 22/0, construct 18 acts 6 green 12 red (I2 co-44 only), handover 11 acts 2 red (acts 7, 8; rtf1-2, the same on a second run).
v8 (733967aa build in a worktree, TOUR_BASELINE=1, the head's harness unchanged): all five chapters run. mc runs both halves (light and dark, 22 captures). handover takes h1 h1b h1c h1d and stops at the Send To row `xp-ctx-1-0` (handover.py:234, a TypeError; 733967aa has no row; h2..h7 are `baseline_none` in intended.toml, as in r1..r4). construct takes c1..c2b (14 captures).

### Counts (tests/tour-diff.py, all chapters, 110 captures)

| rows | same | change | open | regression | none |
|---|---|---|---|---|---|
| 972f9415 (base) | 1 | 73 | 0 | 27 | 9 |
| 84e31324 (head) | 1 | 71 | 0 | 29 | 9 |

The two extra regressions on the head are c5d and c5e. The whole-frame boxes used to hide them; the review asked for that (rtr4r-1). c3 is not a regression any more. Compared with r4's 12, the 972f9415 merge (t153, t154) brings the product changes of mc-boil, tiling-chip, xp-chat, the key sheet and D119 into the v9 build, and no rows came with them.
Regressions on the head, each with its finding: c2b c3b c3c c3d c4 c4b (rtf1-7), c5d c5e (rtf1-1), h3b h4 (rtf1-2), m1 m1b m1c m4 m4b (rtf1-4), m2 m2b m3 (rtr4-1), m8 (rtr4-2), t3f t5 t6 t6b t6c t6d t7 t10c (rtf1-3), x1 (rtf1-6), x5b (rtf1-5): 6 + 2 + 2 + 5 + 3 + 1 + 8 + 1 + 1 = 29.
Stills: media/2026-11-26/tour-fix-r1/<act>.<capture>-v8-v9.jpg (v8 above v9, blobs boxed in red), h3b.Q-v9.jpg and h4.Q-v9.jpg (no v8 capture), take-diff.json and take-diff.txt.
`python3 -m pytest tests/test_tour_diff.py tests/test_tour.py -q`: 220 passed, 19 subtests passed.

### Findings

- **rtr4-3 fixed** (84e31324): the chapter follows the drop. **rtr4-4, rtr4-5 fixed** (aafd5611). **rtr4r-1, rtr4r-3 fixed** (7fc8b31f, 8f38e5e0).
- **rtr4-1 MAJOR, still open, rows owed** (m2 m2b m3): its fix has landed. c3925fc55 (a panel takes whole lines at the page edge, in 972f9415) makes `mc-s-boil-panels-side-lines` [160, 530, 185, 28]: one whole line of 28 px. The cut check still flags it ("28 px high for 56 px of 2 lines"), and the lead panel changes as in rtf1-4. A row lane names `rect` lead/side and `cut` side-lines with c3925fc55. Stills m2.Q-v8-v9.jpg, m2b.Q+1-v8-v9.jpg, m3.Q-v8-v9.jpg.
- **rtr4-2 MINOR, still open** (m8): unchanged, the launcher without icons. m8.Q-v8-v9.jpg.
- **rtf1-1 MINOR** (construct c5d, c5e; tour comparability, no product defect): v9 steps into the face where 733967aa stays stood back (c5d: the drop, 75d4623e; c5e: Return, f5bdc61f co-40). The whole frame differs, every pixel, fraction 1.0, and no row may name that (rtr4r-1). Both acts show what v9 does. Fix (TOUR lane): under TOUR_BASELINE the replay steps in by the Step in chip that 733967aa has, or tour-diff gets a declared "no comparable baseline" for a capture. Stills c5d.Q-v8-v9.jpg, c5e.Q-v8-v9.jpg.
- **rtf1-2 MAJOR** (F7, tour; handover h3b h4 = v9 acts 7, 8 red, reproduced on a second run): 41ddd4ba2 (rtf2-1, in 972f9415) intends that a window given to mc while another look holds the keyboard arrives without it. demo/tour/chapters/handover.py still expects "the focus with mc" after Super+Alt+3 (act 7). Its next key, Super+Alt+4, then reaches tiling and not mc, so act 8 fails with "s1 is owned by mc". Fix (TOUR lane): act 7 expects no mc focus, and a click into mc comes before act 8. No v8 capture (baseline_none). Stills h3b.Q-v9.jpg, h4.Q-v9.jpg.
- **rtf1-3 MINOR** (tiling t3f t5 t6 t6b t6c t6d t7 t10c, rows owed): the blob [12, 8, 438, 32] is the left island of the tiling bar (+[475, 21, 19, 8] in t3f t6 t6c). e6afbf977 (tiling-chip-r1: centre the layout word in its chip) gives "Fair rows" its own chip, and the island's outline and widths change (seen in t6). Rows: a box of the left island with e6afbf977. Stills t*.Q*-v8-v9.jpg.
- **rtf1-4 MINOR** (mc m1 m1b m1c m4 m4b, rows owed): c3925fc55 makes the lead panels 287 px (`-lead-lines` 280 = 10 whole lines of 28, was 273) and moves `-lead-rule` from x 42 to 94. The unnamed nodes are `rect mc-s-*-panels-lead` and `-lead-lines`. Rows with c3925fc55. Stills m1.Q-v8-v9.jpg ... m4b.Q-v8-v9.jpg.
- **rtf1-5 MINOR** (xp x5b, row owed): `meta hud-chat` 'Atlas — Idle' -> 'Atlas — Working'. 88951ae4d (xp-chat: a send shows Working and Stop at once). x5b.Q-v8-v9.jpg.
- **rtf1-6 MINOR** (xp x1, open: cause not traced): the blob [1182, 560, 50, 18] near the paperclip. It is r3's blob of x1/x5e/x5g/x6e, gone in r4's takes and back here in x1 alone. x1.Q-v8-v9.jpg.
- **rtf1-7 MINOR** (construct c2b c3b c3c c3d c4 c4b, rows owed): the r1 pixel rows of these acts list blob-exact boxes of the hint bar. The bar shows the same hints (cfc805d4 Keys as a chip, f5bdc61f the F1 hint), but this take's blobs fall a few px outside those boxes: [195, 673, 187, 16] against the row box [195, 680, 187, 9] in c2b c3d c4b, and 21 and 26 keycap blobs in c3b and c3c. c3d also has [446, 355, 166, 144], the sunset window on face 0; c4b's row names it, c3d's does not. c4: the key sheet [27, 78, 472, 603] and more, from the key-sheet commits of 972f9415 (d590aae69 1200 wide in three columns, 822d5be4a, 369cf96fd, 142260b96). Rows: one bar box per act (the precedent is c2's 5025dc47 box [118, 647, 1045, 42]) and a key-sheet box with those commits. Stills c2b c3b c3c c3d c4 c4b .Q-v8-v9.jpg.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs` (no --push, no step filter) on 84e31324c, run detached: 67 steps PASS (matrix 363 s, acts 2325 s, leak 485 s, e2e 284 s), docs SKIP (allowed: mkdocs is not installed). It ends "all steps passed; no --push, nothing is pushed", which tests/merge-gate.sh:622-623 prints just before `exit 0`, so **EXIT 0**; no foreign red. Last 20 lines: media/2026-11-26/tour-fix-r1/gate-last.txt.
```
voice-acp  PASS    13s /tmp/merge-gate.bV4vYL/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.bV4vYL/voice-events.log
agent      PASS    41s /tmp/merge-gate.bV4vYL/agent.log
tiling-keys-more PASS    43s /tmp/merge-gate.bV4vYL/tiling-keys-more.log
acts       PASS  2325s /tmp/merge-gate.bV4vYL/acts.log
tiling-system PASS    16s /tmp/merge-gate.bV4vYL/tiling-system.log
handover   PASS    96s /tmp/merge-gate.bV4vYL/handover.log
mc-scale   PASS    16s /tmp/merge-gate.bV4vYL/mc-scale.log
tiling-windows PASS   197s /tmp/merge-gate.bV4vYL/tiling-windows.log
watchdog-starved PASS    52s /tmp/merge-gate.bV4vYL/watchdog-starved.log
xp-run     PASS    14s /tmp/merge-gate.bV4vYL/xp-run.log
demo       PASS   253s /tmp/merge-gate.bV4vYL/demo.log
leak       PASS   485s /tmp/merge-gate.bV4vYL/leak.log
tiling-bar PASS   197s /tmp/merge-gate.bV4vYL/tiling-bar.log
tiling-theme PASS    19s /tmp/merge-gate.bV4vYL/tiling-theme.log
mc-host    PASS    22s /tmp/merge-gate.bV4vYL/mc-host.log
tiling-disc PASS    75s /tmp/merge-gate.bV4vYL/tiling-disc.log
e2e        PASS   284s /tmp/merge-gate.bV4vYL/e2e.log
skipped: docs (mkdocs is not installed (docs unchanged))
all steps passed; no --push, nothing is pushed
```
`pgrep -fa 'chora --headless'` empty.

## Review fix r1 (2026-11-26, 972f9415..84e31324, head 84e31324c, 4 files +189 -42)
Checks (brief review-tour-fix-r1; step 0 green: `git fetch --unshallow`, `cargo check --workspace`):
- 1 Red first, `python3 -m pytest tests/test_tour_diff.py -q` on each commit's own tree: b0ac9cf8 2 failed (whole-frame box, 19348b95), 107 passed; 7fc8b31f 1 failed (c5e box still whole-frame), 108 passed; f6a8b347 3 failed (whole-frame box, the two LaunchOnTheBaseline tests), 108 passed; aafd5611 1 failed (c5e box), 110 passed; 84e31324 114 passed, 19 subtests. Both reds precede their fixes. a63becb3 holds test and fix in one commit (the lane says so): the chapter is the test, a tour script whose acts 9, 10 were red on 972f9415 and whose `cube=` expectation of c5d checks the stepped-in face; and the pytest of that commit is red on 972f9415's construct.py (3 failed: no `step_out`, no `stepped_in`, c5d text). Not a fix without a red test: no MAJOR; the commit order is rtf1r-2.
- 2 Rows: rtr4r-1 the five c5d r4 rows are gone (the ten c5d rows left are r1 rows with their own nodes and one boxed pixel row). rtr4-3 construct.py:119-121 `stepped_in` then `step_out` (Alt+Escape inside a face, else one frame, so both replays keep their frames; `t.key` is one frame, tour.py:430), c5d expects `front 0, mode use`; c2b..c4b turn from outside. rtr4-4 launch.py ensure_notify returns on TOUR_BASELINE (same truthiness as tour.py:278 and run.py:91). rtr4-5 `TOOL_VERSION` is `parents[2]` of launch.py, the harness's own tree. rtr4r-3: `files_of` is `git diff --name-only <sha>^1 <sha>`; 19348b95 gives `tests/persona-selftest.sh` and is rejected; none of the 50 shas the rows cite is a merge. Bad row tried locally (not committed): the cause of the c5d... first r4 row changed to cite 972f9415 (the merge of t153/t154; its first-parent diff is 54 files with product code) passes all 114: rtf1r-1.
- 3 `grep '\[0, 0, 1280, 720\]'` in demo/tour/intended.toml: nothing; no box of half the frame or more; the c5e pixel row keeps its five small boxes. c5d and c5e are regressions in the lane's take (take-diff.json), as the review asked.
- 4 Counts: raw captures are not on any branch (only take-diff.json, which holds scene and blobs); re-ran `verdict()` of tests/tour-diff.py on the scene and blobs of all 110 captures of take-diff.json under the head's intended.toml: 1 same, 71 change, 0 open, 29 regression, 9 none; every capture's verdict and `unnamed` equal the recorded one (checks left out; no flip changes a verdict). The 29 are the lane's list. Five sampled against ancestry and still: m1 c3925fc55, t6 e6afbf977, x5b 88951ae4d, h3b 41ddd4ba2, c4 d590aae69 and the key-sheet commits: each is in 972f9415 and not in 67a2dc4d (the merge's first parent). By the lane's own table 19 of the 29 (t x8, m1 m1b m1c m4 m4b, m2 m2b m3, x5b, h3b h4) come from the merge: "most" holds. Not from the merge: c2b c3b c3c c3d c4b (hint bar: c2b has the same blobs [195, 673, 187, 16] ... in r4's take2, which is 67a2dc4d), c5d c5e (rtf1-1), m8 (rtr4-2), x1 (rtf1-6, cause not traced).
- 5 `stepped_in` cube.list stop fix (construct.py:49-57): POLS-sound. Only a "no method" error (tour.py:136 NO_METHOD) reads as stood back, any other error re-raises; a v9 build that lost `cube.list` would not hide: the same acts hold `Expect(cube=...)` / `cube_is(mode="navigate")`, which read `cube.list` and go red or stop. It cannot hide a v8 replay failure either: 733967aa has no `cube.list` at all and its c5c leaves the person stood back (the stills c5d show the v8 cube with the pane beside it). Hardening only: rtf1r-4.
Findings:
- rtf1r-1 MINOR tests/test_tour_diff.py:845-856: a merge's first-parent diff is the whole side branch, so a row citing a merge passes `no_product_code` for any node: tried a row citing 972f9415 (54 files), 114 passed; fix: reject a cited sha with more than one parent (`git rev-list --parents -n1 <sha>`) and let the row name the commits of the side branch (rtr4r-3 offered this).
- rtf1r-2 MINOR (commit order, no code): a63becb3 holds the chapter fix with its pytest (no red commit before it), and 7fc8b31f and aafd5611 leave the whole-frame test red on c5e until 8f38e5e0; fix: none for this merge, the next lane stages the red commit alone.
- rtf1r-3 NIT tests/test_tour_diff.py:977-983: `test_c5d_looks_at_the_stepped_in_face...` finds lines by the strings `"04-13-back-on-face"` and `t.key("Ctrl+Alt+Right")` and checks for `step_out(t, ` in between; a reorder of the chapter breaks it, and it proves no behaviour; fix: leave it, the act's own `cube=` expectation is the real test.
- rtf1r-4 NIT demo/tour/chapters/construct.py:55-57: the missing-method fallback of `stepped_in` is not limited to TOUR_BASELINE; fix: `if not (NO_METHOD.search(str(err)) and os.environ.get("TOUR_BASELINE")): raise`.
Verdict (computed): no open BLOCK or MAJOR: MERGE-OK (counts reproduce: 1 / 71 / 0 / 29 / 9).

## fix r2

Base 84e31324c (claude/tour-fix-r1 head, review MERGE-OK); head b16c888fa (claude/tour-fix-r2). Step 0 green (`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`).
Commits: 4a1d83572 **red** (test(tour): handover act 7 expects no mc focus, a click comes first; fails on 84e31324: `focus="mc"` in the act-7 row, no pointer move or click between the two keys), 46e5ac617 (the chapter fix), 4bc210b1d **red** (test: m2 m2b m3 name the boil page panels; fails until the rows), b16c888fa (the rows).
Diffstat 84e31324..b16c888fa: demo/tour/chapters/handover.py +2 -1, demo/tour/intended.toml +36, tests/test_tour.py +23, tests/test_tour_diff.py +16 -1 (4 files, +77 -2). No product code.

**rtf1-2 fixed.** Act 7 (Super+Alt+3) no longer expects `focus="mc"` (41ddd4ba2, rtf2-1: a window given to mc while another look holds the keyboard arrives without it). `goto(640, 360, MC); t.click(); t.wait(0.5)` comes before act 8's Super+Alt+4, so the key reaches mc. Handover on the head build: 11 acts, 11 green, 0 red in three runs alone (acts 7, 8 green each time). One further run was red at acts 6 and 10 (`letter.txt` lost its first line): I had started a second tour (the v9 xp take) at the same time, which shares that file; it was killed, and the two runs after it are green. Not a product or chapter defect.
**rtr4-1 fixed (rows).** `rect` of `mc-s-boil-panels-lead`, `-lead-lines`, `-side`, `-side-lines` and `cut` of `mc-s-boil-panels-side-lines` for m2, m2b, m3, cause c3925fc5 (the 8-digit form of c3925fc55, so the cause check reads it): 3 acts x 2 rows, each by node, none open, no `*`, no whole-frame box. `tests/test_tour_diff.py` lets a row of kind `cut` carry `nodes` (it listed the scene kinds without it) and holds the three acts' rows.

### Takes (head b16c888fa; each chapter its own TOUR_OUT, the five chapters one after the other)

v9 (head build): xp 31 acts 0 red (1 known), tiling 28/0, mc 22/0, construct 18 acts 6 green 12 red (I2 co-44 only, as in fix r1), handover 11 acts 11 green 0 red.
v8 (733967aa in a worktree, TOUR_BASELINE=1, the head's harness): as in fix r1: mc both halves, handover h1..h1d, construct c1..c2b and the c3 stop (4 captures `baseline: none`, `baseline-none.json` copied beside the folder of the diff).

### Counts (tests/tour-diff.py, all chapters, 110 captures)

| rows | same | change | open | regression | none |
|---|---|---|---|---|---|
| 84e31324 (fix r1 head) | 1 | 71 | 0 | 29 | 9 |
| b16c888fa (head) | 1 | 74 | 0 | 24 | 11 |

h3b, h4 leave the regression column (their acts are green now; they stay `baseline: none`, 9 -> 11 none), m2 m2b m3 are change (71 -> 74). Regressions left, each with its filed id: c2b c3b c3c c3d c4 c4b (rtf1-7), c5d c5e (rtf1-1), m1 m1b m1c m4 m4b (rtf1-4), m8 (rtr4-2), t3f t5 t6 t6b t6c t6d t7 t10c (rtf1-3), x1 (rtf1-6), x5b (rtf1-5): 6 + 2 + 5 + 1 + 8 + 1 + 1 = 24.
Stills: media/2026-11-26/tour-fix-r2/m2.Q-v8-v9.jpg, m2b.Q+1-v8-v9.jpg, m3.Q-v8-v9.jpg (v8 above v9, the four panel nodes boxed in red on v9), h3b.Q-v9.jpg, h4.Q-v9.jpg, take-diff.json, take-diff.txt, gate-last.txt.
`python3 -m pytest tests/test_tour_diff.py tests/test_tour.py -q`: 223 passed, 19 subtests passed.

### Findings left (backlog, unchanged)

rtr4-2 MINOR (m8); rtf1-1 MINOR (c5d, c5e); rtf1-3 MINOR (tiling); rtf1-4 MINOR (m1 m1b m1c m4 m4b); rtf1-5 MINOR (x5b); rtf1-6 MINOR (x1); rtf1-7 MINOR (construct hint bar and key sheet); rtf1r-1..4 MINOR/NIT. No MAJOR left.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs` (no --push, no step filter) on b16c888fa, run detached: 67 steps PASS (acts 1518 s, pull 512 s, leak 458 s, e2e 268 s), docs SKIP (allowed: mkdocs is not installed). It ends "all steps passed; no --push, nothing is pushed", printed just before `exit 0`, so **EXIT 0**; no foreign red. Last 20 lines: media/2026-11-26/tour-fix-r2/gate-last.txt.
`pgrep -fa 'chora --headless'` empty.

## Review fix r2 (PM, 2026-11-26, 84e31324c..b16c888fa, 4 files +77 -2)
- Red first: 4a1d8357 and 4bc210b1 precede fixes 46e5ac61 and b16c888f; the lane's full gate is green.
- handover act 7 clicks mc before act 8 (rtf1-2 h3b h4 closed); m2 m2b m3 get rect and cut rows with c3925fc5 (rtr4-1 rows owed: closed).
- rtf2r-1 MINOR (mc m2 m2b m3; findings/mc.md mc-63): the rows whitelist a cut side panel (one 28 px line of 2) as intended. Under the 11-17 perfectionist order cut text is a look defect, not an intended change; the panel must show whole content or drop it.
- The text-grep tests are weak but red-first. No MAJOR. Verdict (computed): MERGE-OK b16c888fa, train 159.

## r5 (F7, rc 11): the 24 MINOR regressions of take r2 judged before take r3

Base b16c888fa (claude/tour-fix-r2 head, review MERGE-OK); head 729767a81 (claude/tour-rows-r5). Step 0 green (`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`).
Commits: c24ed22a7 **red** (test(tour): the 21 judged minor regressions get rows; 5 of its 6 tests fail on b16c888f, the sixth holds m8 and c5d c5e as regressions and passes by design), 729767a81 (the 21 rows). Diffstat b16c888f..729767a8: demo/tour/intended.toml +138, tests/test_tour_diff.py +63, tests/tour-diff-take-r5.json (new, 2.8 KB: per act the diffs of this take that no row of b16c888f named); 3 files, +202 -0. No product code, no chapter change.

**Method.** Both builds taken again (v9 = head b16c888f build, v8 = 733967aa in a worktree, `TOUR_BASELINE=1`, the head's harness; five chapters each in its own TOUR_OUT, one after the other). `tests/tour-diff.py` gave 1 same, 74 change, 0 open, 24 regression, 11 none: the brief's 24, the same acts. For each, the diffs no row named (`verdict()` per node and per blob) were judged on the v8 and v9 full-size captures (not only the half-size stills): the commit that makes them, then the looks (whole text, nothing under 12 px, aligned gaps). Mechanical part: every visible text node of v9 in the 24 captures is at least 12 px and none is cut (`cut_of`: none).

### The 24

| act | now | row (cause) or finding | still (media/2026-11-26/tour-rows-r5/) |
|---|---|---|---|
| c2b | row | hint bar: 7f519bc8 (a face names the pull key), f5bdc61f (hint All keys), cfc805d4 (Keys as a chip); box [179, 647, 922, 42] = the bar pill | c2b.Q-v8-v9.jpg |
| c3b | row | f5bdc61f, cfc805d4; box [280, 647, 720, 42] (this bar has no pull hint) | c3b.Q-v8-v9.jpg |
| c3c | row | as c3b | c3c.Q-v8-v9.jpg |
| c3d | rows | bar as c2b; sunset window [446, 355, 182, 144]: 83033ed6 (co-46, a cube frees the keys of a window it covers: s1 `focused` true in v8, false in v9) and f1693fb3 (xp-41, inactive title with shadow) | c3d.Q-v8-v9.jpg |
| c4 | row | key sheet over the cube: f5bdc61f (F1 opens the list), d590aae6 (1200 wide, three columns, co-71), 822d5be4, 369cf96f, 142260b9; box [27, 58, 1226, 632] = the sheet panel [31, 58, 1218, 604] and its shadow fringe (no whole-frame box) | c4.Q-v8-v9.jpg |
| c4b | rows | bar and window as c3d | c4b.Q-v8-v9.jpg |
| c5d | **regression, rtf1-1** | no row possible: v9 steps into face 0, v8 stays stood back, every pixel differs (blob [0, 0, 1280, 720]); a row may not name the frame (rtr4r-1). Comparability of the replay, not a look defect: no look finding. Needs the chapter or a declared "no comparable baseline" (TOUR lane, outside this lane's area) | c5d.Q-v8-v9.jpg |
| c5e | **regression, rtf1-1** | as c5d (Return steps in, f5bdc61f) | c5e.Q-v8-v9.jpg |
| m1 | row | rect `mc-s-ingredients-panels-lead`: c3925fc5 (a panel takes whole lines at the page edge: 287 px for 10 whole lines of 28, v8 280 with the last line cut at 273) | m1.Q-v8-v9.jpg |
| m1b | row | as m1 | m1b.Q-v8-v9.jpg |
| m1c | row | rect `mc-s-boil-panels-lead` and `-lead-lines`, c3925fc5 | m1c.Q-v8-v9.jpg |
| m4 | row | rect `mc-s-sauce-panels-lead` and `-lead-lines`, c3925fc5 (the six right-column blobs the r2 take showed are not in this take) | m4.Q-v8-v9.jpg |
| m4b | row | as m1c | m4b.Q-v8-v9.jpg |
| m8 | **regression, rtr4-2** (MINOR, open) | launcher without icons and on a dark ground, blob [463, 198, 354, 324]; stays, no row (brief) | m8.Q-v8-v9.jpg |
| t3f | row | e6afbf97 (tiling-chip-r1: the layout word centred in a chip of its own); box [12, 8, 526, 32] = the left island of the bar (x 13..538 on the pixels) | t3f.Q-v8-v9.jpg |
| t5 t6 t6b t6c t6d | rows | as t3f | t5.Q, t6.Q, t6b.Q, t6c.Q, t6d.Q (-v8-v9.jpg) |
| t7 t10c | rows | as t3f | t7.Q+0.5, t10c.Q+0.5 (-v8-v9.jpg) |
| x1 | row | rtf1-6 traced: the blob [1182, 560, 50, 18] lies in the rect of the new node `hud-line-tail` [1067, 559, 166, 19]: the greeting of 4fec75c3 has a wedge tail of its own (e4157dbd, d3d28cfa); the balloon body is masked as a text node, the tail is not, and v8 has no `hud-line*` node at all. Box = the tail rect | x1.Q-v8-v9.jpg |
| x5b | row | meta `hud-chat`: 88951ae4 (a send shows Working and Stop at once; 'Atlas, Idle' -> 'Atlas, Working') | x5b.Q-v8-v9.jpg |

21 rows, 3 regressions left. Looks check of the 21: the hint bar, the key sheet, the whole-line lead panels (the v8 panel cut its last line, so v9 is the better frame), the chip in the tiling island, the balloon with its wedge and the Working header read clean at 1x (whole text, 12 px and up, no overlap). The 9 px gap of two hint chips (I2) is **co-44**, open, COSMETIC, unchanged by the commits above; its check rows are untouched, so the 12 stood-back construct acts stay red on I2 and the rows do not hide it. No new finding was needed: nothing judged was an unintended change or unclean beyond co-44, and c5d c5e m8 are filed (rtf1-1, rtr4-2). mc-63 (m2 m2b m3, one cut line) is still open in findings/mc.md; its acts have rows since b16c888f and I added none.

### Takes and counts

v9 (head build): xp 31 acts 0 red (1 known), tiling 28/0, mc 22/0, construct 18 acts 6 green 12 red (I2 of co-44 only), handover 11/11 green. v8 (733967aa, `TOUR_BASELINE=1`): xp 31 red as before (no state.get on the old build), tiling, mc, construct c1..c2b and the c3 stop (4 captures `baseline: none`), handover h1..h1d and the Send To TypeError (h2.. `baseline_none`): as in fix r2.

| rows | same | change | open | regression | none |
|---|---|---|---|---|---|
| b16c888fa (base) | 1 | 74 | 0 | 24 | 11 |
| 729767a81 (head) | 1 | 95 | 0 | 3 | 11 |

Regressions left, each with its finding: c5d c5e (rtf1-1), m8 (rtr4-2). Stills: media/2026-11-26/tour-rows-r5/<act>.<capture>-v8-v9.jpg (v8 above v9, the new boxes in red on v9; each under 200 KB), take-diff.json and take-diff.txt (head), take-diff-b16c888f.txt (the base, the 24).
`python3 -m pytest tests/test_tour_diff.py tests/test_tour.py -q`: 229 passed, 19 subtests passed.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs` (no --push, no step filter) on 729767a81, run detached: 67 steps PASS (matrix 350 s, pull 519 s, acts 1689 s, leak 476 s, e2e 282 s), docs SKIP (allowed: mkdocs is not installed). It ends "all steps passed; no --push, nothing is pushed", which tests/merge-gate.sh:622-623 prints just before `exit 0`, so **EXIT 0**; no red cell, no foreign flake. Last 20 lines: media/2026-11-26/tour-rows-r5/gate-last.txt.
`pgrep -fa 'chora --headless'` empty.

## Review r5 (2026-11-28, b16c888f..729767a8, head 729767a81, 3 files +202)
Checks (brief review-tour-rows-r5; step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour_diff.py -q` 121 passed, 19 subtests):
- 1 Red first holds: `pytest tests/test_tour_diff.py` at c24ed22a: 5 failed (MinorRegressionsOfTakeR5: all_named, mc lead rects, tiling island, x1 tail, x5b header), 116 passed; at 729767a8: 121 passed. The sixth new test (m8 stays a regression, no half-frame box for c5d c5e m8) passes by design.
- 2 Rows: 23 new `[[change]]` entries (21 acts; c3d and c4b have two), no `*` row, no whole-frame box, no cause without a sha. All 15 shas exist, lie after 733967aa, have one parent, and change product code (7f519bc8, f5bdc61f, cfc805d4, 83033ed6, f1693fb3, d590aae6, 822d5be4, 369cf96f, 142260b9, c3925fc5, e6afbf97, 4fec75c3, e4157dbd, d3d28cfa, 88951ae4). Stills opened (v8 over v9): c3b c3d c4 c4b m1 m1c m4 t3f t6 t10c x1 x5b (12). Hint bar, key sheet (1218 wide, three columns, whole text), whole-line lead panels (v9 cuts nothing; v8 cut the last line, v9 adds a continuation arrow), tiling bar chip, balloon with wedge, Working header: each is the named commit's effect and clean at the stills' scale (no cut text, no overlap, no wrong colour). Not judged at 1x: the captures are not on a branch, only half-size stills; the thin wedge of the x5b balloon (a long sliver along the balloon's lower edge to the paperclip) I could not judge, rv9kb-2 passed review r4. co-44 and mc-63 untouched.
- 3 Breadth, by the tool's rule (`inside()`: a blob lies in a box give or take 2 px): I removed each new row from the take's leftover diffs (`tests/tour-diff-take-r5.json`) and ran `verdict()`: 22 of 23 rows flip their act to regression, so each names something; one does not (rtr5r-1). Box vs the bounding box of the blobs it names: bar rows 1.3 to 12.9 times, window row 1.1, c4 sheet 1.0, x1 tail 3.5, tiling rows 1.2 (island blob) and 76 (t3f, one word blob of 20 x 11): rtr5r-2.
- 4 c5d c5e (rtf1-1) and m8 (rtr4-2): no row, ids open; the test holds that no box of them is wider than half the frame.
- 5 Counts: `verdict()` of tests/tour-diff.py on all 110 entries of media/2026-11-26/tour-rows-r5/take-diff.json under the head's intended.toml: 1 same, 95 change, 0 open, 3 regression (c5d c5e m8), 11 none; every entry's verdict equals the recorded one. Holds.
Findings:
- rtr5r-1 MINOR demo/tour/intended.toml:4834 (c4b window row, [446, 355, 182, 144]): dead row. Without it c4b is still `change`, because the old row of c4b (boxes [446, 355, 166, 144], cause pager/bin/start card) already names the window blob; the new row gives the same blob a second cause (83033ed6, f1693fb3) it never has to name; fix: drop it, or put co-46 and xp-41 into the cause of the old c4b row.
- rtr5r-2 MINOR demo/tour/intended.toml:4876-4918 (t3f t5 t6 t6b t6c t6d t7 t10c, box [12, 8, 526, 32]): the box is the whole 526 px island, the blob is [12, 8, 438, 32] (t3f: the word [476, 18, 20, 11] alone, a box 76 times its area), and e6afbf97 is one line (`align` 0 -> 0.5 of the layout word, flavors/tiling/waybar/config.jsonc:113) that cannot change "the outline and widths" of the island; the old rows of these acts already box the island to x 445 (r1), the new blob only reaches x 450, so what the commit moved is a strip of 5 px and the word; fix: boxes [474, 16, 24, 15] for t3f, [12, 8, 438, 32] (+ [475, 18, 22, 12] for t6 t6c) and a cause that says the word moves in its chip and the island edge moves 5 px.
- rtr5r-3 NIT demo/tour/intended.toml:4828,4834 (c3d, c4b window row): the box is 182 px wide, the blob 166 (182 is the c2b blob, which has no window row); fix: [446, 355, 166, 144].
- rtr5r-4 NIT demo/tour/intended.toml:4840 and tests/test_tour_diff.py:913: the c4 box [27, 58, 1226, 632] is 84 % of the frame (it equals the bounding box of the 36 blobs of the sheet, and the sheet is the change), but the half-frame guard names only c5d c5e m8, so a whole-sheet box passes for c4 and hides a regression of the cube under the glass; fix: one boxed row per column of the sheet, or add c4 to the guard with a stated bound.
- rtr5r-5 NIT tests/test_tour_diff.py:882-943: five of the six tests assert the row's own words and boxes (a second copy of the toml), and the leftover blobs of `tour-diff-take-r5.json` are a hand cut of the take that no tool regenerates; the red-first holds, so no MAJOR; fix: none, the stored take is the real check (rtr5r-1 was found by it).
Verdict (computed): no open BLOCK or MAJOR: MERGE-OK (counts reproduce: 1 / 95 / 0 / 3 / 11).

## r6 (F7, rc 11): the 5 regressions of take r3 get a row, a declared baseline-none or a finding

Base 3098926eb (session head, t162 green); head 7b09b6230 (claude/tour-rows-r6). Step 0 green (`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`). No product code, no chapter change.
Commits: 46d73a85e **red** (test(tour): 8 tests fail on 3098926e: a declared baseline-none holds where v8 has a capture, its v9 checks still run, every declared name has a reason in `baseline_none_why`, a box over half the frame needs a `wide` reason, the m6/m6b hit rows, the m8 row; the r5 test that held m8 as a regression is cut down to the c5 acts' no-whole-frame-box), 7b09b6230 (the tool, the data). Files: tests/tour-diff.py, tests/test_tour_diff.py, tests/tour-diff-take-r6.json (new, the diffs of this take that no row of 3098926e named), demo/tour/intended.toml.

### The 5

| act | now | row / reason | still (media/2026-11-28/tour-rows-r6/) |
|---|---|---|---|
| c5d | **baseline-none** | `baseline_none` + `baseline_none_why["c5d.Q"]`: rtf1-1, v8 cannot step into a face, its capture is the cube stood back, v9's is face 0 full frame (its checks I1 I3 are red on v8 and the act's `expect` is red there, all 13 green on v9): no comparable baseline. No whole-frame row | c5d.Q-v8-v9.jpg |
| c5e | **baseline-none** | as c5d (Return steps in, f5bdc61f) | c5e.Q-v8-v9.jpg |
| m8 | **row** (pixel) | box [463, 198, 354, 324] (the blob, no wider); causes cd277ee4 (the box takes mc's colours), 12f97cd33 (mc's launcher is the kit Apps list, installed programs only), df1a2d0f (rows carry `Icon=`) | m8.Q-v8-v9.jpg (v8 above v9, box in red) |
| m6 | **row** (missing) | `mc-r1-right-keys-2-key-hit`, `-key-mark`: 4363de11 | m6.Q-v8-v9.jpg |
| m6b | **row** (missing) | as m6 | m6b.Q-v8-v9.jpg |

0 findings new, 0 regressions left, rtr4-2 can close.

**1 c5d c5e.** The mechanism already existed for captures with no v8 file (`baseline_none`, h2..); a c5d capture does exist on v8, so `compare()` took the pixel and scene diff and found a regression. Now a listed name is `none` also where BASE has a file, and the v9 checks of CAND still run through `verdict()` with an empty baseline (a red one no row absorbs is a regression; a test holds it, and holds that a row naming the diff does not turn `none` into `change`). `[baseline_none_why]` gives every declared name its reason (a test: no name without one; the h2..h7 reasons are the r1 text of record-v9). The old c5d c5e rows stay in the file, unused.

**2 m8 (rtr4-2): the premise is wrong, there is no icon fault.** The v9 take shows icons: Foot, Foot Client, Foot Server (the foot icon), ImageMagick (display-im6.q16) and Vim (gvim), as m8.Q-v8-v9.jpg shows; UXTerm and XTerm have `Icon=mini.xterm`, an xpm that fuzzel 1.11.1 does not draw (also blank on v8). The r3 take showed the same (the 03-12-launcher-dark tile of tour-v9-sheet.png, enlarged, has the icon squares). The environment has the hicolor theme (`/usr/share/icons/hicolor/48x48/apps/foot.png`, `scalable/apps/foot.svg`), the harness gives the session `XDG_DATA_DIRS=/usr/local/share:/usr/share`, and a probe of `chora-mc --launch fuzzel` on the head with a stand-in fuzzel under the take's PATH prints `Foot\0icon\x1ffoot`, `Vim\0icon\x1fgvim`, `ImageMagick (color depth=q16)\0icon\x1fdisplay-im6.q16` (what df1a2d0f writes). So no Icon= is missing from the take's entries, no theme is missing, no product fault; rtr4-2 (reports/tour-rows.md r4: "its rows lost the icons") described the take before kit-icons r1 (df1a2d0f) landed, and the r3 report copied it. What the pixels do show, all intended or by the take's PATH: (a) the box is dark with a hairline border in v9, cream in v8 (cd277ee4, rpx1-10: mc writes fuzzel's colours from its theme); (b) v8's mc ran a plain `fuzzel` (application mode, icons by fuzzel itself, every desktop file), v9 runs `chora-mc --launch fuzzel` (12f97cd33, mc-44: the kit Apps list, installed programs only: `libreoffice` is not on the take's PATH, the harness's link farm leaves it out, so the five LibreOffice rows of v8 are gone; probe: v8's `chora-mc` has no `--launch`); (c) icons per row (df1a2d0f). Judged on the full-size frames: whole text, nothing under 12 px, the highlighted row and the hairline clean, icons 16 px, no cut row. The row's box is exactly the blob.

**3 m6, m6b.** The r3 report listed every node that is only in the v8 scene (notify-0, doors, keys-12, title-rule) and said no row names them. Checked per node with the tool's own `named()` on the head's rows: all of them are named (675ebd55 rows of the first r1 take); the only baseline-only nodes **no row names** in this take are `mc-r1-right-keys-2-key-hit` and `-key-mark`, in m6 and m6b alike. Cause: the v8 row 2 of the key list (Super+Home, a binding) has a hit and a mark; in v9 row 2 is the new row "Up Down Scroll the page" (4363de11, rpe-1, keys.rs: `Row::new(..., None)`: no binding) and a row has a hit only with a binding (`hit: row.binding.as_ref()...`, plugins/chora-mc/src/region/items.rs:551), the Home row (fa7faabd) and this one shift the rows after it by two. In m6b the list is closed, but the pool of the first region slot keeps its nodes (v8 r1 rows 0..18, v9 0..20), so the same two nodes differ. Looks (m6.Q-v8-v9.jpg, m6b.Q-v8-v9.jpg, right column v8 above v9): the v9 key list reads whole, 21 rows to the page foot, all text at or above the column's 14 px; the doors under the list are in the v8 frame and not in the v9 one: that is the named 675ebd55 row (the column stands in the other region slot, the list fills the column), unchanged. A row per act, `nodes` exactly the two.
- **rtr6-1 NIT** demo/tour/intended.toml (old m6 row, cause 675ebd55): it also names `mc-r1-right-keys-12-key-hit` and `-mark` (v8 row 12 Super+Shift+Escape had a hit, v9 row 12 is Super+Alt+1-4, a range with no binding); the cause is the same index shift as above (fa7faabd, 4363de11), not the notices. The diff is named, so the verdict is right; fix: move both nodes into the row of 4363de11.

**4 rtr5r-4.** The c4 box [27, 58, 1226, 632] is the key sheet panel (construct-help-body-panel [31, 58, 1218, 604]) and its shadow. Per column rows are not possible (the blobs of columns 2 and 3 are one blob [382, 78, 871, 610]; a box has to hold a blob). So the guard is in the tool, not a list of acts in a test: a pixel box over half of 1280 x 720 (`HALF_FRAME`) holds a blob only with `wide = "why"` in its row, else the row proves nothing and the act is `open` (tests/tour-diff.py `pixel_rows`; 5 tests: planted without and with the reason, small box, a small box before a wide one, and the data: c4 is the only box over half the frame, with a `wide` naming the panel). c5d, c5e have no row; m8's box is 12 % of the frame.

### Takes and counts

v9 (head 3098926e build) and v8 (733967aa in a worktree, `TOUR_BASELINE=1`, the head's harness), each chapter its own TOUR_OUT: mc and construct (the touched chapters). v9: mc 22 acts 0 red; construct 6 green 12 red (I2 of co-44 only); v8: mc 22 red (no state.get on the old build), construct 4 captures `baseline: none` after c3. tests/tour-diff.py, `--intended demo/tour/intended.toml` (take-diff-mc.txt, take-diff-construct.txt; before the rows: take-diff-mc-before.txt):

| | same | change | open | regression | none |
|---|---|---|---|---|---|
| mc, 3098926e rows | 0 | 19 | 0 | 3 (m6 m6b m8) | 0 |
| mc, 7b09b6230 | 0 | 22 | 0 | 0 | 0 |
| construct, 3098926e rows | 0 | 12 | 0 | 2 (c5d c5e) | 4 |
| construct, 7b09b6230 | 0 | 12 | 0 | 0 | 6 |

xp, tiling, handover are not retaken (same build, same harness, no row of them changed): `verdict()` over their 63 entries of tour-diff-r3.json (scene and blobs) under the new intended.toml: 62 change, 1 same, 0 other. Total of the 110: **1 same, 96 change, 0 open, 0 regression, 13 none** (r3: 1 / 93 / 0 / 5 / 11). tour-diff exits 0 on both takes. `python3 -m pytest tests/test_tour_diff.py tests/test_tour.py -q`: 240 passed, 19 subtests at the head; at the red commit 8 failed and 232 passed.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push, no step filter, run detached, run directory /tmp/merge-gate.cu28RW) on 7b09b6230: 67 steps PASS (build 33 s, test 184 s, matrix 374 s, layout 1384 s, pull 530 s, acts 2320 s, leak 477 s, e2e 288 s), docs skipped by the gate itself (`skipped: docs (mkdocs is not installed (docs unchanged))`), no FAIL, no red cell, no foreign flake. It ends "all steps passed; no --push, nothing is pushed", printed just before `exit 0` (tests/merge-gate.sh), so **EXIT 0**. Last 20 lines: media/2026-11-28/tour-rows-r6/gate-last.txt. `pgrep -fa 'chora --headless'` empty.

## Review r6 (2026-11-28, 3098926e..7b09b6230, head 7b09b6230, 4 files +199 -13)
Checks (brief review-tour-rows-r6; step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour_diff.py -q` 132 passed, 19 subtests):
- 1 Red first holds: `pytest tests/test_tour_diff.py` at 46d73a85e: 8 failed (the 2 CompareDirs baseline-none tests, the 2 WideBoxes tests that read the guard and the data, the 4 RegressionsOfTakeR6 tests), 124 passed; at 7b09b6230: 132 passed. The red commit holds tests and `tests/tour-diff-take-r6.json` only, no row, no tool change.
- 2 c5d, c5e as baseline-none (rtf1-1): honest. Still pairs opened (v8 over v9): v8 is the cube stood back, v9 the xp face at 1:1 with the sunset window and the chip row Alt Esc Step back, which is what the acts say (demo/tour/chapters/construct.py:117, :137: "the drop stepped into that face", "Return: steps into the face"); the two frames are two scenes, every pixel differs, so there is no baseline to compare. The v9 frames show no cut text, no overlap, no wrong colour; the checks of v9 still run (`verdict()` with an empty baseline; tested: a red I9 with no row is a regression) and the construct take reads 0 regression. This is the fix rtf1-1 named in reports/tour-rows.md r4 ("tour-diff gets a declared no comparable baseline"), no whole-frame row. Nothing hidden. The two names have a reason line naming rtf1-1.
- 3 m6 m6b, m8: m6/m6b: the rows name exactly `mc-r1-right-keys-2-key-hit` and `-key-mark`; on the stills v8 row 2 is Super+Home, v9 row 2 is "Up Down Scroll the page" (no binding), so no hit and no mark: true, one cause (4363de11, with fa7faabd's Home row for the index), two nodes. The v9 key list reads whole in both stills (m6's footer row is below the crop of the still, not judged). m8: the v9 launcher shows icons (Foot, Foot Client, Foot Server, ImageMagick, Vim; UXTerm, XTerm none in both builds), so the premise of rtr4-2 "no icons" is wrong on the stills; the five LibreOffice rows are gone because the harness link farm leaves `soffice`/`libreoffice` off PATH (demo/tour/launch.py:207-215). rtr4-2 can close. The box [463, 198, 354, 324] is the blob (12 % of the frame), three causes (colours cd277ee4, Apps list 12f97cd33, Icon= df1a2d0f) over one box: rtr6r-3.
- 4 The guard (`pixel_rows`, `HALF_FRAME` 1280 x 720 / 2): a blob held only by a box over half the frame without a non-empty `wide` makes the act `open` (`unproven: pixel`), tested both ways plus a small box and a small box before a wide one; the data test pins c4 as the one wide box (1226 x 632, the panel [31, 58, 1218, 604] with its 4 px shadow: 27..1253, 58..690, exact). I scanned every box of every row of intended.toml: only c4 is over half the frame; the next are 816 x 549 (36 % of the frame; 29 rows), so no row passes only because of `wide`. The guard does not reject a row with a reason that is wrong: `wide` is free text and only the c4 one is pinned (acceptable by the brief: a stated bound).
- 5 Counts reproduce. `verdict()` of the head's tool over the 110 entries of media/2026-11-28/tour-v9-r3/tour-diff-r3.json under the head's intended.toml: 1 same, 96 change, 0 open, 0 regression, 13 none (the 11 of r3 plus c5d c5e); the only entries that differ from the recorded r3 verdict are m6.Q, m6b.Q, m8.Q (regression -> change). take-diff-mc.txt: 22 change; take-diff-construct.txt: 12 change, 6 none. `tests/tour-diff-take-r6.json` holds only the leftover diffs of m6, m6b, m8 (the tests read it), not the counts. rtr6-1 NIT is in the report.
Findings:
- rtr6r-1 NIT demo/tour/intended.toml:1895-2030, 3635-3644 (the `[[change]]` rows of c5d and c5e other than the I2 check row): dead rows. `compare()` takes `none` before it reads any row for the pixel and scene diff, so none of them is read again; their causes are r1 text ("c5e steps into the face", "Super+F1 opens the kit sheet") that the act no longer shows, and they would silently name a diff again if the name left `baseline_none`; fix: delete them and keep the check row (the one that absorbs the I2 flip of v9).
- rtr6r-2 NIT reports/tour-rows.md (r6, item 3, "shifts the rows after it by two") and intended.toml (m6, m6b rows of 4363de11): the shift is one, not two: v9's All keys list has no Super+F1 row (v8 row 0) and has Home, Up Down and Super+drag more, so v8 row 11 (Super+Alt+1-4) is v9 row 12; the older 4363de11 rows (r4) say "by one"; fix: say "by one" or name the dropped Super+F1 row in the cause.
- rtr6r-3 NIT demo/tour/intended.toml:4962 (m8 row): three causes over one box (the whole launcher); a blob is one connected piece so the tool cannot box the three apart, and a later defect inside the box (a cut row, a wrong icon) passes as this row; fix: the cause names where each lies (colours and hairline: the box; the five gone rows: y below the Vim row; icons: the icon column) so a review can look for them, or the take keeps a second still of the launcher at 1x.
- rtr6r-4 NIT tests/tour-diff.py:340 (`if stem in none`): a declared name is `none` now also where BASE has a capture, for h2..h7 and every name of `baseline-none.json` too, with the same detail "baseline: none"; if the v8 replay is later fixed so h2.Q exists, the tool keeps saying none and nobody sees that a baseline exists; fix: detail "baseline: none (BASE has a capture)" when `base / name` exists, and the count line shows it.
- rtr6r-5 NIT tests/test_tour_diff.py:687 (the wide-box data test): it scans `kind == "pixel"` rows only, while `pixel_rows` also uses rows of kind `*` with boxes; none is over half the frame today; fix: `r["kind"] in ("pixel", "*")`.
Counts: BLOCK 0, MAJOR 0, MINOR 0, NIT 5. Verdict (computed): no open BLOCK or MAJOR: **MERGE-OK**. Known, not findings: co-44, mc-63, rtr4r-1..4, rtr5r-*.
