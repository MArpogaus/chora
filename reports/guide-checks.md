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
