# flakes-f r1 (F5, F17, F21, F29), 11-19

Lane: cloud, opus, branch claude/flakes-f from dc4bc46d. Host: 4 CPUs, 15 G, no stress-ng;
load = `tests/under-load.sh` (new): N runs of a gate beside one busy loop per CPU (4).
F27: `fuzzel --version` = 1.11.1 +cairo +png +svg(nanosvg) -assertions.

## Fixed and proven

| item | cause (file:line at dc4bc46d) | red | fix | proof |
|---|---|---|---|---|
| F21/F5 pull 168-197 px of 200 | plugins/chora-construct/src/pull.rs:316 `take_pulled` placed the pane, its depth (:316) and its yaw (panes.rs:217 `move_pane`, input.rs:764 `drag_pane`) for the pose the camera had reached mid fly-in (the handover flies in, the drag turns it back); late in the flight the depth is clamped to focal/4 and the pane ends behind the face, and at the right edge (the gate pulls at x 1480) the yaw toward the flight pose foreshortens it | f106713d `a_window_pulled_during_the_flight_into_it_comes_at_1_to_1_where_the_camera_stays`: 1532 deep for 832; f1db38ad same cell at x 1480: yaw 1.32 for 0.55 | 10c5a50b depth and place for `Construct::target()`; 847f0964 yaw toward it too (`move_pane_facing`, drag faces the end of a glide) | pull 10/10 under 4 busy loops, width 260 px in all 10 (before: 3/5 and 7/8 at the same load; after 10c5a50b alone 206-259 px, and 173 px in merge gate 1) |
| pull "the grant input.inject" (socket: No such file) | tests/gate-lib.sh:60 `session_start` polls an env file that `pull behind` (run twice) reuses; it could read the gone socket of the first session before the new one truncated the file | f106713d gate-lib-selftest case (new session held 1 s before its redirect): took /old socket | 10c5a50b `: >"$env_file"` before the start | selftest green; in the 10/10 above |
| F17 words slow backtrace | crates/chora/src/words.rs:197 `stuck`: a handler whose 170 KB backtrace fills the stderr pipe sleeps in write(2) (state S, no CPU) while the reader (test harness, journal) is slow; an alarm on another thread ends it as stuck. Also :98 `write_all` wrote once: an alarm on the writer cuts the words at 64 KB (seen: 63954 bytes) | 16dbdd4a `a_backtrace_into_a_full_pipe_waits_for_its_reader` (SIGALRM blocked on the faulting thread, reader 300 ms late): "stuck after 205 ms in the state S", 3/3 | cc413838 the alarm reads /proc/self/task/TID/syscall: a write(2) to fd 2 waits for its reader (as R, D); write_all loops on short/EINTR writes; no malloc in the alarm | words:: (12 cells) 10/10 under 4 busy loops beside the tiling gate |
| F29 tiling-audit fullscreen | tests/gate-tiling-audit.sh:184-186 captured 4 s after `foot -F`; waybar hides on SIGUSR1 when it gets the CPU | d9100536 `BAR_STALL=6` (waybar stopped 6 s): "the bar shows over the window at x 66" 2/2; BAR_STALL=30 gives both lines of train 26 | 5955b3a9 wait (<= 20 s) until the window covers the output and `work_area` has no bar reserve; a bar that never hides fails with its own line | tiling-audit (full step, 4 rounds) 10/10 under 4 busy loops with BAR_STALL=3 (40 fullscreen acts) |

## Found, not fixed

- cube (kit2b "a still cube draws"): not seen in 5 runs under 4 busy loops (4/5 pass). The fail was
  tests/gate-construct-cube.sh:656 "cube.open took 414 ms, 380 ms more than construct.list" (the 300 ms
  of docs/cube-modes.md; 108-212 ms in the passing runs). Cause: plugins/chora-construct/src/cube.rs:1142
  `open_cube` makes about 8 sequential core round trips (clients.list, output.add x4 at :1607, apply,
  modes, frame), each waiting for a core loop that renders software frames under load. Fix proposal:
  pipeline the four output.add (write all, then read the answers) or one output.add with a count;
  needs a kit Conn API. Not a gate wait.
- doors (F26) and chat-keys (F28): not reached in this round (budget). F26 lead stays the train 22 dump
  (click lands on a hit node over mc's surface s1); F28 held after flakes r6 per backlog.

## Merge gate on the head

- Gate 1 on 5955b3a9 (`--jobs 3`): 25 steps PASS, then pull FAIL 173 px at x 1063 (the yaw half of F21, fixed in 847f0964).
- Gate 2 on 847f0964: killed with a worker restart after 26 steps PASS.
- Gate 3 on 847f0964 (`--jobs 3`, /tmp/merge-gate.jW4RQ3, lane container): all 41 steps PASS (pull 70 s, cube 114 s, tiling-audit 348 s, doors 343 s, chat-keys 8 s), 11:18 UTC.

Commits on claude/flakes-f: f106713d (red), 10c5a50b, 16dbdd4a (red), cc413838, d9100536 (red), 5955b3a9, f1db38ad (red), 847f0964.

## Review r1

Reviewer: cloud lane, opus, read-only on claude/flakes-f 847f0964 (base dc4bc46d, 9 files, +322 -35). Brief briefs/review-flakes-f-r1.md.

Ran: `cargo test -p chora-construct` 190/190 ok; `cargo test -p chora --lib words` 12/12 ok; `bash tests/gate-lib-selftest.sh` ok;
reds confirmed: f1db38ad pull cell fails (yaw 1.32 for 0.55), 16dbdd4a words cell fails ("stuck after 150 ms in the state S");
under-load.sh killed with TERM leaves no busy loop.

Checks:
1. F21 pull: take_pulled places, sizes the depth and faces the pane for `target()` (= pose without a glide, so unchanged
   at rest); `boxed` uses the same pose. Other callers: `move_pane` (plugin.rs:148 pane.move) keeps the current pose;
   `drag_pane` now faces `target()` and is equal to the old pose when no glide runs. No caller relies on the old pose. OK.
2. F17 words: `stuck` treats a write(2) to fd 2 (/proc syscall) as held up; a lock wait (futex, state S) is still
   stuck at GIVE_UP_AFTER, and every wait still ends at GIVE_UP_AT_LAST (60 s), so a reader that never reads cannot
   hang the core. write_all: EINTR retries (SA_RESTART makes it rare, a signal per alarm only), short writes advance,
   0 and other errors end; no spin, no malloc (Cursor path, stack buffers, errno read). /proc open failure gives
   false (old behaviour). New code uses /proc and SYS_write only, as the existing state_of/gettid; the crate is Linux-only. OK.
3. F29 tiling-audit: own FAIL line, the audit still runs after it, waybar is at the top so work_area[1] is the bar
   reserve. Bound is not 20 s, see rw-1.
4. gate-lib: `: >"$env_file"` truncates before the new session; the old session is killed and waited in end_session,
   headless-session.sh cleans its core on TERM and the core logs elsewhere, so no old line can land in the new file. OK.
5. under-load.sh: busy loops are `sh -c` (no package), pass = exit 0 and no ^FAIL line, no sleeps. OK, see rw-3.

Findings:
- rw-1, MINOR, tests/gate-tiling-audit.sh:86, `fullscreen_rests` bounds iterations (80 x 0.25 s), not time: each round runs two `ctl` calls (timeout 10 each) and jq, so under load the wait can run far past 20 s (worst 80 x 20 s) while the FAIL line says "within 20 s"; fix: loop on a deadline (`end=$((SECONDS + 20)); while [ "$SECONDS" -lt "$end" ]`).
- rw-2, MINOR, crates/chora/src/words.rs:101, no cell covers the short-write path of `write_all` (the 63954-byte cut of the report): the new cell blocks SIGALRM on the writer, so with SA_RESTART its write is never cut; fix: add the same cell without the sigmask (alarm on the writing thread, reader 300 ms late) asserting `thread_start` arrives.
- rw-3, NIT, plugins/chora-construct/src/input.rs:755, `drag_pane` takes the pace k and the forward of `self.pose` while it faces `target()`: a drop during the turn back lands off the pointer (it catches up on the next move after the glide); fix: use `self.target()` for k and forward too, or say why the current pose is meant.
- rw-4, NIT, tests/under-load.sh:44, a run has no time limit, so a hung gate hangs the tool; fix: wrap each run in `timeout` (a `--timeout S` option, default 1800).

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 2 MINOR, 2 NIT).
