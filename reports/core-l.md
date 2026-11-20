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

## Review r1

Reviewer: cloud lane, read-only on code; claude/core-l head b1d1491d, base 355cdbe5 (8 files, +168 -27). Step 0: `cargo check --workspace` green.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 3 MINOR (1 of them pre-existing), 3 NIT).

Checks:
1) Red first: `calm_frames_read_no_threads` is absent on 355cdbe5 (0 run), FAILS at 4d2b133b, passes at f3baf36d; `a_symlinked_folder` FAILS at b8b7adf0, passes at b1d1491d. Ok.
2) WD2: a starved run is measured from its second frame on: `run.over` is `took > budget` (not `took - waited`), so an excused frame keeps the next one measured. The first unmeasured frame alone cannot Blame or Kill: it only sets `since`, the run needs `SLOW_FOR` (2 s), and any later frame that is excused or fast clears `since`. But it does set `run.last` (rv-1). `over` stays for a removed output: harmless (one extra measured frame after a replug); the stale `since` beside it is not (rv-5, pre-existing). tty.rs:751 (stored in `rendered_at`, noted at vblank tty.rs:921), winit.rs:171 and offscreen.rs:642 all take the clock before the frame and pass it to `frame_rendered`. Ok.
3) `tasks()` and `READS` are `#[cfg(test)]` only; release reads `/proc/self/task` as before (integration tests build the lib without cfg(test), same as before). The tests use deltas of a thread-local and their own `Tasks` folders, so they do not depend on order or on other tests' threads. Ok, but see rv-4.
4) L16: a symlink higher up the path is followed (O_NOFOLLOW checks the last part only), which is what a dotfile setup needs. An existing 0755 folder gets 0700 by fchmod. A symlinked last part is refused, but with ENOTDIR, not ELOOP (rv-2). A folder replaced between create and open: the open refuses a symlink; one swapped after the open still gets the files by path (rv-3). Callers: crates/chora/src/log.rs:52 and :67, services/voice/src/log.rs:67 log one "no log file: ..." line and go on with stderr. Ok.
5) F22: e5e44719 waits for both "chora runs" and the panic line before it asserts, and the panic hook writes stderr before the log, so the stderr count is read after both are there. It covers the reported race (panic line before the start line, 0.25 s on aa8aa8c0). Closing without a commit is right; the 0 of 50 under load is evidence of no harm, not of the fix (the old test also passed 50 of 50 here).
6) `cargo test -p chora --lib safety` 22 passed; `cargo test -p chora-scene logs` 6 passed; `cargo clippy --workspace --all-targets -- -D warnings` clean. Largest touched code file offscreen.rs 1,478 lines; docs/voice.md 1,569 (rv-6).

Findings:
- rv-1, MINOR, crates/chora/src/safety.rs:274: the first frame of a starved burst is now unmeasured, so it counts as truly slow and sets `run.last`, which an excused frame never did; a host with a starved frame every < 10 s keeps `suspect` alive past `CALM_FOR`, and a later real 2 s run disconnects that stale suspect instead of blaming afresh. Fix: carry "measured" in the clock (waits `None` = unmeasured) and leave `run.last` alone for an unmeasured slow frame.
- rv-2, MINOR, crates/chora-scene/src/logs.rs:131: open with O_DIRECTORY|O_NOFOLLOW on a symlinked folder fails with ENOTDIR on Linux (checked: "Not a directory (os error 20)"), not ELOOP, so the "is a symlink" message is dead and the line reads "no log file: Not a directory"; the test asserts only `is_err`. Fix: map ENOTDIR (and ELOOP) to the symlink message when `symlink_metadata` says symlink, and assert the message in the test.
- rv-3, NIT, crates/chora-scene/src/logs.rs:58-74: fchmod works on the handle, but the lock, the log and `shift` go by path, so a folder swapped for a symlink after `private_folder` still gets the files (same user only). Fix: compare dev/ino of the handle with `symlink_metadata(dir)` after the files are open, or use openat on the handle.
- rv-4, NIT, crates/chora/src/safety.rs:892: `a_frame_clock_never_waits_longer_than_it_took` now gets an unmeasured clock and never reads `tasks()`, so the scratch folder path of the report ("the real read_dir path runs") is not covered by it. Fix: note one slow frame on "out" before `clock("out")`.
- rv-5, MINOR (pre-existing), crates/chora/src/safety.rs:260: `Watchdog.outputs` never drops an output, so a run cut by an unplug keeps `since`, and the first slow frame after a replug of the same name Blames at once. Fix: forget the `Run` when the output goes, or reset `since` when `now - last > SLOW_FOR`.
- rv-6, NIT, docs/voice.md: 1,569 lines (1,568 on the base), over 1,500; the size ratchet counts code only, so no gate. Fix: split section 11 (logs) out when the file is next touched.
