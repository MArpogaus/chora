# flakes-n (F5: no flaky tests; the two unowned rows)

## r1

Lane: cloud, branch `claude/flakes-n` from `f5ac3500` (probe-leak, on session head `d81c8293`). Written 11-20.
Head: `fe398489`. Host: 4 CPUs, jq present, fuzzel 1.11.1 on PATH.

| Item | Result | Red, fix | Runs |
|---|---|---|---|
| 1. rw-3 `panel::the_layout_word_is_a_word_of_a_person` | not reproduced, shared state named | none | 0 of 20 loaded full `cargo test -p chora-tiling`; 0 of 40 `panel::` at 32 threads under load |
| 2. MX-CAP | cap 315 s -> 293 s by the rule of its comment | `da57c00d`, `fe398489` | 5 timed runs: 234, 232, 231, 229, 233 s |

Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,matrix` PASS
(pre-commit 5 s, gate-lib 76 s, ci-check 102 s, build 29 s, test 222 s, clippy 19 s, matrix 293 s; the
matrix cell's own run time 218 s of at most 293 s).

### 1. rw-3
Load: 20 runs of the full `cargo test -p chora-tiling` (176 tests, the layout test among them, jq present,
none skipped) with a `cargo build -p chora-kit` loop in a second target directory beside it (load average
4 to 6 on 4 CPUs). Reds: **0 of 20**. A second try aimed at the suspect below: 40 runs of the test binary
filtered to `panel::` with `RUST_TEST_THREADS=32`, same build loop: 0 of 40. Not reproduced, so no red-first
commit and no change to the test (a fix without a red would be a guess).

Shared state, read in `plugins/chora-tiling/src/main.rs` (`mod panel`, line 2062 on):
- No shared directory: `bin()` takes `TempDir::new`, which is `name-<pid>-<n>` with a process-wide counter,
  so the three `bin("layout")` calls of this test get three directories. Child environment is cleared
  (`env_clear`), so no env var is shared.
- The one shared thing is the process: every panel test writes a stand-in script (`fs::write`, then chmod)
  and runs it, while other test threads fork (`Command::output`, also `command -v` in `bin()`). A child
  forked between the open for write and the close of another thread's stand-in holds the write fd until
  its own exec; an exec of that stand-in in that window fails with "Text file busy" (the known Rust
  fork/exec race, rust-lang/rust#114554). `layout.sh` then reads no layout from `choractl`
  (`2>/dev/null`) and prints `""`, and the first assert fails with `"" != "Fair rows"`: the same
  shape as the single red of co-p2-fix rw-3 (one of 176, green alone). **Not shown**: no `ETXTBSY` seen in
  60 runs. If it comes back: take one process-wide lock around the write of a stand-in and around every
  spawn of the module; red-first by a test that spawns in a loop on one thread while another writes and
  runs a stand-in.

### 2. MX-CAP
Measure: `tests/merge-gate.sh --jobs 2 --only build,matrix,layout` five times (the matrix step beside
`layout`, as two steps at once at `--jobs 2`; not the cell alone), the script's own `run time:` of
`gate-handover-matrix.sh`: 234, 232, 231, 229, 233 s. Slowest 234 s, plus 25 % rounded up: 293 s.
Red `da57c00d`: `tests/matrix-cap-selftest.sh` reads the five runs of the comment and the cap of the
comment, of the usage line and of the code; at the base it said 315 s against 312 s for the old runs
(slowest 249 s). Fix `fe398489`: comment, usage line and default say 293 s, the gate-lib step runs the
selftest (`tests/merge-gate-selftest.sh` checks that), selftest ok.

Caution for the PM: 293 s is **below** the old 315 s. My runs had a lighter companion (`layout`, 203 s)
than a full train, and ran at load 1.2 to 3.3; the train that failed (t85b, 320 s) is not reproduced by
them. If a train with all its cells in parallel takes more than 293 s of the matrix's own run time, remeasure
under that set and move the three numbers together; the selftest holds them to the rule.

Other findings on the way:
- FLK-K1 (flake, F5, 11-20 ~14:38): gate-construct-k1 in `construct-extra` red once at 17 s: "the fly-out
  did not return to the pose before" ({505.06, 301.41, 299.23, 197.18} not {504.94, 301.32, 299.49, 197.35}),
  a camera pose off by up to 0.26 px; not seen again (the five matrix runs did not run it). Row for
  flakes-m.
- The gate does not stop the steps that run beside a failed one: after `construct-extra` failed, its
  sibling `matrix` kept running as an orphan (parent 1) and a queued gate took the lock after it. A row
  for the harness (GATE-ORPHAN).

## PM review r1 (2026-11-20 15:25 UTC)
Head fe398489 (red da57c00d). rw-3: not reproduced in 60 runs, cause named (ETXTBSY), accepted as evidence.
- pr1-1 MAJOR: MX-CAP 293 s is below the matrix's own run time in real trains on the PM host: `grep -hoE 'run time: [0-9]+' /tmp/merge-gate.*/matrix.log` over 86 logs gives slowest 283, 290, 300, 301, 309 s (trains of 11-20 04:38, 05:25, 11:23). 293 s would turn those trains red. Measure under a full train set, not beside `layout` alone.
Verdict: FIX (computed: one open MAJOR). r2: cap from the full-train figure (309 s + 25 % = 387 s, or a slower lane-measured full set), the three numbers and the selftest move together.

## r2

Lane: cloud, branch `claude/flakes-n` from `fe398489`. Written 11-20. Head: `d7c69aac`.

pr1-1 (cap from the full-train load): fixed. The five runs of the comment are the PM-host figures of real
trains (283, 290, 300, 301, 309 s); cap 309 s + 25 % = 387 s in comment, usage line and default; selftest ok.
Red `d861f34b` (selftest fails: cap 293 s, rule gives 387 s); fix `d7c69aac`.

Confirming run: `tests/merge-gate.sh --jobs 2 --only build,matrix,layout,construct-extra,doors` PASS
(layout 197, matrix 299, construct-extra 37, doors 359 s); matrix own run time **227 s**, below 387 s.

Gate: `--only pre-commit,ci-check,build,test,clippy,gate-lib,matrix` PASS (pre-commit 5, gate-lib 74,
ci-check 94, build 0, test 209, clippy 21, matrix 284 s; matrix own run time: 212s).
