# p2-harness-m

## r1

Branch claude/p2-harness-m, head 4d738ce8 (pushed), on 1c9e64f7; origin/claude/autonomous-dev-session-3w-i33pg7 is already an ancestor, nothing to merge. Step 0: unshallow and `cargo check --workspace` green.

Red then fix:

| # | red | fix |
|---|---|---|
| 1 known-red | d117c9f9 | 16c24276 |
| 3 T7 no event | 0a63c5fb | 3fe48aee |
| 2 L2 | d8bf577b | 4727946f |

Then 4d738ce8 (test): tests/test_acts.py PersonaContracts read the new L2 digest (the first gate run was red on them: `acts` cell, test_l2_the_look_holds_the_focus and test_end_reads_the_begin_of_the_goal_and_never_start; my change).

- Fix 1: `Run.breaches()` (check.py) hides a breach only when a row of its (look, id) names it by `row_covers` (capture, match; `act` ignored, persona actions are no acts). Rows whose match changed: all 189 rows of known-red.toml have a capture or match (0 pair-wide rows), so all 189 narrow from hiding the pair to hiding what they name. Tests: KnownRedNamesBreaches (3).
- Fix 2: L2 = Expect(focus, verify_reloaded): the config digest at `begin` (kept by `Run.save` as check/begin.config) must differ from the digest at `end`, and the look must hold the focus after the reload. Gap: the state answer has no reload counter, so "reload took" is the focus/answer of the look, not a counter. Tests: NoConfig (2 new, 1 changed).
- Fix 3: `NEEDS_EVENT = {T7, C2, L3}`: with no observation since `begin` the line is `task T7 not reached (no event since begin)`, not counted red, never pass. Other contracts with this vacuous shape: C2 and L3 (done). The rest need a positive effect (new window, call, notice, page, rect, closed window, config) and fail by themselves without an event; X1, C3, M2, C1 read state only and were left (a look that holds focus at begin and end is a valid state). Tests: NoEvent (3), existing T7/L3 tests now pass an event.
- README updated.

Gate: `python3 -m pytest tests/persona -q` 48 passed. `tests/merge-gate.sh --jobs 1` (second run, after 4d738ce8): 22 PASS, 1 FAIL, not rerun: cell `tiling-look` (log /tmp/merge-gate.m3rVLt/tiling-look.log): `FAIL: ink-1280x800 bar-font: the clock is 120.0 wide, not Inter 14 px weight 500 (100-119)` (same for ink-2560x1600@2, paper-1280x800, paper-2560x1600@2). Not mine: a pixel measure of the tiling bar clock, no persona code involved. The gate stopped there, so the cells after it did not run; the first run (before 4d738ce8) passed 26 cells up to `acts`.
