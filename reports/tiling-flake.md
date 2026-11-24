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

## Review r2 (PM, 09:29 UTC, 11-23; builder: sonnet lane, not the PM)
Own diff 8f2ed754..15a5d2fc without merges: 6 commits, tests only (gate-tiling-focus.sh +17 -5, gate-tiling-motion.sh +20 -11, doors/tiling.sh +5 -1). Lane gate on fresh 230ea0e5: full, EXIT 0; counts 10/10 under TILING_LOAD=4.
- BORDER (2aa2466e): the capture is taken only when the tb- ring rects are the same before and after it (40 tries, else a named fail); the check reads each ring's own edges (1 px in, 16 px off the corners) instead of fixed rows. Holds: a broken edge of any ring still fails; the fixed-row check could not follow the I2 gap of 12.
- MOTION (f97f3e35): the old check compared one late sample after ; now tag 1 is hidden first, then choractl record steps 24 frames: frame 24 equals the settled capture and frame 2 differs at 330 and 960. Holds and is stricter (a jump-cut fails at frame 2).
- ti-46 (ded5d30d red, 0483bbeb fix): the red commit starts the helper 3 s late; the fix waits with until_ok 40. Holds.
- Red first: 13f61462 and a68aeb1a only sharpen the failure text (the reds were load-bound flakes, shown under TILING_LOAD=4 in "## r2"); ded5d30d is a real red. Not a finding (flake fixes; the H-RF note of friction 11-22 21:30 applies to product fixes).
- Note (no finding): the ring-based border check no longer pins where the rings lie; placement stays with gate-tiling-layout and I2.
**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR). Joins train 115 after t114.

## r3

Branch claude/tiling-flake-r3, head f68f596a (pushed; merge of origin/claude/autonomous-dev-session-3w-i33pg7 at 2235551f over 4e257c06). Both cells are test-side; no product fault, plugins/chora-tiling untouched.

### sh-39 gate-tiling-apps-click ("a click on the Zebra row did not start zebra")
- Reproduced: 1 red of 30 runs with the cell beside a running tiling-motion cell (CHORA_GATE_LOCK_HELD, as in --jobs 2); 0/10 alone, 0/10 at TILING_LOAD=4, 0/10 at TILING_LOAD=12. Red line: `FAIL: a click on the Zebra row did not start zebra`.
- Cause: tests/gate-tiling-apps-click.sh:52 (old), `until_ok 'ctl capture ... && ! cmp -s empty.png launcher.png'`: the launcher counted as shown when ANY pixel differed from the empty capture. The bar clock (text at x 581..697, y 18..30 in the capture, minute tick, seen in a diff of two captures) or any bar change is such a pixel; fuzzel's surface is not yet in the scene then (a query at that moment lists none). The inject goes out before fuzzel is up and the click lands on the bare desktop. Not a product delay.
- Red 4e3291b4: the cell starts a fuzzel stand-in that waits 3 s (PATH wrapper) and switches tag after the empty capture (stands for the clock tick). On 4e257c06 logic: 3/3 red, same line.
- Green 3da841b9: the launcher is up when the pixel of the Zebra row (600,331) differs from the empty capture's. No sleep, no longer timeout, no retry. With the lag and the bar change: 3/3 green, then 20/20 at TILING_LOAD=4 and 60/60 at TILING_LOAD=4 with a tiling-motion loop beside it.

### sh-40 gate-tiling-motion ("the slide is not between the places")
- Reproduced: 1 red of 4 with TILING_LOAD=2 motion beside an apps-click loop at TILING_LOAD=4; 0/10 alone, 0/10 at TILING_LOAD=4, 0/20 as two motion cells at TILING_LOAD=2 side by side. Red text: `({'tw-s1': 14.0, 'tw-s2': 648.0}, {'tw-s1': 648.0, 'tw-s2': 14.0}, {'tw-s1': 14.0, 'tw-s2': 648.0})`: the first sample equals the end.
- Cause: tests/gate-tiling-motion.sh:57-67 (old): one `groups` query after `ctl call tiling set_layout`, asked to lie strictly between the places; the slide is 200 ms and under load the ctl spawn plus the query come after it (the same one-shot sample as FLK-TILING-MOTION in r2, in the layout part of the cell).
- Red f65d21b9: the query after the call is 0.3 s late (stand-in for the loaded host), failure text names first sample, settled, end: red every time with the sh-40 line.
- Green e7c46b84: set_layout is called at frame 1 of a `choractl record --frames 24` (stepped clock), the end is waited for until two reads of the groups agree, and the check reads the frames: a frame among 2..12 must show a window in the gap (640,400) that is neither the settled picture's nor the final capture's (4 of 11 frames at rest). A jump-cut shows none and fails. Under TILING_LOAD=4: 20/20 green (95-105 s per run when loaded).

### Counts
| cell | alone | under load | after fix, under load |
|---|---|---|---|
| apps-click | 0/10 red | TILING_LOAD=4 0/10, 12 0/10, beside motion 1/30 | 20/20, 60/60 green |
| tiling-motion | 0/10 red | TILING_LOAD=4 0/10, 2x beside each other 0/20, beside apps-click 1/4 | 20/20 green |

### Gate
`tests/merge-gate.sh --jobs 2` (full, detached) on f68f596a: EXIT 0, "all steps passed", 65 PASS lines (tiling-windows 182 s, e2e 270 s), no red. `pgrep -fa 'chora --headless'` empty.

## PM review r3 (f68f596a, base 4e257c06; 2026-11-24)
Diff of the lane (4e257c06..e7c46b84): 2 files, +35 -15, tests only; f68f596a merges the session head.
- sh-39: red 4e3291b4 (a bar change counted as the launcher showing; a late fuzzel stub makes it reproducible), green 3da841b9 (waits for the Zebra row pixel). Red first: yes.
- sh-40: red f65d21b9 (slide sampled after its 200 ms), green e7c46b84 (choractl record, 24 frames; frames 2-12 must show a window in the gap). Red first: yes.
- Gate: full, EXIT 0, 65 PASS (lane report).
- Ratchets on d40142e4 + f68f596a (merge 0f3af52a, scratch): sleep 779/779, sleep-all 930, size 0 over, lint ok.
Findings:
- rtf3-1 NIT: the stub line `printf '#!/bin/sh\nsleep 3\n...'` is not counted by the ratchet regex (`\nsleep` has a letter before it). It is a stand-in for a slow program, not a test wait, so it is fine; a later ratchet change may count it.
- rtf3-2 NIT: the poll loop in gate-tiling-motion.sh still steps by `sleep 0.25` (it replaces `sleep 1`, net 0).
Verdict (computed): 0 BLOCK, 0 MAJOR -> MERGE-OK. Joins the next train.
