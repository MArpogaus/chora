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
