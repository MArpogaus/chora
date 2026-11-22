# tour-fix report

## r1 (F7, rc 11: unblock RECORD of tour v9; 2026-11-21)

Branch `claude/tour-fix` from 892d035c, merged with origin/claude/autonomous-dev-session-3w-i33pg7 (e87b0ff9) at d76fbaac. Head **faf958fe** (pushed to claude/tour-fix). Step 0 green (unshallow, cargo check, cargo build). No product code touched: demo/tour, tests/test_tour.py, tests/tour-diff.py, tests/acts, tests/tiling-audit.py.

### Commits (red, then green)
| # | red | green | what |
|---|---|---|---|
| 1 | 0b14978a | 82afbbc4 | the tour writes `<id>.<capture>.png` and `.json` per capture in `<out>/captures` |
| 2 | 40fdb206 | 21777e46 | the harness degrades on a build without `scene.boxes` / `notify state.get` |
| 2b | a222e45b | db187a87 | choractl words a missing method "is no exported method": the regex missed it (found by the first replay) |
| 3a | 4b27cf2f | 65560447 | I6: xp text floor 11 px (replica); the tour judge keeps the design exemption of known-red.toml |
| 3b | a4e57870 | b1317554 | the harness reads boxes before and after the captures |
| 3c | b86e3ffb | 98cee862 | I2: the inner gap of tiling is 10 (design/looks/tiling-v2.md), not a unit of 4 |
| 3d | 0d26d8f6 | e21d21f4 | the mc chapter starts chora-notify with plugins/chora-mc/notify |
| 3e | d1a26eb6 | 8222044d | I6: a notify card is xp-sized only in a session of xp alone (the gate found it: the xp on a face of the cube holds the focus, so the card of a construct session read as xp and its known-red row went stale) |
| 3f | - | faf958fe | the acts gate: one known-red row (construct I6 `construct-mode-step-in-text: the contrast`, act "the cube is held halfway through a turn") that the 3b read no longer meets; it was the same capture race |

### 1) Dumps
`Tour.dump` (tour.py) runs at the quiet read of each act (`check_armed`): `<id>.<capture>.json` = `{nodes: scene.query "*", masks: text boxes of scene.boxes (the followed output), checks: every invariant and `expect` green/red}`, `<id>.<capture>.png` = `choractl capture` of the output at that read (the scene, no pointer). H50: the png is the recorded frame in the middle of the act (`run_segment`, record mode), the pointer is a mask. `--check` writes nothing. The id is the look letter and the act number of its chapter: `x3`, `t10`, `m4`, `md4` (the dark sofa chapter numbers again), `c5`, `h2`. `tests/tour-diff.py` now splits the stem at the first dot (a capture `Q+0.5` has a dot). Real proof: the replay below wrote 62 files (31 png + 31 json) that tour-diff reads (`x2.Q.png` 1280x720, `x6.H50.png`).
Limits (honest): Q+t is the read at the end of the act's own settle and tail, not t seconds later (the scene clock stands still between segments); without a checker (cannot attach) nothing is written.

### 2) Old build
- `OldCore` wraps the harness connection: `scene.boxes` answers None on "no method" (said once), anything else raises. Snapshots then have no boxes, no masks; "missing boxes" is not a red line there.
- `Session.dismiss_notices`: on "no exported method" for `state.get` it closes ids 1..24 with `busctl --user ... CloseNotification` (the old path), said once.
- `run.py` with `TOUR_BASELINE=1`: a chapter that stops (an act the old build cannot do) is `baseline: none from the act that stopped it: <reason>`, no BROKE, listed in `baseline-none.json`.
- Replay of chapter xp on 733967aa (worktree, `TOUR_REPO=/home/user/base733 TOUR_BASELINE=1 python3 demo/tour/run.py xp`, built there with cargo build --workspace, target/ deleted after, worktree removed):
```
chapter xp: 31 acts, 0 green, 0 inv, 0 known, 31 red, 0 todo, 0 unchecked
tour: record: 31 red, 0 broken
```
31 acts, no BROKE. The 31 red are the checks of the candidate on the old build (xp answers no `state.get`: I1, I3, the `expect`s); tour-diff reads "check red on BASE" as no regression. Log kept in the container only.

### 3) Triage
Before (record-v9 r1 on 892d035c): 90 red acts, xp 30 (+1 known), tiling 23, mc 8, construct 18, handover 11; I6 1793 lines, I2 90, I10 2. Re-measured here: xp 30 red, 409 I6 lines; construct 18 red, 1067 I6 + 28 I2 lines.
After (`python3 demo/tour/run.py --check` on e21d21f4, one run): **66 red, 0 broken**.

| look | acts red before | after | I6 lines after | I2 | I10 | expect |
|---|---|---|---|---|---|---|
| xp | 30 | 30 | 97 | 0 | 0 | 0 |
| tiling | 23 | 5 | 108 | 0 | 0 | 2 |
| mc | 8 | 2 | 4 | 0 | 2 | 0 |
| construct | 18 | 18 | 440 | 28 | 0 | 0 |
| handover | 11 | 11 | 34 | 0 | 0 | 0 |

Check faults fixed (each with a red test, commit above):
1. **I6 14 px for xp** (3a): design/product-briefs.md:49, 8 pt Tahoma = 11 px is the replica and replaces the floor for xp. `least_px(owner, shown)`: 11 for a text node of xp and for a notify card on an output where xp is shown; 14 for the rest. 73 known-red rows of that floor (xp and the xp faces in construct) left tests/acts/known-red.toml, two tests that named them went.
2. **I6 tray contrast** (3a): the one `[[exempt]]` row of known-red.toml (Luna tray, floor 3.0) is a design exemption; the tour judge now applies it (known-red rows are not applied: the tour must show real breaches). The tray at 3.21:1 passes, 2.90:1 would not.
3. **I6 1.00:1 on the mc right column (33 lines in the key sheet acts)** (3b): a race of the check, not mc: the capture was taken before the Super+F1 sheet finished sliding in and the boxes after (node y 28 vs box y 56). `settled_scene` reads the boxes before and after the captures and retries until equal.
4. **I2 tiling gaps "10 apart, no whole number of 4"** (3c): design/looks/tiling-v2.md line 17: grid 4 px, outer gap 12, inner gap 10; gate-tiling-gaps.sh asserts 10. The tb- audit accepts a gap of the inner gap (a 9 or 11 stays red).
5. **I6 11 px card in mc** (3d): the tour started chora-notify plain; the mc session starts it with the assets of mc (card 19 px, bottom left; gate-mc-sofa.sh). tiling and construct do start the plain one: that stays red (sh-31).
6. **I2 construct "9 apart" (design/tour-v9.md:162): the check is right**, not a fault. It measures the chip rects ([289,652,187,32], [485,652,129,32], [623,652,197,32]): the gap is 9. The rule is whole units (construct 8). It stays red, row co-44; the same breach is a known-red row of the acts gate. Nothing marked known.

Findings rows (findings/ on claude/session-log, 08644900): xp-39 (pager contrast 2.57/3.77/3.63), xp-40 (Luna blue pairs: taskbar button 3.86, session button 3.56 against 7:1, Start menu user 4.32, icon label 1.67), xp-41 (inactive title 2.16), mc-55 (`AGENTS` I10), mc-56 (doors under the open chat 1.00:1, labels covered by hud-chat), ti-30 (key list 13 px, key line 6.50:1), co-44 (chips 9 apart, COSMETIC), co-45 (construct help 11 px), sh-31 (notify card 11 px in tiling and construct). All MINOR (COSMETIC co-44): they break the guide checks, not a main path; rc.md criterion 1 counts BLOCK and MAJOR.

Red cells I did not cause or could not fix (not rerun to green):
- tiling act t26 "Scrolls on the volume word of the bar": `the mixer is at 40 percent, expected 45` and t28 "Clicks the row Screenshot": `no new notice in chora-notify`. Both aim at fixed pixels of waybar (1018,24) and of the power menu (`TOUR_SHOT_X/Y` 640,476); waybar is an outside program with no scene nodes, so no selector can replace the pixel. In this container the aim misses (the same two were red in the 90). A row aim fault, not a product breach; needs the aim read from the capture or waybar's layout.
- the one `known` row of the xp chapter (ca-1, the key list of Help at 1280x720) is unchanged.
- one hand run of the acts cells (outside the gate, to list stale rows) showed once `tiling I14: in act 'Escape shuts the key list of the keys key': chat-door disconnected, the act did not declare it`. Not seen in the gate runs (acts PASS 1470 s on faf958fe), not caused by these commits (no product code); named here, not chased.

### Gate
`tests/merge-gate.sh --jobs 1` (no --only) on faf958fe, which is claude/tour-fix merged with origin/claude/autonomous-dev-session-3w-i33pg7 e87b0ff9 (d76fbaac): **all steps passed** (run directory /tmp/merge-gate.f8NgTh, container only): red-first, identity, pre-commit, gate-lib, ci-check, build, test, clippy, 2d-guard, ..., acts 1470 s, ..., agent, e2e 300 s. An earlier full run on 98cee862+merge failed at `acts` with 1 blocking line (the stale card row, 3e); `--only acts` then found the second stale row (3f); both fixed, then the full run above.
Commits carry no attribution lines (hook: no Co-Authored-By). Author and committer Marcel Arpogaus.

### Counts after, on the head
`python3 demo/tour/run.py --check` on faf958fe: **66 red, 0 broken** (before: 90). Per look, red acts and lines per invariant:

| look | red acts | I6 lines | I2 lines | I10 lines | expect lines |
|---|---|---|---|---|---|
| xp | 30 | 97 | 0 | 0 | 0 |
| tiling | 5 | 108 | 0 | 0 | 2 |
| mc | 2 | 4 | 0 | 2 | 0 |
| construct | 18 | 440 | 28 | 0 | 0 |
| handover | 11 | 34 | 0 | 0 | 0 |

The table of 3) above was measured on e21d21f4; the run on the head gives the same acts (30, 5, 2, 18, 11); the line counts are the ones here.

## Review r1 (review-tour-fix, F7 rc 11; 2026-11-22)

Seeded review of claude/tour-fix faf958fe against base 892d035c (cargo check --workspace green; `python3 -m pytest tests/test_acts.py tests/test_tour.py -q`: 271 passed, 123 subtests). No merge gate run. Product code untouched (confirmed: the lane diff is demo/tour, tests/acts, tests/*.py, known-red.toml, plus the merged autonomous-dev commits).

**Verdict: FIX** (3 MAJOR open, 4 MINOR, 1 NIT).

### Checks
1. Dumps: red 0b14978a fails (`CaptureDumps`), green 82afbbc4; tour-diff splits at the first dot and reads `<id>.<capture>.png/.json` (layout, keys `nodes`, `masks`, `checks` match tests/tour-diff.py:197-211). Two defects below (rw-1, rw-2).
2. Old build: red 40fdb206 fails, green 21777e46. The degrade path is in the code: `OldCore` answers None for `scene.boxes` only, `dismiss_notices` falls back to `busctl CloseNotification`, `run.py` with TOUR_BASELINE prints `baseline: none ... <reason>` and writes baseline-none.json. Not a silent pass on the baseline; but see rw-3 (the same degrade is active on the candidate) and rw-4.
3. Check faults, each red commit run on its parent: all eight red commits fail on their own test (0b14978a, 40fdb206, a222e45b, 4b27cf2f, a4e57870, b86e3ffb, 0d26d8f6, d1a26eb6). Regex a222e45b/db187a87 real (choractl wording "is no exported method"). xp 11 px floor 4b27cf2f/65560447: design/product-briefs.md:49 says 8 pt Tahoma = 11 px "replaces the 14 px floor (D1) for xp"; it covers size only. The tray exemption is label-scoped (`xp-tray-`, `exempted()` ignores the row's `look`), floor 3.0, its only source is the toml's own reason (rw-8). Boxes a4e57870/b1317554 real (capture and boxes were two moments; see rw-7 for the last-try case). Tiling gap b86e3ffb/98cee862: design/looks/tiling-v2.md:17 names inner gap 10 and gate-tiling-gaps.sh asserts it, but the same page says "everything on a 4 px unit": see rw-5. mc notify 0d26d8f6/e21d21f4 real (gate-mc-sofa.sh starts chora-notify with the mc assets), the test greps the source text (rw-8). Card xp-sized d1a26eb6/8222044d real (owner "notify" and looks == {"xp"} only).
4. known-red.toml: 74 rows removed (73 + the stale one of faf958fe). I classified all 74 by label: 73 are `guide wants 14 px` on xp-owned nodes (xp-pager 24, xp-item 17, xp-icon 9, xp-ctx 6, xp-session-button 6, xp-tray 6, xp-task 3, xp-tooltip 1, plus the notify card in xp and the xp faces in construct); the rows are gone because the floor for xp is 11, not because a match widened (the new floor still fires below 11, and the gate acts step was green with no row left, so no xp text under 11 px exists in the acts). The 74th (construct-mode-step-in-text, the contrast, held halfway through a turn) rests on one gate run and the race fix; its sibling row `construct-hints-construct-mode-step-in-keys-0-text` in the same act stays, so the race alone may not be the whole story (not a finding: the gate calls a row nobody met stale). No removed row hid a real breach.
5. Remaining reds: xp-39..41, mc-55, mc-56, ti-30, co-44, co-45, sh-31 all exist in findings/ (checked the text of each). t26 and t28 are not caused by the lane (demo/tour/chapters/tiling.py is untouched; both acts aim fixed pixels, tiling.py:230,233; the lane diff of the harness is the judge and the dump only), but they have neither a fix nor a findings row (rw-6). The remaining known-red rows name free text ("guide-checks r2: ..."), not a findings id; that is old.
6. pre-commit on the changed files: all hooks that apply pass; the ruff hooks are scoped to `^services/voice/` (.pre-commit-config.yaml:98), so they were skipped, not passed (ruff run by hand shows old noise only). Largest file: tests/test_acts.py 1810 lines (1801 at the base, +9 by the lane); no rule in rules.md names 1,500 for tests; demo/tour/tour.py 716.

### Findings
- rw-1 | MAJOR | demo/tour/tour.py:569 | `dump` writes the png only `if not os.path.exists(png)` and run.py never clears `<out>/captures` (TOUR_OUT defaults to a fixed folder), so a second take in the same folder keeps the old png beside the new json; shown: a stale `x1.Q.png` survives an act that dumps, 0 captures taken; tour-diff then diffs the old pixels | fix: take the Q picture always and keep the exists-guard for H50 only (the frame copied by `copy_held`), or empty `captures` at the start of a take, and add a red test with a stale file.
- rw-2 | MAJOR | demo/tour/tour.py:68,451 (and demo/tour/intended.toml:10,15,20) | the act id is the number of the act in the chapter (handover: h2 = "Click Start", the Send To act is h5), but intended.toml and design/tour-v9.md name scenes (h2 = title-bar Send To; 11 xp scenes against 31 xp acts); the three `h2` rows (xp-give Send To) would name the Start-menu diff as intended and leave the real Send To act h5 an unnamed regression; the docstring says "as intended.toml names it", which is false | fix: key the dump by the scene id of design/tour-v9.md (an `id=` on the act, or a table per chapter) or re-key intended.toml to act numbers, and test one id of each chapter against intended.toml.
- rw-3 | MAJOR | demo/tour/tour.py:226,246-247 | `OldCore` and the removal of "missing boxes" are active in every run, not only the replay: on the candidate a build that lost `scene.boxes` (or a chora with the call broken) prints one log line and the box-based invariants (I5, I6, I8, I9) go green on nothing; rules of the brief: never a silent pass | fix: apply both only when TOUR_BASELINE is set (pass the flag to `Checker.attach`), keep "missing boxes" red on the candidate, and test the candidate case.
- rw-4 | MINOR | demo/tour/run.py:82-95 | with TOUR_BASELINE every exception (a TypeError of the harness, a dead core) becomes `baseline: none` with exit 0, and baseline-none.json holds chapter and reason, not the `<act>.<capture>` names that tour-diff's `baseline_none` takes (tests/tour-diff.py:203), so the list is hand-made | fix: convert only a RuntimeError that NO_METHOD matches (the rest stays BROKE) and write the acts and captures not run into baseline-none.json.
- rw-5 | MINOR | tests/tiling-audit.py:89-110, findings/tiling.md ti-31 | the check now accepts a gap of 10 and ti-31 (the persona's "10 px, not a multiple of 4", open) is never reported again, while design/looks/tiling-v2.md says both "4 px unit" and "inner gap 10"; the lane decided a design question and left ti-31 open and unmentioned in its report | fix: record the owner's decision in tiling-v2.md (the inner gap is the one off-grid value) and close ti-31 as a check fault with the commit, or open the design question to the owner and keep the row.
- rw-6 | MINOR | demo/tour/chapters/tiling.py:230,233 | t26 (volume word 40, expected 45) and t28 (no new notice) stay red with neither a fix nor a findings row; the done criterion asks one of the two for each of the 90 reds, and the RECORD stays red there | fix: file the two as rows (aim, not product, with the capture shown) or read the aim from the capture / waybar layout.
- rw-7 | MINOR | tests/acts/harness.py:708 | after 5 tries `settled_scene` returns the last boxes read with the earlier pixels and nodes without a sign, so the race the lane fixed still passes silently when the sheet slides for the whole retry window | fix: on the last try return the snapshot with a red line "the scene did not settle" (or one more sleep) and test it.
- rw-8 | NIT | tests/test_tour.py:694, tests/acts/known-red.toml:15, .pre-commit-config.yaml:98 | (a) the mc notify test asserts a string of mc.py, not the start of chora-notify; (b) the tray exemption (floor 3.0) rests on its own reason "authentic Luna tray, vision card", no design page says it (product-briefs.md:49 is the size only) and the tour now applies it; (c) ruff is not run on demo/ and tests/ by pre-commit | fix: (a) check the argv in `launch` with a fake `L.start`; (b) cite the owner decision in design/looks/xp.md; (c) leave or widen `files:`.

Counts: BLOCK 0, MAJOR 3 (rw-1, rw-2, rw-3), MINOR 4, NIT 1.
