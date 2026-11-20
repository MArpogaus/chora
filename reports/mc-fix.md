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

## PM review r2 (merge cb2b80f1, 2026-11-20 09:15 UTC)
Checked `git show --cc cb2b80f1` for tests/persona/check.py and session.sh against the brief:
- row keeps `page` (mc-fix) and `toplevels` (head): ok.
- T1, T4: open=ctx.left plus verify_only_keys_open; T2, M2 open=ctx.left (head intent); M1 verify_page_changed; C2 verify_no_window_lost: ok.
- wpctl: head's state file (volume, mute) logs to @MARKS@ = $fakes/calls.log, the same file mc-fix's fuzzel stand-in writes ($here/calls.log): no lost log.
- merge-tree vs t87 shas: clean except the known merge-gate.sh comment conflict with co-r4 42c9304b.
Verdict: MERGE-OK (0 BLOCK, 0 MAJOR); cb2b80f1 joins train 87. Persona-selftest one red rerun green ("no click on Notepad"): watched by flake series.

## r3 (RC 2 persona round 4, mc-42, mc-46, I3 and I5 on xp+mc)
Lane: cloud, branch claude/mc-fix, from cb2b80f1. The session head had not moved past adf71956, so nothing was merged. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. Pushed b0997b71 to claude/mc-fix. Each fix has a red commit before it.

### 1. mc-42 (BLOCK reopened): the page that a plain Right starts
- Cause: the Foot page of the cook is the terminal scene, and its program starts with the turn. The plain turn kept the keyboard on the page, then the window came (adopt, not a plain turn) and took it, so Right and Escape reached the shell. terminal-back only reached a window that was already there.
- Red 34270632: gate-mc-paths cell `terminal-turn` (Home, Right x4 to the terminal page, a fresh start each time, then Left to mission, Right to overview, Escape to mission, XF86Back then Left; fails at the first key, the page stays `terminal`).
- Fix 03cff6d2: `plain_start` in show.rs/windows.rs: a window that comes for a scene that a plain turn showed and started is adopted as after a plain turn. Super+Return and the launcher still give the window the keyboard (cells terminal-keys and the launcher start of remote-back stay green). The rule of docs/configuration.md (a page reached with a plain key keeps the keyboard) now holds for a turn that starts the program.
- Same key rule for both arrivals: a plain-key arrival keeps the keyboard on the page, a launcher or Super arrival gives the window the keyboard and XF86Back (first press) gives it back to the page.
- Capture: none new (the cell reads texts); r3 evidence p2r3-cook 007, 008.

### 2. mc-46 (partly): the first XF86Back
- Not reproduced as "does nothing": with the program started by the launcher, Escape, then one XF86Back, the keyboard did move (Left then turned the page; sofa 009 shows the terminal cursor hollow). What the persona could not see: the key list on a window page always drew the Super form (`Super+ <- ->`), whoever held the keyboard.
- Red 9eff365f: gate-mc-sofa `remote-back` starts the window through the launcher (fake fuzzel `pick` file), presses Escape, asserts the key list names Super, presses XF86Back once, asserts the plain arrows.
- Fix 45fcc3f1: `page_keys_work` (the page of a window uses the plain form while the page node holds the keyboard) and `sync_keys`, which draws the list anew when the holder changes (focus event, XF86Back, Return). docs/configuration.md says it. terminal-turn also asserts the plain arrows. mc-50 loses the first-press half; the yellow square stays.

### 3. I3 and I5 on xp+mc: verified with a live xp+mc session (state dump)
State of the unchanged build, no action of a persona: `choractl state`: xp `{"focus":{"holder":"xp"}}`, mc `{"focus":{"holder":"mc"}}`; the one focused node of the core: `xp-icons` (owner xp). scene.boxes of HEADLESS-1: mc nodes are the boxes 0 to 56 (mc-bg first), `xp-wall-HEADLESS-1` is box 57 (image, `opaque` false, rect [0,0,1280,800]), the xp panel and the pager 58 to 69. The capture shows xp only.
- **I5 is the checker.** mc is wholly under the wallpaper of xp, so no mc text is on the screen; the checker counted it as shown because a picture is never opaque for I5. Red 51642134 (three cells in tests/test_acts.py with the coordinates of the breach: a look under the wallpaper passes; the same texts above the wallpaper still fail; a part picture such as the panel still hides nothing). Fix dab46f6b: a picture of the whole output covers the nodes below it. No loosening for the shown look (the wallpaper is box 57, the shown look's texts are above it). Live: no I5 line after the fix. Ledger mc-51.
- **I3 is mc.** mc's page node had the keyboard at the start; xp then took it with `xp-icons`, and the core sends no `focus.changed` for a node (rw-3), so mc kept saying `holder: mc`. A real lie of `state.get`, not a draw problem. Red 08ae4d56 (unit test `state_says_nobody_when_a_node_of_another_client_has_the_keyboard`), prep 18247e0d (`Plugin::refresh`, runs before `state.get` answers; default does nothing; `Focus::node`, `Focus::drop_node`), fix b0997b71 (mc asks the core which nodes are focused and drops its node when its own is not among them). Live after the fix: xp `{"holder":"xp"}`, mc `{"holder":null}`, no I3 or I5 line from `check.py after`. Ledger mc-52. xp and tiling have the same tracker and keep the old answer until they implement `refresh` (backlog, no breach seen there).
- Nurse T7 stays "not reached": the way to the other look is not shown on xp (ux row of p2r3-nurse, not in scope).

### Gate
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,chat-keys,doors,mc-paths,mc-sofa,mc-handover,matrix,layout` (run dir /tmp/merge-gate.oCVcby): all 13 PASS (pre-commit 5 s, ci-check 96 s, build 28 s, test 237 s, clippy 17 s, chat-keys 9 s, layout 213 s, keys 24 s, matrix 333 s, mc-paths 109 s, mc-handover 14 s, mc-sofa 49 s, doors 375 s). Also `python3 tests/test_acts.py`: 151 OK, `cargo test -p chora-mc`: 175 OK. A red cell I did not cause: none. Partial run, nothing pushed by it.

### Commits (author and committer Marcel Arpogaus, no attribution lines)
| item | red | fix | test / cell |
|---|---|---|---|
| mc-42 | 34270632 | 03cff6d2 | gate-mc-paths `terminal-turn` |
| mc-46 | 9eff365f | 45fcc3f1 | gate-mc-sofa `remote-back` |
| I5 (checker) | 51642134 | dab46f6b | tests/test_acts.py I5 (3 cases) |
| I3 | 08ae4d56 (prep 18247e0d) | b0997b71 | mc unit test, live xp+mc state |

Not touched: T1 checker end-time gap, fuzzel 1.9.2 launcher click, mc-50 yellow square.

## Review r3 (seeded, claude/mc-fix b0997b71 against cb2b80f1)
Lane: cloud, read only on code, one round. Step 0: `git fetch --unshallow`, `cargo check --workspace` green.
**Verdict: FIX** (1 MAJOR open). Counts: 0 BLOCK, 1 MAJOR, 2 MINOR, 2 NIT.

Checks:
1. Red first holds for all four pairs (scratch worktree, headless): gate-mc-paths `terminal-turn` FAIL at 34270632 ("Left on the terminal page reached by Right does not go to mission"), ok at 03cff6d2; gate-mc-sofa `remote-back` FAIL at 9eff365f ("after one XF86Back the key list does not name the plain arrows: Super+ ← →"), ok at 45fcc3f1; tests/test_acts.py 1 FAIL at 51642134 (`test_a_look_under_the_wallpaper_of_another_is_not_on_the_screen`), 151 OK at dab46f6b; `state_says_nobody_when_a_node_of_another_client_has_the_keyboard` FAIL at 08ae4d56, ok at b0997b71. The two other new I5 cases pass without the change by design (guards against loosening), so not every new case is red.
2. Kit `refresh`: no deadlock and no recursion. The core routes `methods.call` asynchronously (`method.called` event, `methods.return`; methods.rs), so it answers mc's `scene.query` while choractl waits; a `method.called` that arrives during the query waits in the event queue, so `state.get` does not run again inside `refresh`. The default no-op leaves xp and tiling as before (only mc overrides). An error of the query is swallowed (rv-5).
3. I3: mc cannot drop a node that it just focused: `scene.apply` answers after the transaction applies (transaction.rs), and ids are not namespaced (own_node by owner). A rejected focus is now corrected too. After a drop the holder falls back to the core surface, which `keyboard_to` set to none, so `state.get` says null, not a stale window. The next key goes where the core focuses; mc's next `to_node` sets the node again.
4. mc-42: page turn and launcher arrivals act as the report says (cells green), but a turn away and back while the program starts still gives the window the keyboard (rv-1).
5. mc-46: the first XF86Back acts (the red cell fails only on the key list; Left then turns the page), and the list now follows the holder.
6. I5: the whole-output-image rule fixes the xp+mc count, and texts above the wallpaper still fail; it also covers translucent or own full-output images (rv-3).
7. `cargo test -p chora-kit -p chora-mc` green (17 result lines, 0 failed), `python3 -m pytest tests/test_acts.py -q` 151 passed, `cargo clippy --workspace --all-targets -- -D warnings` clean, `tools/size-ratchet --base cb2b80f1` ok (0 files over 1500 production lines; mc main.rs is 2767 lines with its tests).

Findings:
- rv-1, MAJOR, plugins/chora-mc/src/show.rs:149, mc-42 still happens when the person turns back to a page while its program starts: Right to the terminal page (starts foot), Left, Right before the window maps; `show` clears `plain_start` and, because `waiting` is true, does not set it again, so the window that comes takes the keyboard (scratch unit test: last focus `s1`, holder surface `s1`; with `plain_start` set: page node, holder mc); also windows.rs:174 `take()` lets any other window adopted in between clear it. Fix: set `plain_start` whenever `user && plain_turn && app.is_some() && window.is_none()` (also while waiting), and take it only when the name matches (checked in scratch: the page keeps the keyboard, all 176 mc tests green).
- rv-2, MINOR, plugins/chora-mc/src/main.rs:649, `refresh` queries selector `*` with the focused filter on every `state.get` (the persona checker calls it after every act); the core computes `screen_rects` for every matched node before it applies the filter (crates/chora/src/ipc/query.rs:25), text layouts and projections of the whole scene on the main thread. Fix: query `id:<own node>` with `focused: true`.
- rv-3, MINOR, tests/acts/invariants.py:389, `hides` takes any image of the whole output as a cover, whatever its owner and opacity (boxes carry no opacity, images are never `opaque`), so a wallpaper that fades in over a look, or a translucent full-output picture of mc itself, hides a real overlap. Fix: count it as a cover only when its owner differs from both texts' looks (and ask the core to report opacity in `scene.boxes` later).
- rv-4, NIT, plugins/chora-mc/src/main.rs:2125, the I3 unit test covers only the drop: the FakeCore ignores `filter.focused` and returns nothing for `*` (crates/chora-kit/src/testing.rs:349), so a `refresh` that always dropped would pass; the keep case (own node focused, holder mc) is untested. Fix: let the FakeCore record `FocusNode` and honour `focused`, and add the keep case.
- rv-5, NIT, plugins/chora-mc/src/main.rs:657, `refresh` changes `focus` only when somebody calls `state.get`, so mc's holder (and `page_keys_work`) depends on being watched, and a failed query is silent. Fix: log the error, and name the backlog (rw-3, node focus events) in the doc comment.

Known, not findings: launcher click red in lane containers (fuzzel 1.9.2), mc-50 yellow square, T1 end-time gap.

## r4 (RC 2 persona round 4, review r3 rv-1 to rv-5)
Lane: cloud, branch claude/mc-fix from b0997b71. The session head was still d81c8293 (already in the branch), so nothing was merged. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. Pushed a8cd0de0 to claude/mc-fix. Each fix has a red commit before it.

| item | red | fix | test / cell |
|---|---|---|---|
| rv-1 MAJOR mc-42 | ad1f2ded (unit test; cell variant a8cd0de0) | 0b72ff84 | mc unit `a_turn_away_and_back_while_the_program_starts_keeps_the_keyboard_on_the_page`; gate-mc-paths `terminal-turn` |
| rv-2 MINOR | ee8b0ada | 75272909 | mc unit `refresh_asks_the_core_for_its_own_node_only` |
| rv-3 MINOR | 5c422e10 | 6508d95d | tests/test_acts.py I5 (2 new cases) |
| rv-4 NIT | ee8b0ada (keep case; FakeCore records `selectors`, honours `filter.focused`) | 75272909 | mc unit `state_says_mc_when_its_node_still_has_the_keyboard`, `state_says_nobody_...` now has xp take the keyboard |
| rv-5 NIT | none (log line, doc comment) | 75272909 | none |

- rv-1: `show` sets `plain_start` whenever `user && plain_turn && app && no window`, also while the program waits; `adopt` takes it only for its own scene (`take_plain_start`), so a window of another scene in between leaves it. Red cell: the first cell I wrote (Right, Left, Right with 0.4 s between keys) did NOT fail at ad1f2ded (the window mapped first); a8cd0de0 sends the three keys in one inject, which fails at ad1f2ded ("Left after Right, Left, Right before the window mapped does not go to mission", twice) and passes at 0b72ff84 and on the head. So the cell is red only from a8cd0de0, which is after the fix in the history; the unit test is red at ad1f2ded.
- rv-2: `refresh` asks `id:<own node>` with the focused filter; the unit test counts the selectors the FakeCore got.
- rv-3: the rule of the review ("owner differs from both texts' looks") would break the r3 guard (xp wallpaper over an mc text, xp text above), so the cover is a whole-output picture that belongs to the look of the upper text and not to the look of the lower one. A translucent picture of mc between two xp texts, or of mc between two mc texts, stays a breach. Limit: opacity is still not in `scene.boxes`.
- rv-5: a failed `refresh` query is logged; the doc comment names the backlog (rw-3, node focus events).

### Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,gate-lib,ci-check,build,test,clippy,keys,mc-paths,mc-sofa,mc-handover,acts` (run dir /tmp/merge-gate.ztZoxP, at 6508d95d): all 11 PASS (pre-commit 4 s, gate-lib 71 s, ci-check 119 s, build 157 s, test 228 s, clippy 17 s, keys 23 s, mc-paths 111 s, mc-sofa 45 s, mc-handover 13 s, acts 263 s). The gate ran before a8cd0de0 (cell only); `CELLS=terminal-turn tests/gate-mc-paths.sh` passes on a8cd0de0. A red cell I did not cause: none. Partial run, nothing pushed by it.
