# core watchdog r1 (sh-22 / F23, F25): fixed, F25 not explained

Branch claude/core-watchdog (base 2d9f306f), pushed. Commits:
- 875d0812 test(agent): step 6 names the host pixel before and after the capture and the watchdog lines of chora.log (item 1)
- f9080c67 test(safety): red, "a frame the renderer threads waited for blames nobody" (item 2)
- f67ad247 fix(safety): the frame clock sums the run queue wait of all threads (item 2)
- cdaea871 test(safety): in unit tests the clock reads no thread folder (item 2, see below)

## Item 1 (diagnostic)
Done, not run on CI. Earliest gate: the e2e job of CI (agent.sh is no merge-gate step); locally agent.sh step 6 passed.

## Item 2 (sh-22)
- Cause shown, not guessed. Audit desktop cell, 2560x1600@2, ink, 8 busy loops on 4 cores, base build: FAIL
  (tile border, glow, shadow read as wallpaper), 5 x "the watchdog drops all effects" in the core log.
  Per-thread schedstat over 20 s: main thread waited 4.3 s (ran 0.6 s); the four llvmpipe threads waited
  11.7-12.1 s each (ran 0.7 s each). The old clock subtracted only the main thread, so most of the
  wait counted as scene cost. The old doc comment (all threads 17.2 s vs main 16.7 s) did not hold at this size.
- Red: a_frame_the_renderer_threads_waited_for_blames_nobody fails on f9080c67 ("[Blame] for (40ms, 1ms)").
  Its twin a_costly_scene_beside_idle_threads_still_blames passes before and after (escape hatch).
- Fix: FrameClock reads /proc/self/task/*/schedstat before and after the frame and sums the waits. Cost: a
  readdir and one small read per thread (about 15) twice per frame.
- cdaea871: in a full `cargo test` the parallel tests are threads of one process, their wait counted as host
  load and offscreen::the_watchdog_measures_the_render saw no slow frame (merge-gate attempt 1). In cfg(test)
  the clock reads no folder; the new tests pass a folder of their own.
- Green: cargo test -p chora (583 passed); tests/gate-watchdog-starved.sh both cells ok (a starved host
  blames nobody; budget 1 us still blames mc).
- Earliest gate: the red unit test above, then the tiling audit under load.

## Item 3
- The 500 ms relaxation is not on the base. It is in 366c1cb3 on claude/flakes-r5 (train 17, not landed),
  so there was nothing to remove here: the base already runs the default budget. When train 17 lands, drop
  the `export CHORA_FRAME_BUDGET_US=${...:-500000}` line of tests/gate-tiling-audit.sh (and its header lines).
- tests/gate-tiling-audit.sh, SIZES=2560x1600@2 (both themes, all cells), default budget, 8 busy loops
  (`while :; do :; done`) on 4 cores, 3 runs in a row: 3 of 3 exit 0, 23 ok lines each, 0 "watchdog drops"
  in the core logs, 4 "host is busy" notes per run. Same cell on the unfixed build failed 1 of 1 (see above).

## Merge gate
tests/merge-gate.sh: attempt 1 (before cdaea871) stopped at test (the offscreen test above). Attempt 2 at cdaea871
stopped at pull (known flake F21, harness-r3: pane 192 px wide for 200; no watchdog involved). Attempt 3 at
cdaea871: "all steps passed", all 28 steps PASS (log dir /tmp/merge-gate.yivCIz in the lane container).

## F25 (glow at 5,5 doubled on the CI host frame)
Not explained by item 2. Dropped effects strip glow from the host frame and the capture alike (ci-f25-r1:
budget 1 gave 0 0 0 on both), and CI step 6 runs without load. The CI value 19 15 28 vs capture 10 8 15
needs the new step 6 message from a CI run: host pixel before and after the capture, and the watchdog lines.
If those show "no drop" and a stable 19 15 28 on the host, F25 is a host-path double blend (core), not the watchdog.

## Left open
- Not run: tests with a GPU or pixman as the renderer; the per-frame cost of the thread sum was not measured.
- Sum over threads excuses a frame when the threads wait in parallel (it clamps to the frame time); a costly
  scene on a loaded host is therefore excused as well. Only a quiet host still blames.
