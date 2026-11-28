# flake-gate

## r1

Lane: cloud, `claude/flake-gate-r1` from 0e18f14c, head **9d9a9da21** (pushed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` green. Logs: media/2026-11-26/flake-gate-r1/ (each file under 5 KB).

### Misses before and after

Load before: 6 busy shell loops beside the cell (round 1), plus 4 gates in parallel with `CHORA_GATE_LOCK_HELD` and `TILING_LOAD=1` (sh-43, sh-45). After: the same 6 busy loops.

| sighting | cell | before | after |
|---|---|---|---|
| sh-41 | tiling-bar, gate-tiling-one-bar.sh (persona) | **1 of 10** (run 3) | 0 of 10 |
| sh-43 | tiling-notice, gate-tiling-notice.sh | 0 of 10, 0 of 30 parallel | 0 of 10 |
| sh-45 | xp-notepad, gate-xp-notepad.sh | 0 of 10, 0 of 20 (24 busy loops), 0 of 30 parallel | 0 of 10 |
| sh-46 | acts tiling, `gate-acts.sh --no-selftest tiling` (0e18f14c tree) | 0 of 10 | 0 of 10 |
| (control) | tiling-bar, gate-tiling-bar.sh | 0 of 10 | 0 of 10 |

The full `gate-acts.sh` takes over 10 minutes under load, so the repro used the tiling look only (the cell of the sighting). In round 1 one acts run ended rc=124 at my own 600 s timeout; that was the timeout, not a miss. A first "after" attempt with `TILING_LOAD=2` hung in gate-tiling-one-bar: `session_start` appends the busy-loop pids after the session pid and `stop_session` kills `pids[-1]`, a busy loop. That is a property of the load option for this cell, not of the fix; the after runs use external busy loops like the before runs.

### sh-41: "persona session.sh tiling: 3 waybar processes on one config" (reproduced)

Log (before-tiling-one-bar-miss-run3.log): `FAIL: persona session.sh tiling: 3 waybar processes on one config: 11878`. The list at the failure holds one pid while the count said 3: the extra processes were gone again. Root cause: waybar forks a shell for each of its nine tag modules and the other custom modules, and the child keeps the name `waybar` until it execs. `bars()` counted `pgrep -x waybar` with the session's runtime directory, so a fork in that window is a second bar; a loaded host stretches the window. A sampler of /proc (waybar-sampler.py, 40 s, one product session, 6 busy loops) saw a count of 3 and 42 samples of a `waybar` whose parent is a `waybar`. Not a double bar (ti-47), not a start race of the panel. Red b3404d9d4: new cell `fork` (a fake waybar that forks a child of its name): `FAIL: fork: a waybar with a forked child of its name counts as 2 bars, not 1`. Fix c27f438e7: `bars()` counts the waybar processes whose parent is no waybar (a second real waybar still counts; the cell checks it). 9d9a9da21 replaces the two `sleep` of the fake by `tail -f /dev/null` (sleep-ratchet).

### sh-46: "acts tiling I14: chora-tiling.launch disconnected" (0 of 10; cause from the code)

Not reproduced in 10 runs on 0e18f14c under load. Cause from the code: `chora-tiling --launch X` connects as `chora-tiling.launch-PID` (plugins/chora-tiling/src/main.rs `launch_name`), asks one thing and ends. `Session.clients()` of tests/acts/harness.py drops only `choractl`, `chat-door`, `chat-opener` (ONE_CALL). A snapshot that happens to see the launch and the next that does not makes I14 report "disconnected, the act did not declare it", with "no program of the session had ended" as cause, the line of the sighting. Red 6469eaf4e: unit test `Clients.test_a_launch_of_tiling_is_no_client` fails on the head (`chora-tiling.launch-4242` stays). Fix 76ad29913: `clients()` also drops names that start with `chora-tiling.launch`. The red is the unit test; the sighting itself stays unreproduced.

### sh-43: "gate-tiling-notice.sh:31 choractl: socket: No such file or directory" (0 of 10 + 0 of 30; no fix)

No reproduction, so no fix (brief: no fix without a red). From the code: the text `socket: ...` is the `ClientError::Io` of a connect to `CHORA_SOCKET` (an admin connect names its path first); line 31 is `ctl capture` right after `grant`, which goes through the admin socket and had worked. A main socket that is gone after the core answered queries means the core ended (it removes both sockets on exit; the gate's headless-session.sh then removes the runtime directory), for instance because its startup command chora-tiling ended. The log of the sighting was not kept. Commit dd5d06192 adds `core_diag` to gate-lib (core pids, socket directory, last log lines) to the FAIL line of that capture, so the next miss names the cause. A diagnostic only, no behaviour change.

### sh-45: "xp-notepad: the client typed ':end' and not start::end" (0 of 10 + 0 of 20 + 0 of 30; no fix)

No reproduction, no fix. One hypothesis was tested and dropped: a shell in the typing foot that starts late (a `sleep 3` before `stty raw`): the tty keeps the early text and the cell passes, so a late `stty` is not the cause. What remains: "start:" was typed before the keyboard was with the foot (the gate checks `focused == true` of the state, not that the client holds the keyboard after the Start menu closed), "start:" was lost and ":end" arrived after the four chords. The sighting failed at 13 s, so the host was not slow. The same dd5d06192 adds the surfaces with their `focused` flag and `core_diag` to that FAIL line.

### Full gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no step filter) on 9d9a9da21: 67 PASS, 0 FAIL, `docs` SKIP (mkdocs not installed here, docs unchanged), "all steps passed; no --push, nothing is pushed" (media: full-gate.txt). An earlier start failed `ratchets` on my two `sleep` in the fake waybar (fixed in 9d9a9da21). `pgrep -fa 'chora --headless'` empty after the runs.

### Open

sh-43 and sh-45 stay open without a red; their FAIL lines now carry the state of the core and the surfaces. sh-46 has a unit red and fix but no reproduction of the sighting itself.
