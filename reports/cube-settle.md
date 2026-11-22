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
