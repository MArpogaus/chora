# matrix-cap

## r1

Branch claude/matrix-cap, base 05878d2a, two commits: 25c8e24e (phase lines, two blind waits become conditions), b71c5096 (cap 315 s).

### Where the time goes (3 runs on 05878d2a, quiet box, 4 CPUs: 214, 229, 214 s)

| part | seconds | note |
|---|---|---|
| 12 voice cells | 79 to 81 | a turn is 6.2 to 6.6 s: the fake mic speaks 2.8 s of real time plus 0.8 s end wait, then ASR, LLM, tool call, and the `place` of the next cell; the first one also starts the voice |
| 12 key cells | 28 to 32 | the 3 construct-source cells hold sleeps of 1.5, 1 and 0.5 s on purpose (the walk takes the keys while the construct stands still) |
| give, cgive, cgive-s, mcp | 16 to 21 each | 1.3 to 1.7 s a cell: `place` (focus, rects stable 0.3 s), the path, 0.2 s polls |
| drag | 17 to 21 | |
| startup (session, 4 outputs, 3 plugins, foot) | 5 | |
| voice start | 3 | the greeting plays out |
| panes cell | 11 | 3 fillers and 3 drags |
| leak cell | 0 | |

Cells sum to 197 to 212 s; startup, voice start and panes make up the rest. One traced short run (4 cells) shows no gap longer than 0.3 s except a 2 s step before the grants of session start (not looked at further).

### What grew since 192 s

The matrix script has not changed since 11-14 (a86053b0, the panes cell, about 11 s). The 192 s of the ledger (11-14) is the first run with the panes cell. The runs of 11-17 to 11-19 (192 to 231 s) are the same script: the spread on one head is 214 to 229 s here, so most of the change is the box, not a part. I did not build an older commit to bisect; the per-part times above are the baseline for the next change.

### Fix

Two blind waits became conditions (about 4 s in total, 214 s to 217 s measured in one run: inside the noise, so no claim beyond the 4 s of the phase lines: voice start 3 s to 1 s, panes 11 s to 9 s):
- voice start: `sleep 2` after the TTS fake is ready now waits for the `eof` of play.log (the greeting played out), at most 10 s;
- panes cell: `sleep 1` after each `window.show` of a filler now waits until `window.list` says the pane is `shown`, at most 5 s.
The voice cells and the panes cell still check the outcome, so the wait holds: a greeting that never ends or a pane that never shows costs the full timeout and fails the cell as before.

Not changed: the voice turns (real time, the pipeline's own end wait), the construct key sleeps (the test of the stand-still), shorter timeouts (none).

The matrix log now has `phase startup`, `phase voice-start` and `phase panes` lines, so the next growth is named at once.

### Cap

`tests/merge-gate.sh --only matrix,layout,doors --jobs 3` (three long gates beside each other, as in a train), 5 runs on 05878d2a plus the script before the cap change: matrix run time 226, 234, 228, 249, 243 s; the runs of 249 and 243 s failed on the cap alone (nothing else red). Cap: 249 s + 25 % = 311 s, set to 315 s (`CAP` still overrides it). A hang is still caught.

After the change, 5 runs under the same command: 225, 226, 225, 227, 221 s, all green.

### Merge gate on the head (b71c5096)

`tests/merge-gate.sh --jobs 3`, matrix step passed each time (the step runs the matrix and three more gates: 314 s, 304 s):
- run 1: construct-hud FAIL ("dark-1280x720@1.5: no claim of the indicator", the known flake of mv1 r2, mv3 and mv4 under jobs 3);
- run 2: cube FAIL (cube.open 387 ms against the 300 ms of docs/cube-modes.md, timing under load; nothing that this change touches).
- run 3: all 40 steps PASS (exit 0), matrix script 241 s: it would have failed the old cap of 240 s at 1 s; the step took 329 s with the three gates after it.

Matrix script times inside the three full runs: 228, 218, 241 s. The two reds of runs 1 and 2 are other gates' timing under 3 jobs on 4 CPUs; construct-hud and cube are not changed here.

## Review r1 (PM, 17:25; builder: sonnet lane; diff 1 file, +28 -9)
Checks: base 05878d2a in 85764974, no change to the file since (no conflict); bash -n clean; two
fixed sleeps became polled conditions; cap 240 -> 315 s from five runs under --jobs 3 (worst 249 s
+ 25 %), documented as a hang guard, not a speed test; phase lines per part.
- MINOR m1: both new polls fall through silently on timeout (eof 10 s, pane 5 s); the later cell
  checks still catch a missing pane, so no false pass. Known issue.
- MINOR m2: MX1 asked which time the cap measures (a rerun printed 322 s as PASS); the report does
  not answer it. Stays open in backlog MX1.
Verdict (computed): no open MAJOR -> MERGE-OK b71c5096. Harness branch: its own train 67 first.

## mx1-m2 r1

Branch claude/mx1-m2, base f2b7112a (train 69), head 5197fa22, two commits: 8003f052 (red: gate-lib-selftest cell), 5197fa22 (fix). Files: tests/gate-lib.sh, tests/gate-handover-matrix.sh, tests/gate-lib-selftest.sh.

### Which time the cap measured (review m2, closed)

The cap compared bash `SECONDS` with 315 s. `SECONDS` counts from the start of the script's shell, and `gate-lib.sh` takes the gate lock when it is sourced, so a matrix that waited in the queue for another gate had that wait in its run time (the 322 s line of train 65 and the 242 s of train 66 on the PM host). Under `tests/merge-gate.sh` the gate holds the lock and sets CHORA_GATE_LOCK_HELD, so the sub-gates never waited there: the old and new numbers agree for a matrix run inside the merge gate.

### Change

- `gate-lib.sh`: `gate_t0=$SECONDS` right after `gate_lock`, and `gate_elapsed` (seconds since the gate has the lock).
- `gate-handover-matrix.sh`: the cap check at each cell, the final check, the `run time:` line and the startup phase use `gate_elapsed`/`gate_t0`, not `SECONDS`. The start of the measure is the matrix's own start (lock held, before the session starts), the same point as the five runs that set the 315 s cap, so the cap value is unchanged. The PASS line is now `ok: gate-handover-matrix: every cell as in EXPECTED in 232s of at most 315s`. A breach (at a cell or at the end) fails with `...; the five slowest cells: SRC DST PATH Ns; ...`, taken from the cell lines of matrix.log (phase lines left out).
- Not done: a measure from the first cell instead of the start (the startup of 5 s would drop out of the cap; the cap would then need a re-fit from runs). Say so if wanted.

### Red first

`tests/gate-lib-selftest.sh` holds the lock for 3 s in another shell, starts a gate that sources gate-lib.sh, and asks for `$SECONDS` (at least 2) and `gate_elapsed` (0). 8003f052: FAIL ("gate_elapsed counts the wait for the gate lock: shell time 3 s, gate_elapsed none s"); 5197fa22: `ok: gate-lib-selftest`. The step `gate-lib` of the merge gate runs it.

### Merge gate on 5197fa22

`tests/merge-gate.sh --jobs 3`, with two looping `cargo build -p chora-kit --tests -j1` jobs beside it (own target dirs), 4 CPUs. Printed matrix script time: 223 s, 237 s, 232 s (cap 315 s).
- run 1: layout FAIL (construct HUD layout audit, dark-1280x720: no third terminal in the walk, hint bar, window list; the known construct-hud flake under jobs 3 of matrix-cap r1). Matrix step PASS.
- run 2: doors FAIL ("2 cells or checks are not pass": the helper-key and helper-door cells of the tiling column "missing"). Matrix step PASS.
- run 3: all 41 steps PASS, exit 0. Matrix 232 s; the merge-gate step lines read 316 s, because the step also runs the other gates of its group.
Neither red is in a file of this change. The step time of the matrix line is not the matrix script's time; read the `ok: gate-handover-matrix ... in Ns of at most 315s` line for it.

### Review m2

Closed by this change: the cap measures the matrix's own wall time from the moment it holds the gate lock, the queue is not in it, and the PASS line says which time and which cap. m1 (silent fall-through of the two polls) is not touched.

## mx1-m2 r1 PM review (2026-11-19 20:26 UTC)
Diff f2b7112a..5197fa22, 3 files, +39 -7; builder sonnet lane, reviewer PM. Red commit 8003f052 first; identity ok.
- OK: gate_t0 set right after gate_lock; matrix cap, PASS line and startup phase use gate_elapsed; breach prints five slowest cells.
- MINOR m1: gate-lib-selftest wants gate_elapsed == 0 after the queued start; SECONDS is whole seconds, so a second boundary between gate_t0 and the echo reads 1 (rare flake). Fix: accept <= 1. Backlog.
Verdict (computed): no BLOCK or MAJOR open -> MERGE-OK; 5197fa22 to train 72.
