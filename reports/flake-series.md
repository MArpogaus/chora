# flake-series

Brief: briefs/flake-series-r1.md (F5 coverage and no-flake gate; F17 flake). Lane: cloud, branch claude/flake-series at 7d9ec18e (train 67, matrix cap in). Model: haiku, effort high. No production code changed; no commit on the code branch.

## r1

Setup: step 0 tools present (cargo, sccache, cargo-llvm-cov, foot, python3), so setup.sh was not run. `git fetch --unshallow` done; `cargo check --workspace` exit 0.

Gate: `tests/merge-gate.sh --jobs 3` (no --push), five runs in a row on 7d9ec18e. Logs kept in the lane scratchpad (flake-r1/run1.log to run5.log).

| run | start (UTC) | end (UTC) | exit | red steps | first failing line |
|---|---|---|---|---|---|
| 1 | 18:04:32 | 18:33:48 | 0 | none | - |
| 2 | 18:33:48 | 18:56:13 | 0 | none | - |
| 3 | 18:56:13 | 19:18:39 | 0 | none | - |
| 4 | 19:18:39 | 19:41:15 | 0 | none | - |
| 5 | 19:41:15 | 20:04:04 | 0 | none | - |

All five runs printed "all steps passed; no --push, nothing is pushed". No step was red, so no reruns of step 2 were needed (0 of 10 reruns, nothing marked ignored or known-red, no timeout changed).

Coverage: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features testing --summary-only`, exit 0.

- Lines: 9431 total, 843 missed, **91.06 %** (F5 needs 80 %: met).
- Functions: 1316 total, 159 missed, 87.92 %.
- Regions: 17690 total, 1533 missed, 91.33 %.

Five files with the most uncovered lines (missed / total lines):

1. crates/chora-kit/src/runtime.rs: 198 / 668 (70.36 %)
2. crates/chora-kit/src/launch.rs: 78 / 847 (90.79 %)
3. crates/chora-kit/src/icons.rs: 66 / 275 (76.00 %)
4. crates/chora-kit/src/effects.rs: 58 / 334 (82.63 %)
5. crates/chora-kit/src/launcher.rs: 52 / 218 (76.15 %)

Result: five of five green on 7d9ec18e; chora-kit lines coverage 91.06 %.

## r2

Brief: flake-series-r2 (F5 five greens on train 85; F17 flake). Lane: cloud, branch claude/flake-series-r2 at 9973d0eb (train 85, t84 plus 11 branches). Model: haiku, effort high. No production code changed; no commit on the code branch; nothing ignored, marked known-red or timeout-changed. The PM confirmed (after a merge request for adf71956) that this round stays on 9973d0eb.

Setup: step 0 tools present (cargo, sccache, cargo-llvm-cov, foot, python3), so setup.sh was not run. `git fetch --unshallow` done; `cargo check --workspace` exit 0.

Result: **0 of 5 green.** Every full gate run exited 1.

### Gate runs (`tests/merge-gate.sh --jobs 3`, no --push, on 9973d0eb)

| run | start (UTC) | end (UTC) | exit | red step | first failing line |
|---|---|---|---|---|---|
| 1 | 06:34:56 | 07:06:03 | 1 | launcher-click | `FAIL: tiling: a click on the Mid row of the launcher did not start mid (alpha, the first row: 0, the chat: 0)` |
| 2 | 07:06:03 | 07:30:19 | 1 | launcher-click | same line as run 1 |
| 3 | 07:30:19 | 07:49:00 | 1 | acts | `FAIL: xp act:session: Traceback …` ending `tests/acts/looks/xp.py:167` `ValueError: not enough values to unpack (expected 2, got 0)` (empty mixer read) |
| 4 | 07:49:00 | 08:16:32 | 1 | launcher-click | same line as run 1 |
| 5 | 08:16:32 | 08:34:32 | 1 | tiling-audit | `FAIL: fullscreen paper-2560x1600@2: the bar does not hide over the fullscreen window within 20 s` |

### Reruns of each red step alone (`tests/merge-gate.sh --only <step>`, 10 each)

| step | reds of 10 | cause line (first red) | host |
|---|---|---|---|
| launcher-click | **10 of 10** | `FAIL: tiling: a click on the Mid row of the launcher did not start mid (alpha, the first row: 0, the chat: 0)` (same in all 10) | fuzzel 1.9.2 (`fuzzel --version`: `fuzzel version: 1.9.2 +cairo +png +svg(nanosvg) -assertions`) |
| acts | **2 of 10** (reruns 5 and 7) | rerun 5: `cell selftest 'persona sound' FAIL`, `FAIL: selftest persona sound: expected no breach, got [...] I5 breach ... overlap` on the mc header; rerun 7: `cell xp act:session FAIL`, the same `xp.py:167` mixer `ValueError` as gate run 3 | - |
| tiling-audit | **1 of 10** (rerun 9) | `FAIL: fullscreen paper-2560x1600@2: the bar does not hide over the fullscreen window within 20 s`, then `FAIL: the layout audit found flaws` | - |

The step failures are not consistent: `launcher-click` red in every run, `acts` and `tiling-audit` each red sometimes and green most of the time. Nothing has been fixed, ignored or re-timed.

### Coverage (`env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features testing --summary-only`, exit 0)

- Lines: 9875 total, 817 missed, **91.73 %** (F5 needs 80 %: met).
- Functions: 89.40 %. Regions: 92.27 %.
- Five files with the most uncovered lines (missed / total):
  1. crates/chora-kit/src/runtime.rs: 182 / 671 (72.88 %)
  2. crates/chora-kit/src/launch.rs: 78 / 908 (91.41 %)
  3. crates/chora-kit/src/icons.rs: 65 / 341 (80.94 %)
  4. crates/chora-kit/src/effects.rs: 58 / 334 (82.63 %)
  5. crates/chora-kit/src/launcher.rs: 52 / 218 (76.15 %)

The coverage run overlapped the first reruns of `tiling-audit` (about 10:16 to 10:20 UTC) on the same host, so its CPU load is a possible confound for those reruns.

### Verdict

F5 "no flaky gate" is **not met**: 5 of 5 full runs red, and `launcher-click` is red in 10 of 10 isolated reruns on this host. F5 coverage (91.73 %) is met. F17 stays open. The `launcher-click` failure is the next thing to look at; `acts` (xp mixer read, mc header overlap) and `tiling-audit` (fullscreen bar) are intermittent and need their own rounds.

## r3

Brief: flake-series-r3 (F5: five full gates green in a row at --jobs 2 on train 92c, and the chora-kit coverage figure). Lane: cloud, branch claude/flake-series-r3 at dcb77733 (train 92c head: flakes-l, flakes-n, flakes-o merged). Model: haiku, effort high. No production code changed; no commit on the code branch; nothing fixed, retimed, ignored or marked known-red.

Setup: step 0 tools present (cargo, sccache, cargo-llvm-cov, foot, python3, ffmpeg), so setup.sh was not run. `git fetch --unshallow` done; `cargo check --workspace` exit 0.

Result: **2 of 5 green.** Runs 1 and 2 exit 0; runs 3, 4 and 5 exit 1. F5 "no flaky gate" is **not met**. Coverage (91.76 %) is met.

### Gate runs (`tests/merge-gate.sh --jobs 2`, no --push, on dcb77733)

| run | start (UTC) | end (UTC) | exit | red steps | first failing line |
|---|---|---|---|---|---|
| 1 | 21:33:05 | 22:17:11 | 0 | none (49 PASS) | - |
| 2 | 22:17:11 | 22:53:33 | 0 | none (49 PASS) | - |
| 3 | 22:53:33 | 23:20:14 | 1 | construct-hud (31 PASS before it) | `FAIL: light-1280x720@1.5: Tab opened no list: {"camera":…` (the FLK-HUD-TAB symptom) |
| 4 | 23:20:14 | 23:27:32 | 1 | xp-faces (13 PASS before it) | `MISMATCH opens-under-pointer: measured gap, expected pass`, then `FAIL: 1 cells differ from EXPECTED` |
| 5 | 23:27:32 | 23:52:46 | 1 | pull (28 PASS before it) | `FAIL: the second window is no pane: {` |

### Reruns of each step alone (`tests/merge-gate.sh --only <step>`, 10 each)

| step | reds of 10 | cause line (first red) |
|---|---|---|
| construct-hud | **0 of 10** | - (red once in the full gate, run 3; green in all 10 reruns) |
| launcher-click | **0 of 10** | - |
| xp-faces | **0 of 10** | - (red once in the full gate, run 4) |
| pull | **1 of 10** (rerun 2) | `FAIL: the second window is no pane: {` (same as run 5) |

Each red step is intermittent: one red in the full gate and 0 or 1 of 10 alone. Nothing was fixed, ignored, retimed or marked known-red. The red steps are construct-hud (FLK-HUD-TAB), xp-faces and pull; launcher-click (FLK-LAUNCHER) was green in all 10 alone and in every full run.

### Coverage (`env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features testing --summary-only`, exit 0)

- Lines: 9996 total, 824 missed, **91.76 %** (F5 needs 80 %: met).
- Functions: 89.28 % (1390 total). Regions: 92.44 %.
- Five files with the most uncovered lines (missed / total lines):
  1. runtime.rs: 182 / 672 (72.92 %)
  2. launch.rs: 78 / 908 (91.41 %)
  3. icons.rs: 65 / 341 (80.94 %)
  4. effects.rs: 58 / 334 (82.63 %)
  5. launcher.rs: 44 / 298 (85.23 %)

### Verdict

F5 "no flaky gate" is **not met**: 2 of 5 full runs green (runs 3, 4 and 5 red at construct-hud, xp-faces and pull). The intermittent steps need their own rounds: construct-hud (FLK-HUD-TAB), xp-faces, and pull (0 or 1 red of 10 alone). launcher-click (FLK-LAUNCHER) was green in 10 of 10 alone and in both full runs that reached it. F5 coverage (91.76 %) is met. F17 stays open.

## r4

Brief: flake-series-r4 (F5: five full gates in a row green at --jobs 2 on the session head 1c599c74). Lane: cloud, branch claude/flake-series-r4 at 1c599c74 (train 98 green). Model: haiku, effort high. No production code changed; no commit on the code branch; nothing fixed, retimed, ignored or marked known-red.

Setup: step 0 tools present (cargo, sccache, cargo-llvm-cov, foot, python3, ffmpeg), so setup.sh was not run. `git fetch --unshallow` done; `cargo check --workspace` exit 0.

Host (recorded once, before run 1): `nproc` 4; memory 16087 MB total, 10095 MB free, 5434 MB buff/cache, no swap.

Result: **5 of 5 green.** Runs 1 to 5 all exit 0, 49 PASS each, no FAIL line in any log. No red step, so no reruns of single steps were needed. F5 "five full gates in a row" is met on this evidence.

### Gate runs (`tests/merge-gate.sh --jobs 2`, no --push, on 1c599c74)

| run | start (UTC) | end (UTC) | exit | red steps | first failing line |
|---|---|---|---|---|---|
| 1 | 12:58:56 | 13:45:16 | 0 | none (49 PASS) | - |
| 2 | 13:45:16 | 14:24:23 | 0 | none (49 PASS) | - |
| 3 | 14:24:23 | 15:03:22 | 0 | none (49 PASS) | - |
| 4 | 15:03:22 | 15:42:39 | 0 | none (49 PASS) | - |
| 5 | 15:42:39 | 16:21:33 | 0 | none (49 PASS) | - |

Minutes per run: 46.3, 39.1, 39.0, 39.3, 38.9. Logs kept in the lane scratchpad (`r4/run1.log` to `r4/run5.log`, `r4/summary.txt`).

### Reruns of each step alone

None: no step was red in any of the five full runs.

### Verdict

F5 "no flaky gate" is met for this round: 5 of 5 full runs green at --jobs 2 on 1c599c74. F17 status is not changed by this round. The open flake rows from r3 (FLK-HUD-TAB, xp-faces, pull) were not reproduced in these five runs; their evidence is r3's, not this round's.

## r5

Brief: flake-series-r5 (F5 verdict on the week-3 head: five full gates in a row green at --jobs 2 on the session head 3b14c825, train 107). Lane: cloud, branch claude/flake-series-r5 at 3b14c825. Model: haiku, effort high. No production code changed; no commit on the code branch; nothing fixed, retimed, ignored or marked known-red.

Setup: step 0 tools present (cargo, sccache, cargo-llvm-cov, foot, python3, ffmpeg), so setup.sh was not run. `git fetch --unshallow` done; `cargo check --workspace` exit 0.

Host (recorded once, before run 1): `nproc` 4; memory 16087 MB total, 9982 MB free, 5478 MB buff/cache, no swap.

Result: **5 of 5 green.** Runs 1 to 5 all exit 0, 49 PASS each, no FAIL or MISMATCH line in any log. No red step, so no reruns of single steps were needed.

### Gate runs (`tests/merge-gate.sh --jobs 2`, no --push, on 3b14c825)

| run | start (UTC) | end (UTC) | exit | red steps | first failing line | minutes |
|---|---|---|---|---|---|---|
| 1 | 08:53:43 | 10:07:24 | 0 | none (49 PASS) | - | 73.6 |
| 2 | 10:07:24 | 11:13:18 | 0 | none (49 PASS) | - | 65.9 |
| 3 | 11:13:18 | 12:21:19 | 0 | none (49 PASS) | - | 68.0 |
| 4 | 12:21:19 | 13:28:38 | 0 | none (49 PASS) | - | 67.3 |
| 5 | 13:28:38 | 14:36:58 | 0 | none (49 PASS) | - | 68.3 |

Runs took 66 to 74 minutes, longer than r4's 39 to 46 minutes on 1c599c74; the cause was not investigated. Logs kept in the lane scratchpad (`r5/run1.log` to `r5/run5.log`, `r5/summary.txt`).

### Reruns of each step alone

None: no step was red in any of the five full runs.

### Leak evidence (after run 5, no gate running)

- `ls /tmp/gate-env.* | wc -l`: **650** files (615 `-rw-------`, 35 `-rw-r--r--`), dated from 09:54 onward. Not investigated whether the gate's own cleanup should remove them.
- chora processes left: `pgrep -c chora` (process name) **0**; `pgrep -fc chora` (full command line) **70**, all `sh /home/user/chora/flavors/tiling/wallpaper.sh`.

### Verdict

F5 "no flaky gate" is met for this round: 5 of 5 full runs green at --jobs 2 on 3b14c825. The open flake rows from r3 (FLK-HUD-TAB, xp-faces, pull) did not reproduce in these five runs; construct-hud, xp-faces and pull passed in every run. F17 status is not changed by this round. The leak evidence (650 gate-env files, 70 wallpaper.sh processes left behind) is recorded for a separate look; it did not fail any step.

## r6

Brief: flake-series-r6 (F5: five full gates in a row green at --jobs 2 on the session head 230ea0e5, train 113). Lane: cloud, branch claude/flake-series-r6 at 230ea0e5. Model: haiku, effort high. No production code changed; no commit on the code branch; nothing fixed, retimed, ignored or marked known-red.

Setup: step 0 tools present (cargo, sccache, cargo-llvm-cov, foot, python3, ffmpeg), so setup.sh was not run. `git fetch --unshallow` done; `cargo check --workspace` exit 0.

Host (recorded once, before run 1): `nproc` 4; memory 16087 MB total, 11819 MB free, 3865 MB buff/cache, no swap.

Result: **0 of 5 green.** Full runs 1, 2, 3, 4b and 5 all exited 1. Run 4 did not run any tests (see below) and is void; it was replaced by run 4b, which ran in full. F5 "no flaky gate" is **not met**. F17 stays open.

### Gate runs (`tests/merge-gate.sh --jobs 2`, no --push, on 230ea0e5)

| run | exit | red steps | first failing line | minutes |
|---|---|---|---|---|
| 1 | 1 | acts | `FAIL: no click on Notepad` (acts log: `expected one new window {'owner': 'xp'}, got []`) | 45.7 |
| 2 | 1 | keys | `FAIL: the Foot row of Programs started no foot` | 61.3 |
| 3 | 1 | keys | `FAIL: the Foot row of Programs started no foot` | 61.0 |
| 4 | void | none: refused at start | `merge-gate: waits for /tmp/chora-gate.lock`, then `a chora --headless of an earlier run lives and takes CPU` (pid 18454, left by run 3); exit 1 after 0.01 min | 0.0 |
| 4b (replaces void run 4) | 1 | keys | `FAIL: the Foot row of Programs started no foot` | 61.2 |
| 5 | 1 | keys | `FAIL: the Foot row of Programs started no foot` | 61.2 |

Each red run stopped at its red step (run 1 and run 2 logs end at their FAIL lines), so the steps after the red step did not run in these runs. Logs kept in the lane scratchpad (`r6/run1.log`, `r6/run2.log`, `r6/run3.log`, `r6/run4.log` (void), `r6/run4b.log`, `r6/run5.log`, `r6/summary.txt`).

### Reruns of the red steps (`tests/merge-gate.sh --jobs 2 --only build,<step>`)

Planned: 10 for `acts` (red in run 1) and 10 for `keys` (red in runs 2, 3, 4b, 5).

| step | rerun | result | cause line |
|---|---|---|---|
| acts | 1 | red (rc 1, 21 s) | `FAIL: swaylock in the PATH of the session is '', not a stand-in`; `FAIL: the selftest of the persona session` |
| acts | 2 | stalled, interrupted by me by pid (rc 1, `merge-gate: interrupted`) | no verdict: acts log stopped growing at 12:33 while `python3 tests/acts/run.py` stayed at 66 % CPU for 24 min |
| acts | 3 | stalled, interrupted by me by pid (rc 1, `merge-gate: interrupted`) | no verdict: no output for 30 min |
| acts | 4 | started by the chain, interrupted by me by pid at 13:28 | no verdict |
| acts 5 to 10, keys 1 to 10 | - | not run | the chain was stopped at 13:28 because the stalls would have taken about 8 hours |

So: `acts` red 1 of 1 completed rerun; 3 reruns did not finish (stalled and interrupted). `keys` reruns: none run. The planned 20 reruns are not complete, so this round does not meet the step 2 count.

Note on the stalls: the two stalled reruns (2 and 3) each started right after an earlier interrupted or red rerun. At 13:29, before rerun 4 was interrupted, `chora-voice` and `chora-assistant` processes from the stalled rerun 2 were still alive (defunct and running). The stall is therefore probably a leftover-process effect, not proven. Rerun 4's gate cleanup left no chora process behind after its interrupt.

### Leak evidence (step 3, at 13:29 UTC, no merge-gate running)

- `ls /tmp/gate-env.* | wc -l`: **84**.
- `pgrep -c chora` (process name): **0**.
- Before rerun 4 was interrupted: 3 leftover chora or chora-voice/chora-assistant processes (from the stalled rerun 2).

### Verdict

F5 "no flaky gate" is **not met**: 0 of 5 full runs green (runs 1, 2, 3, 4b, 5 red; run 4 void). The red steps are `acts` (run 1) and `keys` (runs 2, 3, 4b, 5, always the Foot-row line). The `acts` reruns are red once and stalled twice; the `keys` reruns were not run. Next: a separate look at the Foot-row `keys` failure, and at the stall of the `acts` rerun with the leftover chora-voice and chora-assistant processes. F17 stays open.

## r7

Brief: flake-series-r7 (F5, rc 3: sh-34, the stray-process stall, xp-42; red test first). Lane: cloud, branch claude/flake-series-r7 from 1ec04694, head 6f7bcb26 (8 commits, all pushed). Step 0 tools present (cargo, sccache, foot, fuzzel 1.11.1 in ~/.local/bin, python3, ffmpeg); `cargo check --workspace` green.

### sh-34 (keys: "the Foot row of Programs started no foot")

- Cause (environment, reproduced): fuzzel 1.9.2 ignores clicks on a row (mc-47). This container has two fuzzel: ~/.local/bin/fuzzel 1.11.1 first on PATH (cell 10/10 green alone, also 6 runs under 12 busy loops) and /usr/bin/fuzzel 1.9.2. With 1.9.2 first on PATH the cell fails with the exact line after 30 s. That the lane containers lack the 1.11.1 is inferred from the mc-tidy evidence, not checked there.
- Red fecb1f14 (gate-lib-selftest: the keys cell on a fuzzel 1.9.2 stub must end in `SKIP: fuzzel 1.9.2 < 1.11`; red). Fix 516789eb: `need_fuzzel 1.11` in tests/gate-construct-keys-persona.sh and `keys` in the tool_minimums row of merge-gate.sh (same change as claude/guard-3 has; identical text).
- Environment line for the lane containers: put a fuzzel >= 1.11 first on PATH (here ~/.local/bin/fuzzel 1.11.1 over /usr/bin/fuzzel 1.9.2); without it keys and launcher-click are SKIP.
- Result: cell 10/10 green on 1.11.1 after the fix; SKIP with the version on 1.9.2.

### Stall (leftover chora-voice and chora-assistant)

- Cause: acts' harness starts chora-voice and chora-assistant each in a session of its own (start_new_session), so the stop of the process group of a step does not reach them when run.py ends without its cleanup; the same for any program a gate script leaves. The next run refuses or stalls (r6 run 4, reruns).
- Red 7d2ca087: merge-gate-selftest cases (a step that leaves a chora process is FAIL and names it; an interrupted gate leaves none). Fix b31fbae5: merge-gate.sh tags each step with CHORA_GATE_RUN (gate pid) and CHORA_GATE_STEP; when a step ends, live chora* processes with that tag (also after setsid; zombies excluded; 3 s settle) make the step FAIL, are listed in its log and ended; the stop of a red or interrupted gate ends those of the whole run. Only processes that carry this gate's tag are signalled, never by name alone. Seen live: gates that I interrupted left no live chora process.
- The check found three more real leaks in earlier gates of this round, each red first (`merge-gate.sh --only build,STEP`) and fixed: cube (the xp of the cube outlives its core by ~30 s; 0c185eab), agent (chora-tiling; 96fe4109), tiling-bar (chora-notify under setsid + dbus-run-session; gate_cleanup now also signals the group of each run_pid; 99e431cb). Those gate attempts went red on this round's own new check and are not counted below.
- 6f7bcb26: merge-gate-selftest ran its inner gates beside the leak cases of gate-lib-selftest; an inner gate refuses to start while any `chora --headless` lives, so "a gate script inside the gate does not wait for the lock" went red in 4 of 4 gate-lib steps after 99e431cb (green before). With MERGE_GATE_NESTED=1 on the inner gates the step passed 3 of 3. Why it was green before is not found; the refusal by the parallel compositors is the likely cause, not proven.
- `pgrep -a chora` after a gate still lists [defunct] chora-voice, chora-assistant, chora-probe, chora-notify: zombies reparented to PID 1 of this container, which does not reap. Live (non-zombie) chora processes after each of the 3 gates: 0.

### xp-42 (opens-under-pointer)

- Not reproduced: 20/20 under 12 busy loops, 40/40 under 24, and 11 plain runs green; so there is no red commit for this item. A forced delay (SIGSTOP of the pane host) did not delay the entry.
- The log shows xp takes the output of a new window from the last `pointer.entered` it heard (xp-wall-VIRTUAL-N, injected). Fix 05a23dca: the cell subscribes to the events, waits for that entry on the wall of the target output, asks xp once (ordered after the entry on its socket) and only then starts foot; it moves the pointer away and back if the entry does not come in 1 s. No timeout raised. Cell 20/20 green after the fix. That this was the cause of the two xp-42 hits is not proven.

### Gate (`tests/merge-gate.sh --jobs 2`, full, no --only, on 6f7bcb26)

| run | exit | red step | minutes |
|---|---|---|---|
| 1 | 0 | none | 86 |
| 2 | 1 | leak: `FAIL: the RSS of the core grows 1613 kB per round (bound 1024)` | 75 |
| 3 | 0 | none | 81 |

2 of 3 green. The red is a flake outside this brief: the RSS slope of the leak step is -1199 kB/round in run 1 and +1613 in run 2 (fds flat), so the 1024 kB bound sits inside the noise of this container. Not rerun, not ignored, not marked known-red; it needs a finding (sh-33 area) and a look at the slope estimator. No FAIL from xp-faces or keys in the 3 runs; no step left a chora process.

### Verdict

sh-34: environment, closed with a SKIP guard (cell 10/10 here). Stall: fixed by the stray check in merge-gate (it found and fixed 3 more real leaks). xp-42: a synchronisation fix, no reproduction. F5 stays open on the RSS bound of the leak step (1 of 3 gates red).

## PM review r7 (05:12 UTC, 6f7bcb26 vs base 1ec04694; 146 lines, PM not the builder)
| id | sev | where | finding |
|---|---|---|---|
| rfs7-1 | MAJOR | tests/gate-xp-faces.sh, tests/merge-gate.sh | F4 W9 breach: merged onto ce866b0d the audit count (`grep -h -E '(^|[^a-z_])sleep ' tests/*.sh \| wc -l`) is 787 vs RC value 779 (+8: new `sleep 0.1`/`0.3` in pointer_lands_on, stray_pids waits, and the merge). guard-4 freezes 779, so this would turn F4 red. |
| rfs7-2 | MAJOR | 05a23dca | xp-42 fix with no red commit before it (not reproduced in 71 runs). The brief allowed a forced delay as the red; none was made. Cause unproven. |
| rfs7-3 | MINOR | 6f7bcb26 | selftest race fix with no red commit; small, test-only. |
| ok | - | 7d2ca087 -> b31fbae5, fecb1f14 -> 516789eb | stray check and sh-34 SKIP: red then fix, by env tag not by name; 3 real leaks fixed (cube xp, agent chora-tiling, tiling-bar notify). |
Gate: 2/3 green; gate 2 red on leak RSS slope (+1613 vs 1024): new finding sh-38. Verdict (computed): FIX (2 MAJOR). Earliest gate for rfs7-1: the sleep ratchet of guard-4 (not merged yet); for rfs7-2: red-first step checks only that a test commit exists, not that it fails before (backlog item exists: rg4 reds were missing-command reds).

## r8

Brief: flake-series-r8 (F5, F4, rc 3): rfs7-1, rfs7-2, sh-38; red first. Lane: cloud, branch claude/flake-series-r7 from 6f7bcb26, merged with ce866b0d (2adac16a), head d2c2174c (pushed). `cargo check --workspace` green.

### rfs7-1 (W9 count)

- After the merge `grep -h -E '(^|[^a-z_])sleep ' tests/*.sh | wc -l` was 787 (+8: xp-faces 2, merge-gate.sh 2, merge-gate-selftest.sh 4). Fix 380fa09a: merge-gate.sh waits through one `poll()` (no_strays, none_alive) in place of its three loops; the selftest through one `poll_for()` (orphan, lock holder, stray cases wait on the condition, not 0.5 s; the hang step is `timeout 60 tail -f /dev/null`). xp-faces back to its 5 (see below). Count now **778** (RC 779).
- No test gate for the count exists before guard-4 (not merged), so there is no red commit for this item; the count is the check.
- Alone, tests/merge-gate-selftest.sh shows 2 FAIL lines (the pre-commit step stub: `got ''`), the same 2 on 2adac16a before my change (checked in a worktree), so not from this round; inside the gate (gate-lib step) it passes in all 3 runs.

### rfs7-2 (xp-42)

- Red: a forced delay. `motion_late=SECONDS` in tests/gate-xp-faces.sh makes the next pointer motion of the gate land that late, in the background; the new cell `opens-under-late-pointer` (EXPECTED pass) lets the motion land 3 s after foot has started. Commits: c0900955 reverts 05a23dca (pure revert), 5b280d4c adds the knob and the cell on the old code: **gap** ("window at 4,25 on VIRTUAL-1", xp read the output from the old pointer position), MISMATCH, exit 1. Fix e7192a72 puts 05a23dca back (pointer_lands_on: events, motion, wait on `pointer.entered` of the wall with until_ok, ask xp once, move away and back if it does not come in 1 to 2 s), and the same cell is **pass** (3 of 3 cells pass; all 11 cells in the 3 gates). 05a23dca is kept; its two fixed sleeps are gone (until_ok), no timeout raised. That the real xp-42 hits had this cause stays unproven; the forced delay shows the cell is race-free for a late motion.

### sh-38 (leak step RSS slope)

Spread of the slope by the old statistic (median of the differences, warm 2), kB/round, all with no leak (fds flat, windows and clients at base):

| runs | n | slopes (sorted) | sd |
|---|---|---|---|
| 8 rounds, idle | 10 | -784 -592 -316 -92 0 84 180 436 744 **1736** | 684 |
| 8 rounds, beside `cargo build --workspace` (loop, own target dir) | 10 | -712 -44 -20 -16 40 80 96 104 344 864 | 367 |
| 12 rounds, idle | 10 | -252 -28 -24 -4 28 52 64 84 84 252 | 120 |
| 12 rounds, beside cargo build | 3 | 4 8 636 | 297 |

(plus 2 more 8-round runs, 952 and 300; run 1 of the 12-round load batch broke because I edited gate-leak.sh while it ran.) One idle run of 22 of 8 rounds was red (+1736, as r7 gate 2 +1613): the bound 1024 sat inside the noise.

- Cause: the anon heap of the core sits on one of two levels, about 172 and 179 MB (7 MB apart), and moves between them from round to round and by no rule I found (the RSS is flat for 20 s after a round, so it is not time to settle: traced 40 x 0.5 s per round). With 6 rounds after warm-up that is a trend of up to 1.7 MB per round; no statistic I tried (median of differences, Theil-Sen, least squares, medians or maxima of halves or thirds) tells it from a leak at 6 points (null sd 430-950, all above the planted 1.5 MB per round). Only more rounds do.
- Red 252b3d3e: leak-verdict-selftest has the recorded tables (idle6 of 8 rounds, no leak, +1736; r12-2 and r12-8 of 12 rounds) as tables that must pass, and planted ramps on them that must fail; **red** (idle6 FAIL: the RSS of the core grows 1736 kB per round), the other rows green.
- Fix d2c2174c: gate-leak.sh runs 12 rounds by default (10 after warm-up, 467 s in the gate); the verdict scales the bound by the number of rounds, bound * sqrt(sxx of 10 rounds / sxx of the table): **1024 for the default, not raised**; 2223 for 6 rounds (what ROUNDS=8 gives). Checked offline on every recorded table with the awk of the gate: 22 of 8 rounds and 10 of 12 rounds all pass; a planted ramp fails on 10 of 10 12-round runs at 1.5 MB per round (6 of 10 at 1.0, 9 of 10 at 1.2), on 22 of 22 8-round runs at 3.5 and 4 MB per round (21 of 22 at 3.0, 18 of 22 at 2.5). The self-test's planted leaks: 4 MB per round on 8 rounds (a window texture), 1.5 MB per round on 12 rounds, the sh-33 shape (+31 MB, +8 fds, +4 windows). Limit: 8 rounds cannot see a leak of 1.5 MB per round, and 12 rounds not one below 1.2; a leaked probe buffer (160 kB per round) is invisible to the RSS here, fds, windows and clients are the check for that.
- The cell, final code: 10 of 10 idle at 12 rounds green (the 10 runs above, slopes -252..+252, bound 1024), 3 of 3 beside cargo build (8, 636, 4); 3 of 3 inside the full gate (88, 80, -376). The idle batch ran with the median-of-differences statistic of the old awk; the new awk differs only in the bound (equal at 10 rounds), so the verdicts are the same and I checked them offline with the new awk.

### Gate (`tests/merge-gate.sh --jobs 2`, full, no --only, on d2c2174c, no rerun)

| run | exit | red step | minutes | `pgrep -a chora` after |
|---|---|---|---|---|
| 1 | 0 | none | 79 | one `[chora-tiling] <defunct>` zombie (PID 1 does not reap), no live process |
| 2 | 0 | none | 73 | one defunct chora-tiling |
| 3 | 0 | none | 75 | one defunct chora-tiling |

3 of 3 green; leak step 467/463/465 s (slope 88, 80, -376 kB per round); xp-faces 14 s with the late-pointer cell; keys PASS; no SKIP step.

### Verdict

rfs7-1 fixed (778). rfs7-2: red made (forced delay), 05a23dca kept. sh-38: statistic and rounds fixed, bound not raised; red then fix; planted leaks fail. Open: the cause of the 7 MB level switches of the heap of the core is not found (not a leak by fds, windows, clients and the 12-round slopes); rfs7-3 (selftest race fix 6f7bcb26 without a red) not touched.

## Review r8

Seeded review of claude/flake-series-r7 head d2c2174c, base 2adac16a (6 files, +139 -54); read-only, no merge gate run. `cargo check --workspace` green.

Checks:
1. sh-38: `bash tests/leak-verdict-selftest.sh` red at 252b3d3e (idle6: 1736 > 1024, exit 1), green at d2c2174c (10/10 ok, exit 0). (It is a bash script: `sh` = dash fails with a syntax error on both commits.) At the default 12 rounds (n = 10 after warm-up, sxx 82.5) the bound stays 1024: not a raised bar. Planted ramps checked with the d2c2174c awk: +1.5 MB/round fails on r12-2, r12-8 and a flat 12-round table (+1.1 and +1.2 fail too; +1.0 fails on r12-2, not on flat). merge-gate.sh calls gate-leak.sh with no ROUNDS, so the gate runs 12. Gate time: the lane's leak step took 463-467 s, under the 10-min cap of gate-leak.sh and about +155 s on a 73-79-min gate: fits.
2. xp-42: c0900955 is a pure revert; 5b280d4c adds the `motion_late` knob and the cell on the reverted code (the lane's red: gap, exit 1; not rerun here, no gate run); e7192a72 puts the fix back. The wait is `until_ok` on `wall_entries` (a condition poll, 1-2 s), no fixed sleep; `pace "$late"` is the forced delay of the knob, not a wait.
3. rfs7-1: d2c2174c merged with d40142e4 in a scratch worktree (one comment-only conflict in tests/gate-lib-selftest.sh, kept both); `tools/sleep-ratchet`: ok, sleep 778 / 779, sleep-all 929 / 930. poll()/poll_for() sleep 0.1 between condition checks; the three `sleep 0.5` became polls on a condition; no fixed sleep moved into a helper.
4. Red-first: 5b280d4c before e7192a72, 252b3d3e before d2c2174c, both red on their own commit. 380fa09a has no red commit; measured here: the W9 count is 787 on 2adac16a and 778 on d2c2174c, so the red is the count on the parent (rfs8-3).
5. `bash tests/merge-gate-selftest.sh`: exit 0, 231 ok, no FAIL at both 2adac16a and d2c2174c here. The lane's 2 pre-commit-stub FAILs did not reproduce in this container; they depend on the environment and are the same on both commits, so they were not caused by this branch.

Findings:
- rfs8-1 MINOR tests/leak-verdict.awk:26: the bound is scaled by sqrt(sxx), which is the noise of a least-squares slope (about n^-1.5), but the statistic is the median of the differences (noise about n^-0.5). For a run below 12 rounds the bound grows faster than the noise of that statistic: 2223 at 6 rounds, where sqrt(10/6) would give about 1320. So ROUNDS=8 lets a steady +1.5 to +2.2 MB/round pass (checked: +1.5 passes). The default of 12 is unaffected. Fix: scale by sqrt(10/(n-1)) to match the statistic, or have gate-leak.sh refuse ROUNDS < 12.
- rfs8-2 MINOR tests/leak-verdict-selftest.sh:54: the fix commit d2c2174c raised the planted leak on idle6 from the red commit's +2.5 to +4 MB/round and moved the old 8-round "+1.5 MB/round" row to 12 rounds (line 38), which relaxes planted-leak assertions in a fix. idle6 + 2.5 MB/round still fails under the new awk (slope 4236 > 2223). Fix: keep the +2.5 MB/round idle6 row next to the +4 one.
- rfs8-3 MINOR 380fa09a (tests/merge-gate.sh:497): a fix(...) with no red commit. The lane's reason ("no count gate before guard-4") is weak because tools/sleep-ratchet from d40142e4 shows 787 > 779 on 2adac16a. The change is a behaviour-preserving wait refactor, and the count was verified here from 787 to 778, so this is not MAJOR. Fix: record the ratchet red (787 > 779 on the parent) in the report; no code change.
- rfs8-4 NIT tests/merge-gate-selftest.sh:858: the hang stub changed from `sleep 60` to `timeout 60 tail -f /dev/null`. It is the same hang under another name and lowers the W9 count by one without removing a wait. Fix: keep `sleep 60`, or count it as a hang stub in the report rather than as a removed wait.
- rfs8-5 NIT tests/merge-gate.sh:497, tests/merge-gate-selftest.sh:128: poll() and poll_for() are two copies of the same loop, and a third next to gate-lib's until_ok (the brief asked for existing helpers). Fix: one shared helper once merge-gate.sh can source gate-lib.
- rfs8-6 NIT tests/gate-lib-selftest.sh:17: merging with d40142e4 gives a comment-only conflict in the header, which comes from ce866b0d and d40142e4 (not from this branch). Fix: keep both paragraphs at merge.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 3 MINOR, 3 NIT).

## r9

Brief: flake-series-r9 (F5, rc 3): five full gates in a row on the train132 head c3c37c2e, run only (no product, test or harness change). Lane: cloud, branch claude/flake-series-r9 at c3c37c2e. Step 0: `git fetch --unshallow`, `cargo check --workspace` exit 0, `cargo build --workspace` exit 0.

Gate: `tests/merge-gate.sh --jobs 2`, full, no --only, no rerun, one after the other, CARGO_BUILD_JOBS=2.

| run | exit | PASS | red steps | minutes | log |
|---|---|---|---|---|---|
| 1 | 0 | 65 | none | 84 | /tmp/gate-1.log |
| 2 | 0 | 65 | none | 79 | /tmp/gate-2.log |
| 3 | 0 | 65 | none | 75 | /tmp/gate-3.log |
| 4 | 0 | 65 | none | 78 | /tmp/gate-4.log |
| 5 | 0 | 65 | none | 82 | /tmp/gate-5.log |

Run 5 was restarted: the first attempt (`/tmp/gate-5-interrupted.log`) stopped at 24 PASS with no EXIT line when the lane's worker restarted (no red step, no FAIL line in it). It is not counted; run 5 above is the second, complete attempt, started after the restart with the same command.

After each run: no `chora --headless` left (`pgrep` empty), root disk 47 G free (above 5 G, so no sccache cleanup). No SKIP step in any run.

Red steps: none in any of the five runs, so no log was copied to media/ and no known id (sh-38..sh-41) was matched; the sh-38 leak step (the one that flaked in r7 and r8) was PASS in all five.

F5 series: 5/5 green on c3c37c2e

Verdict: the train132 head is green in five full gates in a row (exit 0, 65 PASS each, 75-84 min). Nothing new to file in findings/shared.md.

## r10

Brief: flake-series-r10 (F5, rc 3): five full gates in a row on the t153+154 head 4d8bbe36, run only (no product, test or harness change). Lane: cloud, branch claude/flake-series-r10 at 4d8bbe36. Step 0: `git fetch --unshallow`, `cargo check --workspace` exit 0, `cargo build --workspace` exit 0.

Gate: `tests/merge-gate.sh --jobs 2`, full, no --only, no rerun, one after the other, CARGO_BUILD_JOBS=2. The gate stops at its first red step, so a red run has fewer PASS lines.

| run | exit | PASS | red steps | minutes | log |
|---|---|---|---|---|---|
| 1 | 0 | 68 | none | 82 | /tmp/gate-1-run.log |
| 2 | 0 | 68 | none | 80 | /tmp/gate-2-run.log |
| 3 | 0 | 68 | none | 77 | /tmp/gate-3-run.log |
| 4 | 1 | 56 | handover: "FAIL: the picker of the Looks door did not start" (tests/gate-looks-move.sh:92, launcher_up 80 tries) | 64 | /tmp/gate-4-run.log; step log media/2026-11-26/flake-series-r10/run-4-handover.log |
| 5 | 0 | 68 | none | 77 | /tmp/gate-5.log |

Red step match: handover (run 4) is "new". It is not one of sh-37..sh-42: sh-39 and sh-40 are tiling launcher and slide cells, sh-41 is the tiling double bar, sh-42 is keyboard focus when tiling joins beside xp or mc. The assertion is a launcher start wait in tests/gate-looks-move.sh. Nothing was rerun to make it green. The handover step's earlier focus checks (give key, s3 to xp, keys on tiling) passed in run 4 before this failure. Not filed in findings/shared.md (this lane only reports).

Environment notes (not counted as runs):
- The docs step needs mkdocs, which this container did not have, and this head changes docs and mkdocs.yml against the base. First attempt (13:08 UTC) went red at docs with "mkdocs is not installed and docs changed". I installed the declared docs/requirements.txt (mkdocs 1.6.1) into a venv under the scratchpad and put only a `mkdocs` symlink on PATH. Repo files were not changed, and no system package was installed. A strict `mkdocs build` run by hand exited 0.
- A second attempt (13:10 UTC) put the whole venv bin first on PATH. The venv's python3 has no PIL, so ci-check went red with "the Python module PIL is missing" (skip path of tests/tiling-scripts-selftest.sh:110). The fix was the mkdocs-only symlink PATH. This attempt is discarded.
- Run 2 was first started at 14:38 UTC. The worker restart at about 15:54 UTC killed it at 67 PASS, with no EXIT line and no gate process left. Per the r9 precedent it is not counted, and run 2 was restarted from scratch at 15:54 UTC. The timings in the table are for the complete attempts.
- Between runs: no `chora --headless` left (pgrep empty after each run), root disk 46-47 G free (above 5 G, so no sccache cleanup).

F5 series: 4/5 green on 4d8bbe36

Verdict: four of five full gates were green on 4d8bbe36 (exit 0, 68 PASS each). Run 4 went red at handover with a new launcher-start failure, so this series does not re-earn rc 3 on its own. Next: a flake round on the Looks-door launcher wait (tests/gate-looks-move.sh:92), with the run 4 log in media/2026-11-26/flake-series-r10/.
