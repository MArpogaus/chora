# tiling-flake

## r1

Branch claude/tiling-flake, head 8f2ed754 (pushed). Base 8721b4d6, merged with origin/claude/autonomous-dev-session-3w-i33pg7 (dac7f6bc; one conflict in tests/gate-lib.sh, both sides kept).

### FLK-TILING-LAYOUT
Cause: tests/gate-tiling-follow.sh (burst/changes, old lines 59-81): the time ran from `now_ms` taken BEFORE `ctl call tiling next_layout` to the start of the first capture that showed the change, so a slow ctl spawn and a slow capture on a loaded host counted as delay of the bar. Not a product delay: pill and word follow within about 100 ms unloaded.
- Switch `TILING_LOAD=N` in tests/gate-lib.sh `session_start` (N busy loops per CPU). Red ecf021d4. Red line under TILING_LOAD=4: `layout: 391 ms` / `FAIL: the layout word took 391 ms to follow the change` (1 of 7 runs red; the others 76-144 ms).
- Fix 76c06d05: start after the call returns; the bar (250 ms, unchanged) is on the start of the last capture that still shows the old state, a sound lower bound (as in cube-settle); a burst that never shows the change is the hang guard (9999). No sleep, no known row. The printed ms is now mostly 0.
- After: gate-tiling-follow.sh under TILING_LOAD=4, 10 runs: 10/10 ok.

### Whole cell under load: NOT 10/10
Cell tiling-windows (focus, follow, doors, swallow, gaps, motion) under TILING_LOAD=4:
- focus: 3/3 RED, `FAIL: the border is broken` (broken at [(12, 38), (12, 42), ...], left border of the old window after Super+j). Persistent over a 40-capture re-wait, so not a settle race; green unloaded and in the full gate. Not fixed (the re-capture loop was reverted). New finding FLK-TILING-BORDER.
- follow, doors, swallow, gaps, motion (focus left out): 2 of 3 runs green (about 455 s each), run 3 red in motion: `FAIL: a tag switch does not bring the windows in from the side`. New finding FLK-TILING-MOTION. Only 3 runs, not 10 (8 min per run, cost).

### T1b (pointer on tiling's bar gives an xp window): NOT FIXED
Tried the claim of the full outputs (red 25ee0de8, green 22b82d63), then the claim up to the top edge (red 8b7c2db4, green ccce3e6f). Both turn gate-join-keys xp-first red ("Super+Return opened a window of tiling, not of xp"): there the work area is [0,40,1280,730] with no tiling bar, so the top 40 px are xp's and the default pointer is on them. Geometry of the claim cannot tell tiling's bar from xp's panel. All four reverted in 8f2ed754; gate-join-keys ok again. Needs the owner of the bar surface (layer-shell surface of waybar) or a bar rect sent by the plugin: a design decision.

### Gate
Full `tests/merge-gate.sh --jobs 2` on 8f2ed754 merged with fresh autonomous branch: 60 PASS, `EXIT 0`, no red (run dir /tmp/merge-gate.Iem1AC). An earlier run on ccce3e6f stopped red at matrix xp-first (the T1b claim), mine, since reverted.
`pgrep -fa 'chora --headless'` empty at the end.
