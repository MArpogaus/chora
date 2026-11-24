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

## Review r1

Reviewer lane (sonnet, high), read-only on code. Reviewed claude/guard-4 at 9aa6a556 against cae62f50 (9 files, +215 -13); the branch has since gained a merge f6554f13 of battle-only files (tests/battle/soak-core.sh, RESULTS.md), not reviewed. Step 0: fetch --unshallow and `cargo check --workspace` ok.

**Verdict: FIX** (1 MAJOR, 4 MINOR, 0 BLOCK).

| id | sev | where | finding | fix |
|---|---|---|---|---|
| rg4-1 | MAJOR | tests/gate-mc-scale.sh:48-52, tests/gate-lib.sh load_scaled_limit | The scaled bound no longer catches a 2x regression on an idle CPU. Idle, the probe varies 44..69 ms (15 runs; ratio up to 1.57), so the limit of settings.set floats 5222..7666 ms while the call takes 2.66..2.78 s (debug): 2x = 5.5 s passed in idle run 1 (limit 5652 ms) and failed only in run 2 (limit 5222 ms); the old fixed 5000 ms failed both. | Scale only by the part of the ratio above the idle noise (e.g. `ratio > 2` or best of 5 probes at start and the median of the two around a call), so that an idle host keeps exactly 5000 ms. |
| rg4-2 | MINOR | tests/gate-lib.sh load_scaled_limit | No upper cap: limit = 5000 x worst/best grows with the probe (9042 ms for a 396 ms call beside a cargo build; a probe 9x slower gives 45 s). Past 10 s the bound is moot, since the core answers a call at most after TIMEOUT 10 s (crates/chora/src/methods.rs:29) and mc then fails. | Cap the limit at 9000 ms (below the core's 10 s) and say why in the function header. |
| rg4-3 | MINOR | tools/sleep-ratchet:50-56 | `--update` writes the tree count even when it is above the list: `echo 'sleep 1' >> tests/gate-leak.sh; tools/sleep-ratchet --update; tools/sleep-ratchet` gives list 780, `ok`. "Only falls" then rests on the commit-msg hook alone, which neither CI nor merge-gate runs (pre-commit stage commit-msg; CI runs ratchet-raise-selftest only). Same shape as lint-ratchet, so not new, but the brief asks that --update cannot raise. | Make `--update` exit 1 with "raise by hand with a ratchet-raise: reason" when the count is above the list. |
| rg4-4 | MINOR | tests/ratchet-raise-selftest.sh:22-25 (header :3) | The hook now guards tools/sleep-ratchet.list (tests/ratchet-raise.sh:8) but the selftest has cases for tools/lint-ratchet.list only, and its header names lint and size only; a typo in the new list path would pass unseen. | Run the cases for the sleep list as well (loop over the three lists) and name it in the header. |
| rg4-5 | MINOR | tools/sleep-ratchet:6-9, 31-37 | Blind spots of the count (audit command kept as is): 14 `time.sleep(` lines in embedded python of tests/*.sh (gate-0.8-1.sh:50,66,69; gate-0.8-3.sh:103; gate-0.8-6/9/10; gate-tiling-bar-early-signal.sh:30,36; 3d-bench.sh:140,162) and every sleep in subdirectories (tests/battle, tests/doors, tests/persona, tests/acts, demo/*.sh: about 150 lines) are not counted, so a new wait there never trips the ratchet. The header says "gate scripts" without naming this scope. | Count `sleep[ (]` and add tests/*/*.sh (a second list line `sleep-sub N` if the audit number must stay 779), or state the scope in the header. |

Checks (command, result):
1) Red first: `bash tests/sleep-ratchet-selftest.sh` at 97928c9b: rc 13 (tool missing, 4 FAIL lines); at 427b33fa and 9aa6a556: `ok: sleep-ratchet-selftest`. `bash tests/gate-lib-selftest.sh` at 6b2dc70c: rc 1 (3 FAIL: load_scaled_limit command not found); at 9aa6a556: `ok: gate-lib-selftest` (rc 0, 56 s). Both reds are missing-command reds (weak but as the brief allows). a9e8e887 is a third step the lane reports as red in the full gate (9128 ms); not re-run.
2) Ratchet: count on the head 779 (`grep -h -E '(^|[^a-z_])sleep ' tests/*.sh | wc -l`), equals the list; the diff adds no matching line to tests/*.sh (the only added matches are in tools/sleep-ratchet and ci.yaml). A planted `sleep 1` in a copy fails with rc 1: `sleep: list 779, tree 780; most in: tests/gate-mc-paths.sh (44), tests/gate-construct-cube.sh (40), ...` (list value, tree value, top 5 files). A fall prints "lower the list" and passes, so a valid commit is never blocked. Wired in ci-check (merge-gate.sh:255), ci.yaml:81-82 and ratchet-raise.sh:8. The "two halves" fixture and comment wording hide nothing: no real sleep line is dropped (found only rg4-5 blind spots; no sleep without a trailing space in tests/*.sh outside strings).
3) mc-scale, head, debug build, 4 CPUs (times of the cell; calls scene.save / scene.show / settings.set theme / small show / show again / settings.set write, limit in brackets):
   - idle 1: 19.7 s; 3557 (5000), 1909 (5000), 2776 (5652), 558 (6739), 1878 (6413), 174 (5434) ms.
   - idle 2: 16.5 s; 3603 (5000), 1894 (5000), 2658 (5222), 749 (7666), 1862 (5111), 149 (5111) ms.
   - beside `cargo build --workspace` (separate target dir, load average 3.1-3.6): 17.8 s; 3672 (5000), 2220 (5000), 3221 (5000), 592 (6607), 2135 (5000), 396 (9042) ms. Under this load the calls slowed by 20-40% and passed at the floor; the lane's 9316 ms did not reproduce (as known).
   - Probe `cpu_probe_ms` (python3 -I, sum of 2e6): 44..69 ms idle, about 0.1 s per call pair, cheap; jitter is the problem (rg4-1), the missing cap rg4-2.
4) No new sleep line, no changed timeout (only the five bounds replaced by the probe bound), 9 files, all lane files; no foreign file touched.

## r2

Branch claude/guard-4, head 284ce9a5 (merged origin/claude/autonomous-dev-session-3w-i33pg7 ce866b0d first, merge 68947079).

| commit | what |
|---|---|
| c9e2e851 | red: gate-lib-selftest cases (idle 69/44 must keep 5000 and fail a 5.5 s call; 600/200 gives 7500; 4000/100 capped at 9000) |
| 9b9e0c00 | fix rg4-1, rg4-2: only the slowdown above a probe ratio of 2 scales the bound; cap 9000 ms (core TIMEOUT 10 s, crates/chora/src/methods.rs:29, named in the function header) |
| c99befaf | red: sleep-ratchet-selftest cases (--update must not raise; sleep-all counts a subdirectory and time.sleep) |
| 284ce9a5 | fix rg4-3, rg4-5: --update fails above the list ("raise by hand with a ratchet-raise"); second list line `sleep-all`; also rg4-4: ratchet-raise-selftest loops the size and sleep lists (in the same commit, no separate red: the cases passed at once) |

1) rg4-1: done. Idle numbers of the review (probe 44..69) give exactly 5000.
2) rg4-2: done.
3) rg4-3: done.
4) rg4-4: done; cases for tools/size-ratchet.list and tools/sleep-ratchet.list (raised without line fails, with line passes, lower passes), header names the sleep list. Not a red-first item (it only adds cases).
5) rg4-5: `sleep` stays 779 (audit r6 command, the RC value). `sleep-all` = 930 today: lines matching `(^|[^a-z_])sleep |time\.sleep\(` in tests/*.sh, tests/*/*.sh, tests/*/*/*.sh. Same falling-only rule. `tools/sleep-ratchet` prints `ok: sleep-ratchet (sleep 779, sleep-all 930)`. The commit carries a ratchet-raise line for the new list line.
6) f6554f13 is a merge of 05a63377 (the session branch's merge of claude/soak-r4: tests/battle/soak-core.sh and tests/battle/RESULTS.md, +73 -2). It was put on the branch as a battle-only merge to bring the session head's state; both files are outside this lane's area but touched by this lane in no way, and are now part of the session head anyway (the r2 merge of ce866b0d already contains them). Not reverted: a revert would diverge from the session head and conflict at merge.

Checks:
- `bash tests/gate-lib-selftest.sh`: red at c9e2e851 (4 FAIL), ok at 9b9e0c00.
- `bash tests/sleep-ratchet-selftest.sh`: red at c99befaf (6 FAIL), ok at 284ce9a5. `bash tests/ratchet-raise-selftest.sh`: ok.
- `bash tests/gate-mc-scale.sh` idle, twice (debug build): run 1 calls 2851 / 1247 / 2392 / 512 / 1559 / 176 ms, run 2 2810 / 1323 / 2505 / 458 / 1307 / 356 ms; all bounds 5000.
- Beside `cargo build --workspace` (separate target dir, load average 3.9): 2847 / 1662 / 2414 / 520 / 1764 / 136 ms, all bounds 5000, pass (the calls slowed too little to need more).
- Full gate `tests/merge-gate.sh --jobs 2` on 284ce9a5: all steps passed, rc 0, no reds, none foreign.
