# Review claude/core-watchdog (cdaea871), sh-22

Verified: red f9080c67 fails as claimed ("for (40.37ms, 1ms)"; note that a git-archive copy builds against stale artifacts unless the sources are touched). HEAD: cargo test -p chora safety 21/21 ok; clippy -p chora --all-targets -D warnings clean. Claim 1 changes no tolerance and no explicit wait.

MAJOR crates/chora/src/safety.rs:84-89 with :131. The waits of all threads are summed, so parallel waits count several times over: 4 renderer threads that each wait 10 ms of a 40 ms frame add up to 40 ms, the whole frame. min(took) then excuses all of it. Failure: a costly scene (a client with heavy effects, 40 ms against 16 ms) on a lightly loaded host. A client can cause that load itself with a busy loop in another process, so effects are never dropped and the watchdog only sends Busy notes. That breaks "an escape hatch no plugin overrides". Reproduced with a scratch test in the existing seam, Tasks::frame(0, 10): it returns [Busy { took: 41ms, waited: 40ms }] and no Blame. The builder's report admits it ("a costly scene on a loaded host is therefore excused"). Fix: use the largest per-thread wait delta, which is bounded by wall time, or another bound of the wall-clock stall. Do not use the sum. Then check that gate-watchdog-starved and the audit cell under load stay green. Earliest gate: a unit test (frame(0,10) must Blame) next to a_costly_scene_beside_idle_threads_still_blames.

MINOR crates/chora/src/safety.rs:84-89, 114. Cost per frame, measured: 20 threads give 64 us per read and 129 us per frame (2 reads). That is about 104 syscalls per read (openat, statx, 2 reads, close per thread, plus getdents) and about 12k syscalls/s at 60 Hz, 0.8% of one CPU per output, with a PathBuf and a String allocated per thread. The read runs on the frame path, inside the measured time: Instant::now() comes before wait_in. Failure: on multi-output tty with more threads, the sum is n_outputs x 0.13 ms on the main thread. The old code kept one fd and used pread. Fix: cache an fd per tid, or read only the threads known to render. Earliest gate: none (bench).

OK (no finding) on failed reads. No procfs or a sandbox: read_dir gives None, waited is 0, and the hatch stays strict. A thread that exits mid-read is skipped by filter_map, so its wait is lost and saturating_sub only makes the watchdog stricter. One malformed schedstat makes Sum<Option> None, which is also strict. No panic path.

MINOR crates/chora/src/safety.rs:75. In cfg(test) the production FrameClock::start()/stop() read "/nonexistent", so no unit test covers the real /proc/self/task path in the frame clock, only wait_in(PROC_TASKS) as a smoke test. Earliest gate: gate-watchdog-starved (e2e).

COSMETIC tests/agent.sh:167-171. The 4 extra grim calls before the capture add a few hundred ms before `ctl capture`, so step 6's timing shifts, and a timing-dependent F25 could pass on the new code. If grim fails, `set -e` now stops the script before the capture, without a fail message. Earliest gate: CI e2e.

COSMETIC tests/agent.sh:180. The grep "watchdog\|effects" misses the busy note ("the host is busy and the core waited for a CPU"), which is exactly the line that tells a starved host from a drop. Add "busy". Earliest gate: CI e2e.

500 ms relaxation (tests/gate-tiling-audit.sh, 366c1cb3, train 17): yes, a follow-up should remove the CHORA_FRAME_BUDGET_US=500000 export and its header lines once train 17 and this branch both land, so the audit runs the default budget again. Do it after the MAJOR fix, and run the 2560x1600@2 cell under 8 busy loops again, because a tighter wait bound excuses less.

Verdict: FIX
