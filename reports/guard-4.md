# guard-4

## r1

Branch claude/guard-4 from cae62f50, head 9aa6a556. Rounds: one.

| commit | what |
|---|---|
| 97928c9b | red: tests/sleep-ratchet-selftest.sh (tool missing: exit 13, "not found") |
| 427b33fa | tools/sleep-ratchet + list `sleep 779`; wired in ci-check after lint-ratchet, CI, ratchet-raise hook (ratchet-raise line: new list) |
| 6b2dc70c | red: gate-lib-selftest load_scaled_limit cases (command not found) |
| a9e8e887 | first fix: bound from the scene.save of the same cell; **red in the full gate** (scene.show 9128 ms, bound 5000: baseline taken once was too old) |
| 9aa6a556 | final fix: cpu_probe_ms before and after each call; bound = 5000 ms x slowdown of the probe vs its best, at least 5000 |

1) F4 W9: `tools/sleep-ratchet` prints `ok: sleep-ratchet (sleep 779)` on the head; the grep count is 779 (the selftest and the wiring add 0 lines: fixture lines are built from two halves, comments avoid the word). ci-check PASS (44 s) in the full gate.
2) sh-37: the 5000 ms was half the 10 s that a core method call waits. The load bound now follows a same-cell probe; the late-answer check and the core's 10 s wait keep guarding the real limit. The lane failure (9316 ms) did not reproduce here: cell 2.4 s with 80 busy loops or a parallel cargo build, so the red is the selftest with the observed numbers plus the full-gate red above (9128 ms). After the fix: under-load 10/10 (80 loops) and 6/6 (40 loops); mc-scale PASS 12 s in the full gate.
Full gate (tests/merge-gate.sh --jobs 2, no --only) on 9aa6a556: all steps passed, rc 0, no foreign reds. The first full run, on a9e8e887, was red in mc-scale only.
