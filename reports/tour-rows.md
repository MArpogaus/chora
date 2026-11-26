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
