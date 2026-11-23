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

## Review r1 (PM)
- FLK-TILING-LAYOUT: red ecf021d4, fix 76c06d05, follow 10/10 under load; full gate 60 PASS.
- Done-when not met: the tiling-windows cell under load is not 10/10 (focus 3/3 red, motion 1/3 red); T1b not fixed. Verdict: INCOMPLETE (procedures: a failed Done-when line). r2: BORDER, MOTION and ti-46 on the same branch; merge after r2.
- T1b (MINOR) leaves the lane: a design item (who owns the bar surface), backlog.

## r2

Branch claude/tiling-flake, head 15a5d2fc (merge of origin/claude/autonomous-dev-session-3w-i33pg7 at 230ea0e5). All three are test-side; no product fault.

### FLK-TILING-BORDER (tests/gate-tiling-focus.sh)
- Cause: the border walk used fixed rows, x=12/634/645/1267 over y 30..770, the layout of a screen without the bar. Under TILING_LOAD=4 the bar is in at the capture: the rings of the windows start at y=52 (tb-s1 [646,52,622,736], tb-s2 [12,52,622,736]), so y 38..58 at x=12 is the bar gap and the corner arc, not a broken ring. The state is right (all border nodes visible, tg-s2 opacity 0, tg-s1 opacity 1); the check read the wrong rect. No product fault.
- Red 13f61462 (failure names the rings at the capture; red under TILING_LOAD=4, 3/3 earlier, 1/1 here): `broken at [(12, 38), (12, 42), ...]; the rings at the capture: [{"id":"tb-s1","r":[646.0,52.0,622.0,736.0]},{"id":"tb-s2","r":[12.0,52.0,622.0,736.0]}]`.
- Green 2aa2466e: the walk follows the rings from `ring_rects` (query of the tb- nodes), and a capture counts only when the rings are equal before and after it (40 rounds as hang guard). Under load 4/4 green.

### FLK-TILING-MOTION (tests/gate-tiling-motion.sh)
- Cause: after view_tag 1 the check sampled the groups once and again after `sleep 1`, and asked the first sample to be >200 px from the second. The slide is 200 ms of clock; under load the sample comes after it (first sample at x=14 = final) or late in it (-9 against 14). Seen at TILING_LOAD=6: 2/8 red.
- Red a68aeb1a (message names both samples; red only under load, 2/8): `first sample {'tw-s1': 14.0, ...}, settled {... 14.0 ...}`. A sampling check has no deterministic red at the commit level, the stepped version below removes the sampling.
- Green f97f3e35: the hide on tag 2 is waited for (visible 0 of every tw- group), then `choractl record --frames 24` with a script that calls view_tag 1 at frame 1 (the stepped clock shows the scene one frame time after the call whatever the load): the window is not at its place at frame 2 and is in frame 24 and in a capture after the settle. Under TILING_LOAD=6 8/8 green.

### FLK-DOORS ti-46 (tests/doors/tiling.sh:129)
- Cause: `helper_says "coder · idle"` read the bar word once; the word follows the helper by a poll.
- Red ded5d30d: the cell restarts voice and starts the helper 3 s late (stand-in lag, background subshell); word is `No helper`, helper-door missing.
- Green 0483bbeb: `until_ok 'helper_says "coder · idle"' 40` (20 s hang guard); helper-door pass with the lag.

### Counts
- The whole tiling-windows cell (focus, follow, doors, swallow, gaps, motion) under TILING_LOAD=4, 10 runs one after the other: 10/10 green. Not two at a time: the gates share a lock.
- `TILING_LOAD=4 ONLY=helper-door tests/gate-doors.sh tiling`: 10/10 green (with the 3 s lag in the cell).

### Gate
`tests/merge-gate.sh --jobs 2` (full, detached) on 15a5d2fc: EXIT 0, "all steps passed", 62 PASS lines, no red, no skip. `pgrep -fa 'chora --headless'` empty.
