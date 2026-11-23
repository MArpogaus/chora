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
