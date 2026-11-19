# flakes-g r1 (F5, F26, F28, FF1), 11-19

Lane: cloud, opus, branch claude/flakes-g from claude/flakes-f 847f0964, merged
origin/claude/autonomous-dev-session-3w-i33pg7 (daf83391, clean) at start: 2513bee7.
Host: 4 CPUs, 15 G.

## Blocker: the lane container has no setup

The setup script of this container (119 characters, ran 12:08-12:14 UTC) installed none of the
build or gate dependencies: no libudev, libinput, xkbcommon, gbm, libseat, wayland-server, egl,
libdrm (pkg-config finds only pixman-1), no foot, fuzzel, waybar, grim; the Rust 1.99 toolchain
was not installed either (rustup fetched it on first use). `cargo check -p chora --lib` fails:
"failed to run custom build command for `libudev-sys v0.1.4` ... Package libudev was not found".
The brief forbids system packages, so no compositor, no gate script and no Rust cell of
crates/chora ran here. Owner fix: the environment's setup script (cloud environment menu, Edit,
Setup script) must again run the repository setup (apt build deps, foot, fuzzel 1.11.1, waybar);
new sessions then run it. Items 1-3 and rw-2 need it.

## Fixed and proven

| item | cause (file:line at 847f0964) | red | fix | proof |
|---|---|---|---|---|
| FF1 rw-4 under-load.sh no time limit | tests/under-load.sh:44 ran each command without a limit, so a hung gate hung the tool and its busy loops | 8d555fc5 tests/under-load-selftest.sh (new): `sleep 60` with --timeout 2 must end as a failed run within 10 s saying "timed out after 2 s", runs of `true` pass, no busy loop left in the tool's session; red: "unknown argument --timeout" | f0c9e640 each run under `timeout -k 10 S` (--timeout S, default 1800); a run at its limit fails with "timed out after S s"; the gate-lib step of merge-gate.sh runs the self-test | self-test green 4/4 (2 s each); tests/merge-gate-selftest.sh ok |
| FF1 rw-1 tiling-audit wait counted rounds | tests/gate-tiling-audit.sh:86 `fullscreen_rests` looped 80 x 0.25 s, each round two ctl calls plus jq | no red commit: the fix is in a gate script (a test for red-first); evidence: the scratch probe below | c227d433 loops to a deadline of 20 s by SECONDS | probe (fullscreen_state/fullscreen_rests cut out of the script, ctl stub answers in 1 s, bar never hides): before "rests=1 after 181 s", after "rests=1 after 20 s". Not run under the gate (blocker) |

## Not reached (blocker)

- 1 cube.open over 300 ms (tests/gate-construct-cube.sh:656, plugins/chora-construct/src/cube.rs:1142
  `open_cube`): needs the build and the cube gate; no red, no change. The proposal of flakes-f
  (pipeline the four output.add, or one output.add with a count, a small kit Conn API in docs/api.md)
  stands.
- 2 F26 doors "missing" and 3 F28 chat-keys "text typed before the pick did not reach the picked
  helper" (tests/gate-chat-keys.sh:103): need the compositor, foot and fuzzel; no reproducer, no
  cause. Leads unchanged (F26: train 22 dump, click on a hit node over mc's surface s1).
- FF1 rw-2 (crates/chora/src/words.rs:101 short-write cell): crates/chora does not build here.
- FF1 rw-3 (NIT, drag_pane pace) was not in this brief.

## Merge gate on the head

Not run: `tests/merge-gate.sh` starts with build, which fails here (libudev). Head c227d433 has
three commits over the merge 2513bee7, all shell (tests/under-load.sh, tests/under-load-selftest.sh,
tests/merge-gate.sh step gate-lib, tests/gate-tiling-audit.sh); pre-commit hooks passed on each.
The merge of the session branch must be merged again before the gate (brief).

Commits on claude/flakes-g: 2513bee7 (merge), 8d555fc5 (red), f0c9e640, c227d433.

# flakes-g r3 (F5, F26, F28, FF1), 11-19

Lane: cloud, opus, branch claude/flakes-g from c227d433; setup.sh ran (all present), `cargo check -p chora`
ok, foot/fuzzel 1.11.1/waybar present. Merged origin/claude/autonomous-dev-session-3w-i33pg7 at start
(9bc22c94) and before item 1 with origin/claude/cubesplit (74157504, a463d753; edits on the moved
cube/construct.rs). Host: 4 CPUs, 15 G. Load: tests/under-load.sh, 4 busy loops unless noted.

## Fixed and proven

| item | cause (file:line) | red | fix | proof |
|---|---|---|---|---|
| 3 F28 chat-keys "text typed before the pick did not reach the picked helper" | plugins/chora-assistant/src/chat.rs Session::said, List arm `(Some(index), _) => (Some(index), None)` (r3 base, ~:1016): when the field reads the keys "2" and Enter before the host's `pick_first` line, the host takes Say("moin") and Say("2") in one poll (field polled every 100 ms, chat/host.rs:69); a number alone dropped the held text, the pick cleared the field and sent nothing. services/voice/chat/src/session.rs:182 pick_first also restores only into an empty draft | eb5a23c7 cell a_number_that_comes_before_the_text_is_given_back_sends_the_text (both Says in one poll); red: no agent_say of "moin" | 28df6308 a number alone is a pick, as a click is: the held text goes to the picked helper once, by the chat path of said | cargo test -p chora-assistant 242/242; gate chat-keys 10/10 under load after. Gate-level red not reached: before the fix 10/10 under 4 loops and 10/10 under 12 loops (the race needs the field more than 1 s behind its keys) |
| 4 FF1 rw-2 short writes untested | crates/chora/src/words.rs:101 write_all: the F17 cell blocks SIGALRM on the writer, so its write is never cut | test only | 8d953291 cell a_backtrace_cut_by_the_alarm_of_its_own_thread_arrives_whole: SIGALRM sent to the writing thread (pthread_kill; a process alarm goes to the harness main thread), reader 300 ms late, asks for thread_start | mutation (write_all stops after one write): new cell fails "cut after 61691 bytes", the old cell passes; real code 10/10 under load |
| 1 cube.open over 300 ms | plugins/chora-construct/src/cube/construct.rs add_outputs: one output.add round trip per face, each waiting for a frame of the core | none under load here: 4 runs read 70-116 ms over construct.list (flakes-f's 414 ms not seen) | f075a20d kit Conn::call_all (chora_scene Client::call_all) pipelines calls, the core handles all lines of one wake (crates/chora/src/ipc.rs ipc_readable); add_outputs sends the four output.add in one round trip; docs/api.md IPC methods; kit cell call_all_answers_each_call_in_its_place | cube gate 10/10 under load, 65-141 ms (no measurable gain on this host) |

rw-1 and rw-4 were done in r1 (above).

## Found, not fixed

- 5 pull (gate-construct-pull.sh:163, 169 px for 200): not reproduced. 6/6 pass under 4 busy loops (67-73 s,
  260 px each). With the gate at nice 19 under 4 loops 1/3 pass; the 2 fails are another cell ("the
  fly-in ended before the key", the pane fly check), so too much load to count. Lead, unproven: the query
  rect of a meshed pane is the projected box of the mesh (crates/chora/src/ipc.rs:1302), and
  crates/chora-kit/src/wobbly.rs:228 caps simulated time at 100 ms per advance, so when the construct ticks
  slower than every 100 ms (wm5 r3 ran clippy beside the gate) the wobble after the drop settles slower
  than wall time and 1 s later the mesh still has depth (a narrower box). A red would be a wobbly cell: a
  released grid advanced every 400 ms for 1 s vs every 16 ms must reach the same state.
- 2 F26 doors "missing": not reached in this round (budget went to 1, 3, 5); no reproducer. Leads
  unchanged (train 22 dump: click on a hit node over mc's surface s1; testwatch: different cells each time,
  xp helper-key/helper-door, mc screenshot/volume, tiling keys-alias). The merge gate run below includes doors.

## Merge gate on the head

`tests/merge-gate.sh --jobs 3` on f075a20d (/tmp/merge-gate.azo2Xe): red-first, pre-commit, gate-lib, build
PASS; test FAIL: chora-xp wm::state::tests::a_give_shuts_the_start_menu and
a_give_with_the_drag_shuts_the_start_menu ("the give: xp has no window s1", plugins/chora-xp/src/wm/state.rs:287).
Not this lane: both fail 3/3 alone, and fail the same on origin/claude/cubesplit and on its base fa457b29
(the cubesplit lane's own merge of claude/wm7b and claude/wm3, not yet on the session branch); the session
head 05878d2a has neither test. The brief's merge of cubesplit brought them in. No gate step after test ran,
so doors (F26) did not run either. Next: the owner of the wm7b merge fixes the two xp cells (or the
merge order puts wm7b's fix first), then this head gets the gate again.

Commits on claude/flakes-g (r3): 9bc22c94 (merge), 8d953291 (rw-2 test), eb5a23c7 (F28 red), 28df6308 (F28 fix),
74157504, a463d753 (merges, cubesplit), f075a20d (item 1).

## Review r1

Lane: cloud, opus, read-only on claude/flakes-g f075a20d (own commits 8d555fc5 f0c9e640 c227d433 8d953291 eb5a23c7
28df6308 f075a20d; merges not in scope). `cargo check --workspace` ok.

Checks:
1) Red first: order holds (8d555fc5 < f0c9e640, eb5a23c7 < 28df6308). Red on parent reproduced: under-load-selftest at
   8d555fc5 fails "unknown argument --timeout"; the F28 cell with chat.rs of eb5a23c7 fails "did not reach the picked
   helper once: [agent_list]".
2) F28: a number alone picks with held=None (no send_draft), then said() in Chat sends the held text once; the host's
   clear after the pick follows the late pick_first on the same link, so the field ends empty; no double send.
   `cargo test -p chora-assistant` 242/242. One behaviour change, rf-1.
3) call_all: answers placed by id (id - first), events queued; scratch cell (reversed answers, event between, error in
   the middle, a call after) green. Real core buffers its output (ipc.rs respond, MAX_OUTPUT 8 MiB) and reads 64 KiB
   per wake, so no deadlock against it; a scratch fake core with blocking writes deadlocks at 2000 x 1 KB (rf-3).
   docs/api.md matches. `cargo test -p chora-kit -p chora-construct --features chora-kit/testing` green incl.
   call_all_answers_each_call_in_its_place.
4) 8d953291: mutant (write_all returns after one write) fails "the backtrace is cut after 61691 bytes"; real code 5/5.
   pthread_kill every 1 ms counts 5 ms per alarm, GIVE_UP_AT_LAST 60 s, so >= 12 s before a false "stuck": not
   flaky-red. Weakness rf-5.
5) under-load.sh: timeout(1) signals its process group; gates' gate_cleanup EXIT trap runs on TERM (checked: bash runs
   EXIT trap under timeout), so setsid children of gates end too. Selftest green (2 s). Gap rf-4.
6) c227d433: loop to a SECONDS deadline over fullscreen_state, sleep 0.25 per round: a real wait on a state.

Findings:
- rf-1 MINOR plugins/chora-assistant/src/chat.rs:1033: a number alone now sends the held text also when the user erased
  the given-back text in the field and then typed the number (the host cannot tell this from the F28 race), so text the
  user dropped reaches the helper; fix: the field tags each Say with whether it had applied the last pick_first, and
  the host sends the late text only when it had not.
- rf-2 NIT crates/chora-scene/src/ipc.rs:1180: an error without id (core parse error) is queued and dropped, so
  call_all waits forever where call_until takes it as its own; fix: return that error for the batch as call_until does.
- rf-3 NIT docs/api.md:823: pipelining has no stated bound; the client writes all before reading, so answers past the
  socket buffer plus the core's 8 MiB output become LIMIT_REACHED, and a core with blocking writes deadlocks; fix: name
  the bound in docs/api.md (or read while writing in call_all).
- rf-4 NIT tests/under-load-selftest.sh:17: the hung case checks the time and the line but not that the children of a
  timed-out run die; fix: hang with `sh -c 'sleep 60 & wait'` under setsid and pgrep the session after.
- rf-5 NIT crates/chora/src/words.rs:654: when the child is slower than 300 ms to its first write (load), the reader is
  already draining, no write is cut and the cell passes on the mutant too (false green, not false red); fix: read one
  byte first, then sleep 300 ms before draining.
- rf-6 NIT crates/chora-kit/src/conn.rs (cell call_all_answers_each_call_in_its_place): FakeCore answers in order, so
  the cell does not prove the out-of-order or event-between case of the brief; fix: add the scratch cell above to
  chora-scene ipc tests.

Counts: BLOCK 0, MAJOR 0, MINOR 1, NIT 5. Known, not findings: xp a_give_shuts_the_start_menu (fa457b29, integ
a7d671f2); pull and F26 not reproduced.

Verdict: MERGE-OK.
