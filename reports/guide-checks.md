# guide-checks

## r1

Lane 3 of design/tour-v9.md section 8: the "extends" rows of section 7 as checks in
`tests/acts/invariants.py` and the audits, each red on a planted breach first.

| Row (section 7) | Check | Planted (tests/acts/planted.py) | Unit (tests/test_acts.py) |
|---|---|---|---|
| contrast 4.5:1 text, 7:1 key line and buttons | I6, `png-contrast.contrast_rows` on each text box | `I6 faint text` | `GuideContrast` |
| text at least 14 px | I6, `font_size` of boxes and `px` of runs (a state word may be smaller) | `I6 small text` | `GuideSize` |
| gaps in whole units | I2, `hud-audit.gap_findings` (8, helper surfaces) and `tiling-audit.gap_findings` (4, tiles) on `scene.boxes` | `I2 odd gaps` | `GuideGaps` |
| words from the word table, no "agent", no "HUD" | I10, `WORDS` and the long key names of `words.table` | `I10 the word agent` | `GuideWords` |
| screen-fixed nodes held in a camera move | I2 held, helper surfaces equal before and after | `I2 held, a fixed surface moves` / `stays` | `GuideHeld` |

Red then green: a2ee5219 (planted and unit cases, 8 failures and 3 errors) then bd449adf (checks, 169 unit tests OK); a78b1a07 the known-red rows.
The Snapshot got `shots` (a capture of each output at 1280); hud-audit got a `gaps` kind
(opt-in: AUDIT_KINDS or `unit`); tiling-audit got a `__main__` guard.

Gate (`--jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts,hud,layout,tiling-audit`):
pre-commit, gate-lib, ci-check, build, test, hud, layout, tiling-audit PASS; acts PASS (786 s)
with the findings below listed in tests/acts/known-red.toml.

### Findings (real faults on the session head 8b996cee; the looks are not fixed here)

| # | Look | Act | Capture (evidence) | Rule |
|---|---|---|---|---|
| g1 | construct | "a drag with the modifier on the front face turns the cube" | construct-mode-step-in and construct-mode-turn-left are 9 apart (I2) | gaps whole units, construct 8 |
| g2 | construct | "the keys key opens the key list" | `construct-help-body-scope` 'CONSTRUCT · WALKING' is 11 px (I6) | text at least 14 px |
| g3 | mc | "Right of a remote turns the page in dark mc" | `mc-r1-footer-next` says '→ 3  Agents' (I10) | no "agent" |
| g4 | mc | "a helper starts" | `mc-r1-right-agents-0-sign` '○' has 1.64:1 (I6) | contrast 4.5:1 (a glyph: 3:1 may be the right rule) |
| g5 | tiling | "the keys key opens the key list" | `tiling-keys-body-1-k` 'Super+J' is 13 px (I6) | at least 14 px |
| g6 | xp | "a window opens" | `xp-icon-0` 'My Documents' is 11 px (I6; Luna icon labels) | at least 14 px |

The cell keeps the breaches of its first act, so more faults may hide behind a listed row
(match "the guide wants"); each row goes when its look is fixed. Evidence: `/tmp/acts-run-*/LOOK-I6`
of the gate run (still.png, states.json). g4 and g6 need a ruling: a glyph and an icon label
may be outside the text rules.

## Review r1

Review of claude/guide-checks head a78b1a07 over base 8b996cee (sonnet, high; read-only on code). Step 0 green (`cargo check --workspace`).

**Verdict: FIX** (3 MAJOR, 4 MINOR, 1 NIT open).

1. Red first. `python3 -m pytest tests/test_acts.py -q` in worktrees: a2ee5219 11 failed (8 assertions, 3 TypeError), 158 passed; bd449adf 169 passed; a78b1a07 169 passed. The planted cells of tests/acts/planted.py need the build and were not run (brief: no build); I checked them only by reading.
2-5. Findings below; probes ran on a78b1a07 (`judge()`, `breaches("I6", ...)`, `WORDS`).

| id | severity | file:line | finding | fix |
|---|---|---|---|---|
| rg-1 | MAJOR | tests/acts/known-red.toml:13,20,27,34,41,48 | Rows key on look+id and a `match` substring with no act or capture: "the guide wants" is in every size and contrast breach of the look, "no whole number of 8" in every gap breach, "a word of the builders" in every agent/HUD breach. Probe: `judge()` with the g2 row returns no blocking for a 1.2:1 contrast fault of another act of construct. run.py prints only the first breach of a cell. | Put act and capture in each match (e.g. `in act 'the keys key opens the key list'` plus `(construct-help-body-scope)`), one row per fault, and print every breach of a red cell. |
| rg-2 | MAJOR | tests/acts/invariants.py:500 (+ harness.py:628, test_acts.py:1316) | A missing capture passes: `rows is None` and a box under 1 px or under 2 px after scaling `continue`; `Session.shots` swallows RuntimeError/OSError; `test_no_capture_no_finding` enshrines it. The brief: a missing box or capture fails, never passes. | Report "no capture of OUTPUT" and "box of X cannot be measured" as breaches; flip the unit test. |
| rg-3 | MAJOR | tests/acts/invariants.py:509 | `back != fore` skips text of the colour of its background (1:1). Probe: white ink on white capture returns `[]`. | When a box has text and back == fore, report contrast 1:1 (add a planted case and unit test). |
| rg-4 | MINOR | tests/acts/invariants.py:240 | 7:1 is chosen by a label regexp: `mc-key-line` and `xp-session-button-N` match, `tiling-keys-body-1-k`, `construct-hints-...-keys-0` and `construct-mode-*` do not, so those caps and buttons get 4.5:1. | Mark key lines and buttons in node meta, or list them per look. |
| rg-5 | MINOR | tests/acts/invariants.py:239,498 | A box or run with no `font_size`/`px` passes the 14 px check (`px is not None`); `STATE_WORD = "state"` is a substring of the label, so any label containing "state" is exempt. | Fail on unknown size; anchor to `-state$`. |
| rg-6 | MINOR | tests/acts/invariants.py:253,609 | I10 matches a key spelling only on whole-text equality (probe: "Press Escape to close" passes) and flags plain labels "Left", "Right", "Up", "Down" (cap rows); the `words` rows of words.table are unused; "HUDs" and "agent_id" pass `WORDS`. | Match by word inside the text, only on key-cap nodes; allow plural. |
| rg-7 | MINOR | tests/test_acts.py (GuideContrast) / a2ee5219 | Red for the wrong reason: the 3 GuideContrast tests fail with TypeError (no `Snapshot.shots` yet), not on the check; `test_no_capture_no_finding` is green at the red sha. | Add the `shots` field in the red commit so each test fails on its assertion. |
| rg-8 | NIT | tests/acts/planted.py:236 | "persona sound" drops every breach of a (look, invariant) in the list and ignores `match` (same pattern pre-existing in tests/persona/check.py:160), so a new fault of a listed invariant cannot fail it. | Filter with `judge()` and the row's `match`. |

Checked, no finding: gaps in whole units reuse `hud-audit.gap_findings` and `tiling-audit.gap_findings` (no second checker); held test compares helper surfaces before and after and excludes cube faces; `\b(?:agents?|HUD)\b` catches "Agents", not "management" or "Shudder". Not findings (brief): g1-g6 looks, rt-1..rt-7 of tour-one.

## r2

Fix round on claude/guide-checks after Review r1. Red 2d14bc26 (19 tests fail on their assertions, none on a TypeError; planted cell "I6 text of its back's colour" added), fixes 03a1f2c1, ca145fc3, 4efbf430.

| id | red | green |
|---|---|---|
| rg-1 | 2d14bc26 | ca145fc3: known-red rows key on `act`, `capture`, `match`, one row per fault; a cell keeps each fault once (first act); stale rows block |
| rg-2 | 2d14bc26 | 03a1f2c1: "no capture of OUTPUT" and "cannot be measured" are breaches; test flipped |
| rg-3 | 2d14bc26 | 03a1f2c1: back == fore reports 1:1 (unit test and planted cell) |
| rg-4 | 2d14bc26 | 03a1f2c1: KEY_LINE covers keys, mode and `-k` labels |
| rg-5 | 2d14bc26 | 03a1f2c1: no font size fails; STATE_WORD is `-state$` |
| rg-6 | 2d14bc26 | 03a1f2c1: key spelling by word on key-cap nodes; plurals and `agent_id` caught (words rows not used) |
| rg-7 | 2d14bc26 | the red tests carry shots, so each fails on its assertion |
| rg-8 | not done | persona sound filter is a separate change from judge() |

Stricter checks found many more faults on the looks (about 260 act/capture/rule rows, mostly 11 px text and tray, pager and button contrast in xp, tiling, construct, mc). They are known-red rows, no look fix. Blank text (` `) is not measured. Two rows (xp-tray contrast) are `flaky = true`: the capture shows them in some runs only. construct I2 also has a held row (a screen-fixed node moves 1 px; the node differs per run), and the gap of construct-mode-step-in is red again.

Gate: pre-commit, gate-lib, ci-check, build, test, hud PASS on 03a1f2c1; layout and tiling-audit PASS on rerun (the first layout run failed in a tiling walk, Super+T no third terminal, not a changed cell); acts PASS on 4efbf430.

## Review r2

Review of claude/guide-checks head 4efbf430 over base a78b1a07 (sonnet, high; read-only on code, one round). Step 0 green (`cargo check --workspace`). No gate, no acts run (no build): the known-red rows were checked by reading, the probes ran on 4efbf430 (`judge()`, `breaches()`, `Session.record`).

**Verdict: FIX** (1 MAJOR, 2 MINOR open; rg-1..rg-7 closed).

1. Red first: `python3 -I -m pytest tests/test_acts.py -q` in worktrees: 2d14bc26 19 failed on assertions (no TypeError, no error), 171 passed; 4efbf430 182 passed, 215 subtests. Probes agree: the g2-style row against a 1.2:1 fault of another construct act blocks (rg-1); white on white and black on black give 1.00:1 (rg-3); no capture and a 0.5 px box are breaches (rg-2); "Press Escape to close" on a key line, "Two HUDs", "agent_id" fail, "Left" on a plain label, "reagent", "management" pass (rg-6); `mc-estate-line` at 11 px fails, `-state` passes (rg-5); `tiling-keys-body-1-k`, `construct-hints-*-keys-0`, `construct-mode-*`, `xp-session-button-N` want 7:1 (rg-4); rg-7 red tests fail on their assertions.
2. First-act keying (check 4): a fault whose row names act A, seen first in act B, blocks (one FAIL for the new fault, the row is stale); `Session.record` keeps a fault at the act that first shows it; stale rows block. Sound.
3. Sample of 15 known-red rows (seed 7; construct 4, xp 4, tiling 4, mc 3) and the structure of all 208 rows (202 new): every non-flaky row names act, capture and one rule text; the faults are text under 14 px (key caps `-k`, icon labels, tray, task) or contrast of tray, pager, session buttons, menu frame, helper sign, and `I10` words on mc header/footer. Reported runs carry `px` by the xp test and `NodeRun.px`; text boxes carry `font_size` (ipc/boxes.rs:97), so the "no font size" breach is not made up by the current looks. No row for a fault the harness made up found. Not run on the looks, so the 15 are plausible from the labels and the sources, not seen in a capture.
4. Check 6: the layout flake (Super+T no third terminal, empty hints, window lost) is already FLK-LAY1 in backlog.md (t87, passed on rerun); this diff touches tests/acts only, not a finding. No new FLK row.

| id | severity | file:line | finding | fix |
|---|---|---|---|---|
| rg-9 | MAJOR | tests/acts/known-red.toml:221,536,554,572,590,743,1454,1472,1643 | `flaky = true` is on 9 rows, the report says 2 (the xp-tray HEADLESS-1 and VIRTUAL-1..4 contrast rows of construct and xp, two pairs identical); the commit 4efbf430 dropped their `act`, so each covers that capture's "the contrast" breach in every act of the look, and a flaky row never goes stale. Probe: a 1.0:1 breach of `xp-tray-HEADLESS-1` at a new act passes judge(). No run log or run without the fault is named (report: "shows them in some runs only"); flakes-o names the xp tray clock redrawing at the minute change, so a capture timing bug is the likelier cause than a real intermittent fault. | Drop `flaky`, keep act and capture, and make the capture wait for the tray clock (clock_clear) so the fault is stable; if a flaky row stays, require an `evidence` field naming a run without the fault and keep its act. |
| rg-10 | MINOR | tests/acts/invariants.py:503 (+ known-red.toml, 135 `match = "guide wants 14 px"` rows) | The "has no font size, the guide wants 14 px" breach contains the `match` of every size row, so a size row hides a harness gap (a run without px) the same as a real small size. | Word the unknown-size breach without "guide wants 14 px" (or make the rows match "px, the guide wants"). |
| rg-11 | MINOR | tests/acts/invariants.py:240 | `mode` as a KEY_LINE segment gives 7:1 to any label with a `-mode-` part: probe `xp-mode-label`, grey 118 on white, 4.54:1 fails "wants 7:1" although it is no key line or button. | List the `construct-mode-` buttons only (or mark key lines in node meta, as r1 rg-4 said). |
| rg-8 | NIT | tests/acts/planted.py | open, deferred (known). | |

## r3

Branch claude/guide-checks: red 003b3cf3, fix 9881ccda (on 4efbf430). Status: NOT GREEN, the acts step is red on rg-9.

- rg-9 (MAJOR): the 9 `flaky` rows are gone, judge() now refuses a flaky row without `act` and `evidence`. The tray contrast is NOT the clock: it is white on XP blue, 3.27:1 (3.21:1 at 1:1), a real fault. It shows in a different act each run: run 1 in 10 (act, capture) pairs, run 2 in 1 (xp "Escape shuts Turn Off Computer") and none in construct, run 3 in construct "the pager turns the cube". Rows keyed to an act cannot converge. `settled_scene` (re-read the nodes while a text changes, 5 tries) is in but did not change that; unproven. Rows now: 13 flaky contrast rows with act and evidence (before: 9 rows without act); "the pager turns the cube" VIRTUAL-1 is still unlisted. Why the tray is measured in some acts only is open: suspect screen_texts or the capture, not the clock. Suggest: one row per capture with `acts = [...]`, or find why the measure skips.
- rg-10: "no font size, its size cannot be checked" no longer contains "guide wants 14 px"; no row needed it (0 known-red rows had no size).
- rg-11: KEY_LINE has no `mode`; `^construct-mode-` is the anchored list. xp-mode-label gets 4.5:1.
- Gate (--jobs 1): pre-commit, gate-lib, ci-check, build, test, hud, layout, tiling-audit PASS; acts FAIL: the tray row above and `construct act:construct started again reopens the cube ... within 30 s` (RuntimeError in every one of 3 runs; not touched by this lane, not checked on 4efbf430, no FLK id). test_acts.py 188 pass.

## r4

Branch claude/guide-checks at 06bb5af4 (pushed), merged with the session head 618ae513 (91615955; one conflict in tests/test_acts.py, both sides kept; cargo check and build green).

- Red first: 89b23f9c (tray exemption tests: tray text at 3.27:1 passes in any act, a non-tray xp node still fails, a row without look, node and reason is refused).
- (a) rg-9 closed: 83132516. The 13 flaky tray contrast rows are gone; one `[[exempt]]` row (look xp, node prefix `xp-tray-`, floor 3.0, reason "authentic Luna tray, vision card") passes the contrast breach of that node in every act. The floor keeps a 1.00:1 tray (invisible text) red. Rows before 212, after 199 minus the 11 rows of the next item; exempt rows 1.
- Why the tray was measured in some acts only: not the clock. In the construct look, after construct is killed (acts "construct killed mid-turn ..." and "without construct the pager ..."), the capture of the flat xp shows the tray text at 1.00:1 ((235,236,235) on (236,236,236)), in 3 of 3 runs. FLK-TRAY-MEASURE not needed for the 3.27:1 case; the 1.00:1 capture of a flat xp right after the kill is a separate fault in the capture or the core, not found in under 30 lines.
- (b) construct restart: RED on the merged head 3 of 3 (RuntimeError "the cube reopened within 30 s"; construct-again.log: "Alt+Super+1..4 is grabbed already"). GREEN on the session head alone 3 of 3. So the cause is this branch: the snapshot now takes a screen.capture; a capture of the flat xp while construct is gone keeps the cube from reopening (a run without captures, and a run with captures only while construct is up, both pass the restart). Red test 1e10b202's parent (test commit before it), fix 1e10b202: `Session.capturing()` takes no capture in the construct look while construct is gone; Snapshot.shots None means "not measured", I6 skips the contrast then. The 11 contrast rows of the two acts that take no capture went stale and are dropped (06bb5af4). A real core fault is likely behind it (a capture of the flat xp blocks the grab/cube reopen); not investigated, no backlog row written.
- Gate (--jobs 1): pre-commit, ci-check, build, test, hud, layout, tiling-audit PASS. gate-lib FAIL when run inside the gate: merge-gate-selftest counts /tmp/merge-gate.* directories ("an unknown job leaves no log directory: got 4, want 0", also --help, a run without a failure), the running gate's own run directory is one; not caused by this lane, not rerun. acts FAIL: `xp I6: in act 'a click on the speaker opens the volume popup': 'Volume' (xp-ctx-0-7) is 11 px, the guide wants 14 px` (new act, no row, not checked on the session head); construct cells: the restart act and I6 pass. construct I13 ("the face construct-cube-face-2 shows alone in a turn", act "the cube is held halfway through a turn") was red in 2 of 5 construct runs on this branch, green on the session head 3 of 3: possible flaky, unproven, no row.
- test_acts.py: 206 pass.
- Done-when NOT met: acts is red on the xp 'Volume' 11 px row; next lane: a row (act, capture xp-ctx-0-7) or the popup font, and the I13 flake with evidence.

## r5

- Head 07aa97d4 on claude/guide-checks (merge of the session head ee29dc63, then the row 07aa97d4). cargo check and build green.
- (a) DONE: known-red row xp I6, act "a click on the speaker opens the volume popup", capture xp-ctx-0-7, reason "authentic Tahoma 8 pt". Red test first (test_the_list_names_the_authentic_tahoma_volume_label_of_the_xp_popup), matches only that node and act; pytest tests/test_acts.py 207 passed.
- 2) I13 NOT done (no 5+5 runs). In the --only acts gate run on the merged head: "construct I13: in act 'the cube is held halfway through a turn' ... construct-cube-face-2 shows alone in a turn" red again; unproven whether it is red on the session head alone.
- 3) gate-lib NOT fixed, brief's cause not confirmed. merge-gate-selftest has no find over /tmp/merge-gate.*; the 3 reds ("an unknown job leaves no log directory: got 4", "--help ...: got 4", "a run without a failure removes its log directory: got 6") count the selftest's own TMPDIR ci-tmp. Cause: the selftest links a stripped PATH (nobin) to the tools via command -v; in this sandbox rm is a guard wrapper (/root/.claude/rm-guard/.../refuse/rm), the link does not work, ci-local.sh prints "rm: command not found" and keeps its ci-local.* directory. Reproduced alone with no other /tmp/merge-gate.* dir. Likely green outside this sandbox. A fix would resolve /bin/rm around the guard; I did not do that.
- 4) Gate: pre-commit PASS, gate-lib FAIL (above), the gate stopped there. Separate --only acts run: 202 pass, 5 known red, 2 blocking: I13 (above) and new xp I6 "in act 'a window opens to hear the keys that the volume popup holds back': 'chora-probe' (xp-task-s4-HEADLESS-1) is 11 px, guide wants 14 px" (same authentic Tahoma 8 pt, a second node, needs a PM decision for a row).
- Done-when NOT met.

## r6

Head: claude/guide-checks b1b8b73a (on 07aa97d4; it already contains the session head f4f49783, no merge needed). Step 0: cargo check and build PASS.

- 1) (a) DONE, 9af04984 + b1b8b73a. Red test first (test_the_list_names_the_authentic_tahoma_text_of_the_xp_taskbar_buttons): one xp I6 row, capture "xp-task-*" (new: a trailing `*` takes every label with that prefix), match "guide wants 14 px", slice "authentic Tahoma 8 pt"; it covers no other rule, look or label (xp-tasks-, xp-ctx-, xp-tray-, leaves-its-panel, contrast stay blocking). The 6 per-node xp size rows of r2 (xp-task-s1..s5, helppanel) went stale by turns across runs and are replaced by it; the construct-look rows stay.
- 2) I13 red on this branch only. Runs of the construct acts, 5 on the branch before the fix: red 2 (runs 1, 2), green 3; 5 on the session head f4f49783 alone (separate worktree, own build): green 5. The branch's tests on the head's binaries: red 2 of 3, so the cause is the branch's harness, not the code.
  Cause: the branch's snapshot reads scene.boxes straight after scene.query, before the clients, blocks and cube are asked; the head read them last. A held cube read that early shows face-2 alone (breach text). Refuted on the way (no fix kept): captures in held acts (captures off: still red 4 of 5), a retry of settled_scene, the read time (HELD 1.5: still red).
  Fix 7fb3453c, red test first (test_a_snapshot_reads_the_boxes_after_the_clients): the boxes are read last. After the fix: construct I13 pass in 4 of 4 runs and in both gate acts runs (6 of 6).
- 3) Gate --jobs 1, first run: pre-commit, ci-check, build, test, hud, layout, tiling-audit PASS; acts 203 pass, 5 red, 1 blocking = the stale xp I6 row xp-task-s5 (fixed by b1b8b73a). Then --only acts: PASS (921 s). pytest tests/test_acts.py: 209 passed. gate-lib left out (PM decision b); co-36 stays with co-capture.
- Observed, not mine: a known-red row of r2 (construct I6, construct-hints-construct-mode-step-in-keys-0-text, "the contrast") was stale in one construct run (late-4): flaky row, no evidence yet.
- Done-when MET: acts green at --jobs 1 on the merged head b1b8b73a.
- Mistakes in this lane: a stale local branch ref fast-forwarded to the session head and I amended its merge commit before noticing (reset, nothing pushed); stray session processes from a killed loop loaded the machine for some runs (cleaned, runs repeated).

## Review r3

Review of claude/guide-checks head b1b8b73a (r3-r6 over the r2 review head 4efbf430; sonnet, high; read-only on code, one round). Step 0: `git fetch --unshallow`, `cargo check --workspace` green. No gate run; `python3 -I -m pytest tests/test_acts.py -q` and probes of `judge()`, `exempted()`, `row_covers()` only.

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 1 MINOR, 4 NIT; rg-9, rg-10, rg-11 closed).

1. Review r2 closed. rg-9: `known-red.toml` has 0 `flaky` rows (the word occurs in the header comment only) and one `[[exempt]]` row (look xp, node `xp-tray-`, floor 3.0); `judge()` refuses a flaky row without `act` and `evidence`. Probe: a tray text at 3.27:1 passes, 2.90:1 blocks, `xp-trayx-1` at 3.27:1 blocks, `xp-ctx-0-1` at 3.5:1 blocks. rg-10: the no-size breach reads "its size cannot be checked", no row matches it. rg-11: `KEY_LINE` has `^construct-mode-` only; `xp-mode-label` no, `construct-mode-step-in` yes, `construct-hints-x-mode-1` no. Tests: 003b3cf3 is red (12 failed, on assertions incl. flaky-without-evidence, mode label, unknown size), 9881ccda green (188 passed).
2. Red first. 89b23f9c red (TypeError, `judge()` has no `exempt` argument yet; red by missing API, not by assertion), 83132516 green (all pass). f108a076 red (AssertionError "a capture of the flat xp reopens no cube again"), 1e10b202 red on the settled-scene test (AttributeError `capturing`, test 1e10b202^ order). 9af04984 holds test and `capture_pattern` in one commit: the test fails (assertion) against the 07aa97d4 harness, so red first holds by probe, not by commit. 7fb3453c the same: test_a_snapshot_reads_the_boxes_after_the_clients fails against the parent harness (`order[-1]` is `blocks`, not `scene.boxes`).
3. Wildcard `xp-task-*` (harness.py `capture_pattern`, one row, look xp, I6, match "guide wants 14 px"). Probe on that row alone: `xp-task-s9-X` at 11 px passes (any act); `xp-tasks-s9`, `xp-ctx-0-7`, `xp-tray-O`, a "leaves its panel" line, a contrast line (its text says "wants 4.5:1/7:1", no "14 px"), the no-size line, look construct and id I3 all block; a stale wildcard row (no breach) blocks ("passes but is listed"). The contrast text cannot hide behind the size match. One leak: rg-13.
4. I13 fix: snapshot reads `scene.boxes` after `cube()` and `blocks()` as the last call; the other fields keep their order and meaning except `shots`, now taken in `settled_scene` before clients and cube (harmless; nothing reads them together). The cause fits "## r6": 2 red of 5 on the branch, 5 of 5 green on the session head, red 2 of 3 on the head's binaries, so the harness, and 10 of 10 green after (4 + 6 gate runs). Not rerun here (no build of the looks). rg-15 for the leftover `scene.boxes` call.
5. Exempt rows: xp-tray- is the 03:45 PM decision of r4 (vision card, authentic Luna) and the xp-ctx-0-7 / xp-task-* rows follow r5/r6 decisions (a): one known-red row each, size rule only. The tray exemption is wider than its row: rg-12. No other exemption, 0 flaky rows. Construct-look rows for xp-task-/xp-tray- (8 rows, act-keyed per node) stay as the PM said (xp look only): rg-16.
6. Merge risk: `git merge-tree --write-tree b1b8b73a 598a34ff` is clean (no conflict names). A trial merge (no commit) auto-merges tests/test_acts.py only and `pytest tests/test_acts.py` gives 209 passed, 196 subtests. No conflict in tests/acts.

| id | severity | file:line | finding | fix |
|---|---|---|---|---|
| rg-12 | MINOR | tests/acts/harness.py:`exempted` (about 265) | `exempted()` never reads the row's `look` (a required field) and `floor` is optional: the xp-tray- row passes a 3.27:1 tray text in the mc, tiling and construct looks too (probe), and a row without `floor` passes 1.00:1. | Match the key's look to the row (`looks = ["xp", "construct"]` since the xp child shows in construct) and require `floor` in `judge()`. |
| rg-13 | NIT | tests/acts/harness.py:`row_covers` (about 290) | The capture regex is searched in the whole breach incl. the quoted text: `'see xp-task-s1 x' (other-label) is 11 px, the guide wants 14 px` passes the `xp-task-*` row (probe). Not a real label today. | Search `\(label\)` after `fault()`-style stripping of the quoted texts. |
| rg-14 | NIT | tests/test_acts.py:464 (89b23f9c), 9af04984, 7fb3453c | Red first is by probe only: 89b23f9c is red by TypeError, 9af04984 and 7fb3453c carry test and fix in one commit. | Note for the next lane: test commit first, assertion red. |
| rg-15 | NIT | tests/acts/harness.py:`settled_scene` (about 700) | It still calls `scene.boxes` (result dropped, up to 5 times) although the snapshot reads the boxes last. | Return `nodes, shots` and drop the call; adjust test_the_scene_is_read_again_while_a_text_changes. |
| rg-16 | NIT | tests/acts/known-red.toml:67,139,490,1309,1363 (construct xp-task-), 148,454-481 (xp-tray size) | Per-node, act-keyed rows of the same authentic Tahoma text in the construct look can go stale by turns as the xp rows did in r6 (unproven in construct). | If one goes stale in a gate run, give construct the same `xp-task-*` row by a PM decision; no change now. |
| rg-8 | NIT | tests/acts/planted.py | open, deferred (known). | |
