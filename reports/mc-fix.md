# mc-fix

## r1

Lane: cloud, branch claude/mc-fix from dac835dd (train 85), merged with origin/claude/autonomous-dev-session-3w-i33pg7 (already contained). Step 0: `git fetch --unshallow` and `cargo check --workspace` green. Pushed: claude/mc-fix at b4104cae. Captures: reports/mc-fix/.

| item | verdict | red | fix | cell or test | capture |
|---|---|---|---|---|---|
| 1 mc-47 click on a launcher row | not a bug of mc (candidate reproduces, cause is fuzzel) | cell written, dropped (see below) | none | replay in reports/mc-fix/mc-47-* | mc-47-fuzzel-events.log, mc-47-fuzzel-rows.png, mc-47-mc-after-click.png |
| 2 mc-49 xp+mc never quiet | fixed | c3432638 | 55f75baa | mc unit test a_build_in_the_midst_of_a_write_leaves_mc_quiet | check.py start xp+mc 3 of 3 quiet |
| 3 mc-48 I5 at Super+a | checker misread of a cross-fade, fixed at its source | 137437c5 | bb3de644 | mc unit test a_cross_fade_keeps_mc_busy_until_its_end | mc-48-busy-after-fix.txt |
| 4 P2-CHECK | done, checker only | tests with the change | b4104cae | tests/test_acts.py (6 tests), tests/persona-selftest.sh | |

### 1. mc-47
`mc-sofa launcher-click-row` was written and was red: real fuzzel on mc (a Foot entry, the list Chat with a helper, Foot, Screenshot), a pointer move and a click on the row Foot starts nothing; the control with typed `Foot` and Return passes in the same cell (the window of the program gets its page). The cause is not in mc or the core:
- `WAYLAND_DEBUG=1 fuzzel --dmenu` under chora-tiling (mc-47-replay-tiling.sh, mc-47-fuzzel-events.log): fuzzel gets `wl_pointer.enter` at the row, `motion`, and `button` press and release (272) at the row beta, and prints nothing, stays open. Held press (hold) and a plain tap give the same.
- The installed fuzzel is 1.9.2; its binary has no handling for clicks (its strings and the events agree). The same click on tiling does nothing as well, so no flavor starts a program by a click on a row.
- So the click of r2 sofa (and r1: it fell on the page below) cannot start a program with this fuzzel. The cell can never go green here and a red cell in mc-sofa blocks every merge, so it is not on the branch (patch kept: mc-47-cell.patch, sha ddd43ea3 before the reset).
- For the PM: T1 by mouse needs a launcher that takes clicks (a newer fuzzel if one does, not checked; or a list that mc draws itself). That is a design item, not a fix of this lane. `launcher-click` is no step of tests/merge-gate.sh (--list), the gate ran without it.

### 2. mc-49
Reproduced on the first start: `round.sh start xp+mc` gives `NotQuiet: still busy after 10 s: mc:write:mc-s-agenda-panels-today`, and state.get stays `["write:mc-s-agenda-panels-today"]` for 30 s and more (no rewrite loop; the entry is never removed). Cause: `Mc::build` (a build after the second look joined and the output changed) removes the node `mc` with the panel whose write animation runs; the core ends no animation of a removed node, `animation.done` never comes, and the `writing` entry stays. Fix: `note_writes` forgets the entries of every node that a Remove takes away (the build passes its removal through it). Red test c3432638 fails with the busy word of the capture. After: `round.sh start xp+mc` 3 of 3 with `check.py start` quiet (binaries of the fix, 55f75baa).

### 3. mc-48
Replay of the sofa actions 1 to 18 on mc (replay-sofa: look, Right, Super+space, click, type Foot, Return, XF86Back x2, XF86HomePage, Super+a), nodes read every 150 ms after Super+a:
- before the fix: the group of the old set `mc-regions-1` goes 1.0 -> 0.2 -> hidden between 0.2 s and 0.6 s while `mc-regions-0` fades in (0 -> 1); both are shown for 0.2 to 0.6 s; after that the old set is hidden, not stale, and there is no second draw. The old set is meant to stay under the root hidden until the next switch.
- A check at +0.55 s (the sofa breach: 0.548 s after the key, on a loaded machine with two sessions) reads both sets: I5 17 times with equal rects. mc did not say busy, so `wait_quiet` returned at once. That is a misread of a transient state that mc should have called busy.
- Fix: every fade of opacity that mc starts is a busy word `fade:<node>` in state.get until `animation.done` of the node, at most its time and 2 s (a removed node forgets it too), so the harness waits. After the fix, same replay: busy `['fade:mc-r0-right', 'fade:mc-regions-1']` from 0.4 to 0.56 s, `[]` from 0.71 s, no breaches.jsonl (mc-48-busy-after-fix.txt). Not changed in I5: a set that stays visible after the fade would still breach.

### 4. P2-CHECK (tests/persona, b4104cae)
- M1 passes when the page differed from the start at the end or at any look of the task (the observation now holds the page: the shown scene groups of mc with opacity of at least 0.5); the windows of mc count as before.
- T6 passes on the launcher row `Nothing found for ...`: session.sh puts a `fuzzel` stand-in first in the PATH that logs `fuzzel-first-row <row>` in calls.log and runs the real fuzzel with the same rows.
- T1 and T4 read `open` besides the key list of the look (`expected open []` for anything else).
- The wpctl stand-in keeps its volume in a file: get-volume answers the value after set-volume `N%+`, `N%-` or a number (clamped to 0 to 1).
- Tests: test_acts.py (t1 and t4 key list, t6 row, m1 turned and came back, m1 fade); persona-selftest.sh (wpctl 0.55 after 5%+, 0.45 after 10%-, the first row logged). `python3 -I tests/test_acts.py` 140 ok, `tests/persona-selftest.sh` ok.

### Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,keys,chat-keys,doors,mc-paths,mc-sofa,mc-handover,layout`: all 12 PASS (build 111 s, test 242 s, clippy 46 s, ci-check 134 s, layout 201 s, doors 357 s, mc-paths 80 s, mc-sofa 43 s, mc-handover 12 s, keys 20 s, chat-keys 9 s, pre-commit 6 s). The step `launcher-click` of the brief does not exist and was left out. It is a partial run: nothing was pushed by it.

Not touched: mc-50, the M2 ruling.

## Review r1

Reviewer lane (cloud, read-only on code). claude/mc-fix b4104cae against dac835dd (8 files, +270 -14). Step 0: `cargo check --workspace` green. Verdict: **MERGE-OK** (0 BLOCK, 0 MAJOR, 5 MINOR/NIT).

Checks:
1. Red first, `cargo test -p chora-mc` per sha in a scratch worktree: `a_build_in_the_midst_of_a_write_leaves_mc_quiet` FAILS at c3432638 (left `["write:mc-s-agenda-panels-today"]`, right `[]`) and passes at 55f75baa. `a_cross_fade_keeps_mc_busy_until_its_end` FAILS at 137437c5 ("the cross-fade of the page is not busy") and passes at bb3de644. The test does not exist at 55f75baa (it is added by 137437c5, after it), so "fails at 55f75baa" means: fails at its own red commit, which sits on the fix of mc-49.
2. mc-49: `note_writes` runs inside `apply` (every command path through it) and in `build` for the unbuild list; a Remove forgets the node and every id under `<id>-`. Rename goes through `build()` (Remove `mc`), so a rename or rebuild leaves no entry. A removal of node A does not clear a write of an unrelated node B (but see rv-1 for sibling ids that share a prefix). The batch number stays unique (`max+1` over the entries left).
3. mc-48: the core keeps one animation per node (`Scene::animations` keyed by id), so a second fade on a node replaces the first: `fading.insert` takes the new deadline and the one `animation.done` of the replacing animation removes it. A Remove mid-fade forgets it. When the core ends no animation (an Update that cancels it) the entry lapses at duration + 2 s (`FADE_GRACE`) and `state()` filters by deadline. No thread or timer: nothing to leak on exit.
4. P2-CHECK: M1 still fails when nothing changed (test `m1 ... nothing changed`), and `hist` is read since `start.json`, so only the task counts. T6 reads a logged row of the real launcher. T1/T4 loosening: see rv-3. wpctl stand-in: values 0.55 and 0.45 checked by persona-selftest. Each change has a test (6 in test_acts.py, 3 lines in persona-selftest.sh).
5. mc-47: the evidence (mc-47-fuzzel-events.log: `wl_pointer.enter`, `motion`, `button` 272 press and release reach fuzzel at the row, nothing printed; fuzzel 1.9.2 here) supports "the core delivers the click, fuzzel 1.9.2 does not act on it", with the limit that the log shows delivery, not that the pointer sat on a row (176,61 in surface coordinates is not mapped to a row). Verdict: closing as "not a bug of mc" is right for this container. The mc-sofa cell (patch reports/mc-fix/mc-47-cell.patch, ddd43ea3) should come back once fuzzel >= 1.11 is on PATH: put it back as `launcher-click-row`, reusing the xp/tiling launcher-click helper, with a version guard (`need fuzzel >= 1.11`, a skip below it, not a red) so a lane container with 1.9.2 stays green. The PM runs it on the train with 1.11.1; if it is red there, mc-47 is open again.
6. Runs: `cargo test -p chora-mc` 174 ok; `python3 -I -m pytest tests/test_acts.py -q` 140 passed; `bash tests/persona-selftest.sh` ok (after `cargo build --workspace`); `cargo clippy --workspace --all-targets -- -D warnings` clean; `tools/size-ratchet --base dac835dd` ok, 0 files over 1500 production lines (main.rs is 2708 lines of which +4 production; tests do not count, it is a listed file that did not grow).

Findings:
- rv-1, MINOR, plugins/chora-mc/src/lifecycle.rs:130: the Remove forgets every id that starts with `<id>-`, which also takes a sibling that shares the prefix (scene `foot` removed while scene `foot-server`, id `mc-s-foot-server-panels-..`, writes or fades; panel `today` and `today-x`), so busy clears early for the sibling; fix: match `node == id` or the shapes of the real children (`<id>-panels-`, `<id>-page`) or ask `scene.query` for the descendants, and add a test with two scenes `a` and `a-b`.
- rv-2, MINOR, plugins/chora-mc/src/build.rs:221 (and apply_scenes, build.rs:~43): the 600 ms theme fades and the nodes of a build go through `rt.conn.try_apply`, not `apply`, so they are never in `fading`/`writing`; a check after a theme change can still misread a transient state as mc-48 did; fix: call `note_writes` on `fades` and on the `all` of `apply_scenes` once they are accepted.
- rv-3, MINOR, tests/persona/check.py:364 (`verify_only_keys_open`): T1/T4 now accept the key list open whatever put it there, the ruling says the one the persona opened itself; a list that opens unasked passes; fix: allow the sheet only when `ctx.opened(sheet)` or the chord list holds the key that opens it, and add a test of an unasked sheet that stays red.
- rv-4, NIT, plugins/chora-mc/src/main.rs:2420: the tests cover set and end of a fade only; a second fade mid-fade, a Remove mid-fade and the `FADE_GRACE` lapse are not tested (behaviour read as right in check 3); fix: one test each with `mc.event(AnimationDone)` skipped and the clock moved (an `Instant` in `fading` set in the past).
- rv-5, NIT, tests/persona/session.sh:161: the fuzzel stand-in prints one empty row when stdin is empty (`printf '%s\n' "$first"` after a failed `read`), real fuzzel shows none; fix: `[ -n "$first" ] && printf ...` (test with `fuzzel --dmenu </dev/null`).

Not findings: gate-tiling-apps-click and launcher-click red in lane containers (ti-1).

## r2

Lane: cloud, branch claude/mc-fix. Step 0: `git fetch --unshallow`, fetch of origin/claude/autonomous-dev-session-3w-i33pg7 (9973d0eb) and `cargo check --workspace` green. Merge commit **cb2b80f1** (b4104cae + 9973d0eb), pushed to claude/mc-fix. Nothing else changed beyond the hunks below and two test calls.

### Conflict hunks (tests/persona/check.py, tests/persona/session.sh)
1. check.py, snapshot row: both fields kept, `"page": page_of(snap, look)` and `"toplevels": sorted(snap.toplevels)`.
2. check.py, T1: `Expect(open=ctx.left, verify=verify_all(verify_only_keys_open(ctx), verify_new_window(ctx, before)))`. xp-p2r3's `open=ctx.left` and mc-fix's key-list check both stay.
3. check.py, T4: the same shape, `open=ctx.left` and `verify_all(verify_only_keys_open(ctx), verify_system(ctx, part))`.
4. check.py, T2: per xp-p2r3, `Expect(open=ctx.left, verify=verify_sheet(ctx))` (mc-fix had `open=[]`). No test of mc-fix says otherwise: `test_t2_key_list_opened_spoke_and_shut` passes with it. T3 is the same on both sides.
5. check.py, M1: mc-fix's `verify_page_changed(ctx, before)` (xp-p2r3's `verify_changed(before, shown_scene)` dropped; `page_of` and `shown_scene` both stay, as the page check uses both).
6. check.py, M2: `Expect(open=ctx.left, focus="mc")` (xp-p2r3).
7. session.sh, wpctl: one stand-in, xp-p2r3's version (state file `wpctl.state` with volume and mute: get-volume with `[MUTED]`, set-mute 1|0|toggle, set-volume N%+ / N%- / N%, clamped 0 to 1, logged to calls.log through @MARKS@ and the sed). mc-fix's own `volume` file version is dropped; its behaviour (set value kept, 0.55 after 5%+, 0.45 after 10%-) is covered by the same stand-in. mc-fix's `fuzzel` stand-in (T6 first row) is kept right after it unchanged.

One change outside the hunks: two mc-fix tests (`test_t1_the_key_list_of_the_persona_may_stay_open`, `test_t4_the_key_list_of_the_persona_may_stay_open`) built `ctx()` without `left`, which under `open=ctx.left` reads as "nothing may stay open"; both now pass `left=["xp-dlg-help"]` (the T4 menu check keeps `left` empty, still red).

### Tests
- `python3 -I -m pytest tests/test_acts.py -q`: 148 passed, 3 subtests (the two above failed right after the merge, fixed as written).
- `bash tests/persona-selftest.sh` (after `cargo build --workspace`): ok. The first run failed once at "no click on Notepad" (jq: null divided by number, the Start menu node had no rect yet); the rerun passed with nothing changed in between. Flaky timing in the Start menu click helper, not caused by the merge as far as I can tell; not investigated further.

### Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,mc-paths,mc-sofa,mc-handover` (run dir /tmp/merge-gate.8hyEBj): all 8 PASS (pre-commit 4 s, ci-check 126 s, build 40 s, test 308 s, clippy 26 s, mc-sofa 46 s, mc-handover 13 s, mc-paths 81 s). Partial run, nothing pushed by it.

Not touched: rv-1..rv-5 (backlog MC-FIX-RV), mc-50.
