# core-watchdog r2 (claude/core-watchdog, 8313f011)

Merged claude/autonomous-dev-session-3w-i33pg7 first (ec26cf6c). Class invariant: the excused wait of a frame is the longest wait of one thread, so never more than the frame's wall time.

| Item | Result | Earliest gate |
|---|---|---|
| 1 red test | e9db01f2 `parallel waits of a frame do not excuse a costly scene`; red: 21 passed, 1 failed (Tasks::frame(0, 10) gave no Blame) | cargo test -p chora --lib safety |
| 2 fix | 14a8372b: FrameClock keeps a wait per thread id, excuses the maximum delta (a thread that ends or cannot be read is left out, stricter). safety 22/22 green, clippy -D warnings clean, fmt clean | the same unit test, now green |
| 3 agent.sh | busy added to the grep; host_pixel fails with a message when grim fails (both in 3b883e0b, see deviation) | not run alone; e2e step of the merge gate PASS (271 s) |
| 4 audit | NOT done, export kept (8313f011). gate-watchdog-starved PASS once (54 s) | see below |

## Item 4 numbers
Default budget, 2560x1600@2 (both themes), under 8 busy loops on 4 cores, 3 runs:
run 1 PASS (374 s), run 2 FAIL (386 s), run 3 PASS (380 s).
Run 2 failed 3 desktop/question cells (tile border, glow, shadow = the wallpaper): the core log shows two "no client alone reaches the owner budget; the watchdog drops all effects load=265" lines (12:24:30 and 12:27:46); the busy notes beside them show took 33-58 ms with waited 20-42 ms, so about 13-16 ms of the frame stays as cost at a budget near 16.7 ms. A genuine slowdown of llvmpipe at this size, not an excused wait. Per the brief: no new tolerance, export and header lines restored in 8313f011 (ci). Hand back: decision for the PM (keep the 500 ms export, or give the audit cell a less loaded host).
Without load the audit passed in the merge gate (tiling-audit PASS 321 s, default export in place).

## Merge gate
One full run stopped at doors (FAIL, mc column: screenshot-door and volume-door "missing"; xp, tiling, construct pass). Not related to the watchdog; `--only doors` PASS on the rerun (330 s), then `--only last-words,cube,pull,winit,wm-pane,walk,keys,pixels,mc-paths,e2e` all PASS. Steps before doors all PASS in the full run (2d-guard needed git fetch --unshallow: the clone was shallow). So the gate passed in three runs, not one; doors is a flake (reproduced once, not on rerun), logged here as an observation.
Logs: /tmp/merge-gate.CJm2xh (full run), /tmp/merge-gate.6Ed104 (doors), /tmp/merge-gate.uAurzX (rest); the container may not keep them.

## Deviations
- The agent.sh edits and the audit edit went in one commit (3b883e0b, subject says audit): a rejected first commit left agent.sh staged. Pushed already; not rewritten.
- Not run: the PM's rule of three gate passes in a row (not asked in this brief).
- Open (backlog, as the brief says): per-frame /proc cost, the cfg(test) path.
