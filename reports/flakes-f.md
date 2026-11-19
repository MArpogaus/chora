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
