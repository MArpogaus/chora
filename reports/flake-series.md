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
