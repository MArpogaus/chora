## r1

Branch claude/core-l, from 355cdbe5 (train 84 head; origin/claude/autonomous-dev-session-3w-i33pg7 had not moved: already merged). Pushed at b1d1491d.

| Item | State on the head | Red | Fix | Cell |
|---|---|---|---|---|
| WD2 | open | 4d2b133b | f3baf36d | gate-watchdog-starved (run by hand: both checks ok), unit tests in safety.rs |
| L16 | open | b8b7adf0 | b1d1491d | unit test in chora-scene logs.rs |
| F22 | closed on the head | none | e5e44719 (already on the train) | crates/chora/tests/log_file.rs |

### WD2
- Cause: FrameClock read /proc/self/task at the start and at the stop of every frame.
- Numbers before: 59 us per read, 118 us per frame, 20 threads (throwaway bench of the same read_dir + schedstat code on a 20-thread process). Calm test: 200 reads in 100 frames.
- Fix: `Watchdog::clock(output)` hands out the clock; it reads the threads only if the last frame of the output was over its budget (`Run.over`), else the clock reads nothing. After: 0 reads in 100 calm frames, 2 reads per slow frame (start, stop). Call sites in tty.rs, winit.rs and offscreen.rs go through `state.watchdog.clock(..)`.
- Verdict: same, with one difference. The first frame of a slow run has no wait to excuse, which only makes the watchdog stricter; a run needs 2 s of slow frames to act, so it cannot end on that frame. The Busy note can come one frame later. Tests kept: the 4-thread starved frame blames nobody, parallel waits still blame, idle threads still blame. New: a starved frame after a slow one is measured and blames nobody.
- cfg(test) TASKS is now an empty temporary folder (OnceLock, `scratch_dir`) instead of /nonexistent, so the real read_dir path runs in the unit tests.
- Deviation from the brief: neither "once per interval" nor "only the renderer threads" (the renderer threads have no stable name or id to filter on). I chose "only after a slow frame": a calm output has no cost at all.
- Not done: the bench the review named (gate: bench) is the throwaway above, not a cell; the 500 ms relaxation in gate-tiling-audit.sh (review-core-watchdog-r1) is not part of this brief and was left alone.

### L16
- Cause: `fs::set_permissions(dir)` works on the path and follows a symlink.
- Red: a symlinked logs folder: `File::open_in` returned Ok, the target got 0700 and the log went into it.
- Fix: `private_folder` opens the folder with O_DIRECTORY|O_NOFOLLOW and sets 0700 on that handle (fchmod). A symlink gives the error "the log folder X is a symlink"; both callers (chora log.rs, voice log.rs) already log it once as "no log file: ..." and go on with stderr only. The compositor never stops for a log.
- Not changed: a chmod that fails on a real folder of someone else still stops the file (the review's second suggestion, "a failed chmod is a warning", is not in the brief).
- Docs: voice.md section 11 default filter (`info,tract_linalg=warn`, rmcp a warnings-and-errors cap); glossary "Log file" (moves at the start of the lock holder only); logs.md says a symlinked folder is refused.

### F22
- Closed on the head: e5e44719 (11-19, flakes lane) made the panic test wait for both lines ("chora runs" and the panic line). The failure of 11-17 was on aa8aa8c0, before that commit: the old test waited for the panic line only and then asserted "chora runs", so it failed in 0.25 s when the panic thread logged first. The stderr side has no race: the panic hook writes the fd before it writes the log.
- Proof on the head: `log_file` x 50 with `--test-threads=8` beside 8 busy loops, 0 of 50 failed; x 50 pinned to one CPU beside 8 busy loops, 0 of 50 failed.
- I could not reproduce the old race: the old test file (git show aa8aa8c0) also ran 50 of 50 green on 4 CPUs (8 busy loops) and 30 of 30 pinned. The race needs the slow start of a coverage build (llvm-cov). So no red sha for F22; no code change, no commit.

### Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,e2e`: pre-commit, gate-lib, ci-check, build, test, clippy PASS; e2e FAIL on one script, `gate-tiling-apps-click` ("a click on the Zebra row did not start zebra"). That is F27 (fuzzel 1.9.2 in this container; the cell hard-codes the 1.11.1 layout). It fails the same alone. I did not run it on 355cdbe5, so "same on the base" is from F27 and the fuzzel version, not measured. The sequence ran all 24 scripts; this is the only failed one.
Not green as asked: the gate is red on F27 only.
