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
