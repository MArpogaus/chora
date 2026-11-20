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
