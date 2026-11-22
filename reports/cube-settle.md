# cube-settle

## r1

Branch claude/cube-settle: red 1dc30d2c (CUBE_LOAD=1 switch), green ffb197f3 (fix). The full `merge-gate.sh --jobs 2` was NOT run: stopped on the open item below.

Measured (cube cell, tests/gate-construct-cube.sh, bar read wall time with the calls):
- idle x5: 402, 360, 351, 346, 342 ms
- 4 busy loops (1 per CPU, nproc 4) x4: 352, 511, 299, 386 ms (run 5 killed by my timeout). 4/CPU: 497 ms. 8/CPU: 600, 670 ms. 16/CPU: other steps red first.
- The construct exposes no settle prop or log line. The animation is `Turn::new(.., now, SETTLE)` with SETTLE = 320 ms on `Instant` (plugins/chora-construct/src/cube.rs:61, :841): by time, not frame count.

Split: the product is fine (320 ms by clock, time based). The 729 ms was the gate: `released=$(now_ms)`, then `quick` polls with ctl and python calls, each slowed under load, all counted. The turn key bar (`turn_start`, `turn_took`) had the same pattern: red at 8/CPU, "a turn key took 892 ms" and 855 ms.

Fix: `settle_poll` takes the start of the last poll that still saw the cube unsettled (a sound lower bound: unsettled then means unsettled earlier). The bar is on that; 5 s wall is only the hang guard. No raised bar, no sleep. Under load the 49-degree step now reports "not settled at 0 ms, settled by 357-635 ms with the calls".

After, CUBE_LOAD=1 (4 busy loops per CPU) x10: the 49-degree and turn key bars 10/10 ok; the whole cell 8/10 green. The two reds are later in the cell and not the settle bar:
- run 2: "floating windows: the face after the step in differs from the face before the step back"
- run 8: "the window does not float during a turn" (`until_ok ... 6`, 6 x 0.25 s after `sleep 0.4`, a wall poll)
At 8/CPU that line and "the hint Step back names the keys" go red as well. Open: a further load-sensitive wall poll in the float steps (needs its own row, not filed by me).
Note: the red commit has 8/CPU, the green commit 4/CPU.

## r2

Branch claude/cube-settle at e905195e (pushed): red f0778522, green e0e1f8ca (floats), green e905195e (turn). Base already contained origin/claude/autonomous-dev-session-3w-i33pg7 (merge was a no-op).

Split (both lines are the gate, the product is fine):
1. "floating windows: the face after the step in differs from the face before the step back" (tests/gate-construct-cube.sh `floats`): the two captures differ only in the bottom hint capsule (bbox 480,696,800,760; a capture 3 s later equals the first). The hint Step back shows 3 s after a step in (hints.rs FLOWN). `flat.png` was taken after `sleep 1`, `landed.png` after `sleep 0.5`: bare sleeps against a timer. Idle, both fall inside the 3 s; under load the calls pass 3 s, so flat has no hint and landed has one. A time-dependent element, not a product change. Red message now names it: "(hint Step back: gone before, True after)".
2. "the window does not float during a turn" (`until_ok ... 6` after `sleep 0.4`, a 1.5 s wall poll against a 3 s turn): under load the polls (prop calls) run past the turn, the plane is gone when read.
3. "the hint Step back names the keys" (:339) did not go red at 4/CPU; it was not touched. Not seen at 4/CPU in 30 runs.

Red: CUBE_LOAD=1 at 4/CPU on ffb197f3: 1/10 green, 9 reds all at line 1 (the floats line). With the red commit's message, run 4 of 4 showed "gone before, True after". Line 2 was red 1 in 10 (run 5 of the check run after fix 1), no separate red commit for it: intermittent, no deterministic red.

Green:
- floats: waits with `until_ok hint_gone 40` before each capture (state: hint expired), no bare sleep. After fix 1: 9/10, the one red is line 2.
- turn: the slow turn is 10 s as a stimulus (was 3 s), no `sleep 0.4`; both faces and the float plane are polled by state (40 x 0.25 s guard); the end of the turn has a guard of 80 x 0.25 s. No bar raised. The cell is about 7 s longer.

After, whole cube cell, CUBE_LOAD=1 (4/CPU) x10: 10/10 green (rc 0, 163 to 184 s each).

Gate: `tests/merge-gate.sh --jobs 2` (no --only) on e905195e: 57 PASS, no FAIL, EXIT 0 ("all steps passed").
