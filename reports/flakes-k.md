# flakes-k (F5: no flaky tests)

## r1

Lane: cloud, branch `claude/flakes-k` from `6192f696`; the merge of
`origin/claude/autonomous-dev-session-3w-i33pg7` was "Already up to date". Written 11-20.
Load: `tests/under-load.sh` (4 busy loops, 2 or 6 or 8 where said), other runs beside it. Logs are
under `/tmp/claude-0` of the container only.

| Item | Result | Red, fix | Loaded runs after the fix |
|---|---|---|---|
| 1. FLK-AG1 | cause found, fixed | `b7eddbf9`, `92185802` | agent.sh 10 of 10 (before: 5 of 20 failed) |
| 2. FLK-IN1 | cause found, fixed | `fd0818fc`, `fa53ad43` | whole lib binary 10 of 10 |
| 3. FLK-TR1 | cause found, fixed | `9967f062`, `1bbfdd4a` | subset 0 of 300 (before 3 of 260); whole lib 10 of 10 |
| 4. FLK-HUD1 | seen 1 of 20, cause not named | none; evidence log `f27eb2e3` | n/a |
| 5. FJ-R2 | tiling and mc fixed; construct not shown | `90e9e743`, `2f069c98` | tiling 10 of 10, mc 10 of 10 (helper cells) |
| 6. FJ-R1 | done | `e2508ec1` (red shown on the old scan) | churn test 1 s, 3 of 3 |

### 1. FLK-AG1
Cause. In step 22 the click is injected by `choractl --name user inject` and the client's `free_apply`
(tests/gate_mcp.py) says hello as `user` too. The core sends `pointer.pressed` to the MCP client before it
answers the inject call, so the client runs the denied camera command and its hello while the choractl of
the click is still connected; two programs under one live name: refused, code 7. Evidence: loaded run 11
of 15 (then 16 to 19, with my own builds beside): `hello: "user" is connected from another program`; in
the chora.log of the failed run the hello was refused 0.8 ms after the refusal of the confined camera
command. Red `b7eddbf9`: a choractl of the injector's name kept connected for 3 s
(`injector_exits_slowly`): step 22 FAIL every time. Fix: the click is injected as `clicker` (own grant);
`user` stays the free client. Step 21 was not involved (the backlog guess).

### 2. FLK-IN1
Cause. The activity of the person ends the idle state and starts the 10 ms idle timer again; a roundtrip
slower than 10 ms brings `true` after `false`: `[false, true]`. Not reproduced as a flake: the test alone
40 of 40 loaded, the whole lib binary 20 of 20 (2 busy loops). Shown with a pause of 50 ms between the
activity and the roundtrip: red every time (`fd0818fc`, 3 of 3). Fix: the test accepts `[false]` and
`[false, true]`, nothing else. No timeout changed.

### 3. FLK-TR1
Cause. `Id(57005)` is `0xDEAD`, the id that `NoSubscriber` gives a span. In the unit tests only the
`logged` helper of ipc.rs set a (global) subscriber, late, in whichever test ran first. A scoped
subscriber of another test (`log::tests`, `with_default`) makes a callsite enabled for every thread, so a
core that is built before the global subscriber exists makes smithay's seat span under no subscriber
(id 0xDEAD). The `#[instrument(parent = &seat span)]` spans of `add_keyboard` then run under the global
registry, which does not know 0xDEAD: "tried to clone Id(57005), but no span exists". Evidence: the 5
tests reservation, two `a_refused*`, `hang::the_stall`, `log::a_dependency` failed 1 of 60 and 2 of 200;
without the `logged` test 0 of 200, without the `with_default` test 0 of 300. Red `9967f062`
(`a_core_makes_its_spans_under_a_subscriber`: red alone or first; in a full run it depends on the order).
Fix `1bbfdd4a`: `Core::new` installs the one global subscriber (moved to testing.rs, with a per-thread
filter that drops everything in a thread with no sink, so no test pays for formatting); the same subset
0 of 300.

### 4. FLK-HUD1
`context-held-for-question|context-picks-folder`, 20 loaded runs: 1 failed (run 6, with a lib loop beside):
`timeout 5s: figure_left` with the figure still at its rect. 100 further loaded runs (4, 6 and 8 busy
loops, once with the lib tests beside) did not fail: the figure left after 0 s 64 times and after 1 s 36
times. Read: `Switcher` (context.rs) holds the switch while a card waits and re-arms 150 ms after it
ends; no path found that loses the switch, and assistant.log of the failed run was empty (no
`hud mine:` error). Not slow vs lost: unknown. Change `f27eb2e3` (evidence only): after the 5 s miss the
cell watches 30 s more and logs `the figure left after more than 5s` or `never` to hud.log; the result
stays missing.

### 5. FJ-R2
With `late_field 2` (the xp stress) in the host of the look: tiling helper-key missing; mc helper-key,
helper-door, helper-none and lock/power/screenshot/volume-door missing (the later cells behind a chat
that stayed), 3 of 3 (mc without it: all pass). construct passes at 2 s (at 6 s helper-key fails, not
shown to be the Escape), so it is unchanged. Fix: `wait_for_field` in lib.sh (moved from xp), called by
`chat_close` of tiling and mc. Loaded (4 busy loops) runs of helper-key, helper-door, helper-none:
tiling 10 of 10, mc 10 of 10; xp still passes.

### 6. FJ-R1
The churn test runs 1 s. On the old scan (unwrap of the status read, as before `eef3b496`) it is red in
0.2 s, 5 of 5, with `No such process`; on the scan as it is, 3 of 3 green.

## Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,doors,hud,e2e` at
`e2508ec1` (the session branch merge: already up to date): pre-commit 6 s, gate-lib 67 s, build 61 s,
test 196 s, clippy 19 s, ci-check 39 s, hud 71 s PASS. e2e FAIL 292 s: `gate-tiling-apps-click` "a click
on the Zebra row did not start zebra" (24th script; the 23 before it pass or skip as before). Rerun alone
once: the same FAIL in 16 s. Not caused by this branch: it changes no product code (the diff of
`6192f696..e2508ec1` is `cfg(test)` modules in chora, tests/agent.sh, doors and gate-hud), and that
script and its binaries are untouched; not bisected at the base. The gate stops at the first failed
step, so doors ran alone afterwards (`--only doors`): PASS 361 s. The e2e scripts after the 24th did not
run.

## Open
- gate-tiling-apps-click fails here alone (see Gate): the next lane should bisect it.
- FLK-HUD1 stays open with the evidence line (item 4).
- construct doors: the Escape pattern is unchanged; not shown.
- The red of `9967f062` depends on test order in a full run; it is red run alone.
