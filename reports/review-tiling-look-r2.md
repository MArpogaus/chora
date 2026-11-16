# Review tiling-look r2 (read only)

Head e1dda18f (round 1 head 47fef21b), merge-base with base 13f86740; base now aba97fec.
31 files, +607/-57 over merge-base (r2 alone: 2b0ccd2e..e1dda18f).

## Verdict
MERGE-OK (computed: no open BLOCK or MAJOR). One real merge conflict (rv-1) to resolve at merge.

## Findings
| id | sev | where | finding / evidence |
|---|---|---|---|
| rv-1 | MINOR | flavors/tiling/waybar/config.jsonc:93-96 | Trial merge conflicts: base changed the Apps tooltip text ("... or a typed command ..."), the lane removed that tooltip. Resolution: take the lane (`"tooltip": false`, no tooltip in exec). tests/merge-gate.sh "# Steps:" header also conflicts (known). Other files auto-merge. |
| rv-2 | MINOR | flavors/tiling/waybar/config.jsonc:93-131, helper.sh | ti-23 is fixed by deleting the tooltips of the four doors (Apps, helper, Keys, power), not by making them leave with the pointer. Mouse users lose the key hints ("Super+Space", "Super+F1", "Super+Shift+e") the design had; the key list still has them. Matches the ti-6 suggestion in docs/known-issues.md:16, so a decision, disclosed. Data tooltips (clock, network, volume) can still stand over a tile (disclosed). Cell: gate-tiling-look-r2.sh tooltip passes. |
| rv-3 | LOW | tests/gate-tiling-audit.sh:176, tests/tiling-audit.py:332 | The audit cell "tooltip (that of Keys)" still runs and passes, but Keys has no tooltip now, so it checks nothing new. Rename or drop it. |
| rv-4 | LOW | tests/gate-tiling-audit.sh:120 | `ctl wait id:hud-row-chat` replaced by a fixed `sleep 3` (the row no longer exists). A sleep is a weaker sync than a state wait; the cell passed here. |
| rv-5 | LOW | tests/gate-tiling-look-r2.sh cell_power (about line 160) | Power cell only counts more than 20 pixels differing from one ground sample at (1225,12) in a 20x16 box. It would pass for any glyph. The mouse path to lock/power (gate-tiling-power.sh, passes here) is not in tests/merge-gate.sh at all, so the report's "3 of 3 clicks" claim for ti-24 has no gate step. |
| rv-6 | LOW | plugins/chora-tiling/src/keys.rs (the_doors_of_the_bar_have_no_tooltip) | Unit test added in the last commit e1dda18f, after the fix c1adc900 (not red first). The gate cell was red first (633fc0d3), so the rule is met by the cell; the unit test is a guard only. The older test's `named` count went 4 to 1 (only layout.sh left), fine. |
| rv-7 | LOW | plugins/chora-assistant/huds/tiling/hud.yaml:10 | New tiling HUD folder is neutral minus the agents panel; the dot click is `chat` instead of `mute` (docs/hud.md says the figure click mutes). Intended (bar word and dot both open chat), covered by README; the mute path in tiling is gone. Duplicate of neutral (44 lines + svg) instead of a rule dropping rows; KISS-borderline but small. |
| rv-8 | COSMETIC | tests/gate-tiling-look-r2.sh door cell | The door cell asserts "No helper" and no `hud-row-*` node in both states, but with a live helper it does not assert the bar word equals the helper state (helper-word is covered by gate-hud, passes). |

## Brief items
- ti-22 fixed: red 633fc0d3/81ecfe99, fix b1412720, cell `notice` green (saved with path, failed), both themes. Honest: checks makoctl list text, file path, and pixel change.
- ti-23 fixed (see rv-2): red 633fc0d3, fix c1adc900, cell `tooltip` exact pixel diff 0, both themes.
- ti-24: pill fixed (red b3c95c8e, fix 3ba4fb22, cell `door`); power icon already present (cell weak, rv-5).
- r1 MINOR docs/xp-split.md:282 fixed (d220b3e6, now "on by default").
- No regression seen in xp, mc, construct: diff touches only tiling flavor/plugin, the new huds/tiling folder, a shared screenshot.sh (notifier fallback, only active with CHORA_NOTIFIER), and a rule appended last in `huds` (space: tiling). The doors gate (all looks) passes.
- KISS: acceptable; screenshot.sh adds a mako start path (10 x 0.3 s retry) only when no notifier answers.

## Commands run (worktree /home/user/wt/rev-tiling at e1dda18f, CARGO_TARGET_DIR inside)
- cargo build --locked: ok (3m30).
- cargo test --locked -p chora-tiling -p chora-assistant -p chora-kit: ok (tiling 141, assistant 228, kit 199 + 5, 0 failed).
- tests/gate-tiling-look-r2.sh (ink, paper): all ok, 2m00.
- tests/gate-tiling-wobbly.sh: ok. tests/gate-tiling-bar-palette.sh: ok (ink, paper; 1x, 2x).
- tests/gate-tiling-audit.sh: ok. tests/gate-chat-keys.sh: ok. tests/gate-doors.sh: ok (all looks). tests/gate-hud.sh: ok (65 s). tests/gate-tiling-power.sh: ok.
- Not run: full tests/merge-gate.sh, layout, keys (construct), 10x repeat of the new cells (one run each only; the brief asks 10 of 10, taken from the lane report, UNVERIFIED here).
- Red-first of cells not re-run on r1 code (taken from the lane report).

## Merge conflicts (trial merge with origin/claude/autonomous-dev-session-3w-i33pg7 aba97fec, aborted)
- flavors/tiling/waybar/config.jsonc: custom/apps exec line (rv-1). Take the lane side.
- tests/merge-gate.sh: "# Steps:" header only (known). Keep the base list and add `tiling-look` after `tiling-audit`; the step line itself merged cleanly.
