# co-focus

## r1
Lane: cloud, claude/co-focus from 86e619f3. Step 0 green: unshallow, `cargo check --workspace`, `cargo build --workspace`. Shas: **red 9b0466ae** (test: keys on hidden faces and after a closed window), **fix 866382dd** (fix: the keys stay on the front face of a hosted cube), head 866382dd. No core change. xp change is outside "focus.node only": the cause lines are in the xp focus bookkeeping (keys.rs, windows.rs, dispatch.rs), and no construct-only fix can make xp's report right (xp names its stale window as the holder after any focus the construct gives).

### Step 1: reproduction (scripted, no persona)
Script: one construct session per run via `round.sh start` (PERSONA_CHECK=1, the persona's own act.py commands and coordinates of feedback/2026-11-23-p2r6co-*.md): row Terminal, Super+c; Start (30,490), Terminal (40,322), Super+Return -> `check.py end C3 T1`; pill (741,33), entry, `hello`, Return, 6 s -> `end T3`; drag foot title onto pager 2 (678,490), pager 1, pager 2, key x -> `end C2`; Super+l, Start, Lock (30,471), 9 s -> `end T4:lock` and a state dump.
| Finding | 86e619f3 (old check.py) | Dump |
|---|---|---|
| co-48 | **5/5** C3 FAIL "expected the focus with construct, holders ['xp']", T1 FAIL "no new focused window of construct; new: ['s2']"; Super+Return: nothing | construct `{'holder': None}`; xp `{'holder': 'xp', 'surface': 's2'}` s2 shown on VIRTUAL-1 (front 0); core focused `s2` (surface of xp). One owner, a shown window: consistent |
| co-49 | **5/5** T3 FAIL "the focus is with None, not back with construct" (chat shows hello / Hello) | construct `{'holder': None}`; xp holds `s2` shown; core focused `s2`; 0 I3 rows |
| co-50 | **5/5**, I3 x3 per run ("the focused s2 is no shown window of xp" in click, look, key) | after pager 2: front 1 (VIRTUAL-2); xp `{'holder': 'xp', 'surface': 's2'}` s2 on VIRTUAL-1 shown false; core focused `s2`: the key x and Super+l went to the hidden s2 |
| co-51 | keys nowhere **5/5** (4 runs with the dump line, 1 by hand); the demo's own rows (I2 "no box", I3 "xp says it holds sN, the core focuses nothing" on the lock window) **0/5** | after the lock window (`Screen locked`, 8 s) closed: construct `{'holder': 'construct'}`, xp `{'holder': None}`, core focused `[]`. Trace: `focus.changed null` then `surface.destroyed s9`, no focus after. During the lock window xp held it and the core focused it 0.15 s after `surface.created` (the demo rows are likely that gap; not seen) |

### Step 2: severity
- co-48 **contract fix, not product**: in use mode "the face: pointer and keys reach the window manager and its windows" and "Each face is a virtual output with its own window manager and taskbar" (docs/cube-modes.md, Persons and goal and States). The terminal from Start is xp's, shown, focused by the core and held by xp alone. Super+Return is no key of xp on a face (the terminal is a row of its Start menu, plugins/chora-xp/src/config.rs `acts_in_xp`) nor of the construct (Super+t, which Max T5 used): a UX matter for triage, no lost key (it reached the focused foot).
- co-49 **contract fix, not product**, same cause: `verify_helper` read only the construct's holder; xp holds the shown s2 after the answer.
- co-50 **MAJOR**: "a key reaching a hidden window": the key x after the pager click reaches s2 of face 0 (state dump above).
- co-51 **MAJOR**: "a key going nowhere after a user act": after Lock and the lock window closing the core focuses nothing and the construct claims the focus with no node. The I2/I3 rows of the demo stay open as verify (0/5).

### Step 3: causes (86e619f3)
- co-50: `plugins/chora-xp/src/wm/dispatch.rs:97` xp hears the event `construct.cube` of its host only after the host left (`if self.left.is_some()`), so a turn (the pager goes through `hosted.rs` `go_to_desk` -> `cube.turn`) changes the front face and xp keeps the keys on the window of the face that went out of sight; `windows.rs:179` `topmost_except` also picks windows of any face.
- co-51: `plugins/chora-xp/src/wm/keys.rs:372` `focus.changed null` (which the core sends for a closing window before `surface.destroyed`) takes `focused`, so `windows.rs:251` `forget` sees no active window and gives the keys to nobody; `plugins/chora-construct/src/state.rs:69` then says the construct holds with nothing focused (I3 silent).
- co-48, co-49 (one cause): `tests/persona/check.py:686` C3 `Expect(focus=ctx.look)`, `:336` T1 owner == look, `:369` T3 holder of the look: none takes the manager of the front face.
- co-50 and co-51 are two causes in one place: xp's focus on hosted faces follows neither the front face nor the core's drop.

### Step 4: red and green
Red 9b0466ae (on 86e619f3):
- `cargo test -p chora-xp hosted`: `a_turn_of_the_host_takes_the_keys_to_the_front_face` "s0 of face 0 keeps the keys: []"; `the_keys_stay_on_the_front_face_when_its_focused_window_closes` "the keys go nowhere: [Remove { id: \"xp-win-s9\", to: None }]".
- `tests/gate-construct-focus.sh` (new, in the construct-extra cell): "FAIL: the cube shows face 1 and s1 of face 0, which does not show, still holds the keys: core: s1 xp: xp s1 construct: {'holder': None}".
- `python3 -I tests/persona/test_check.py FaceManager`: 3 FAIL with the persona lines ("expected the focus with construct, holders ['xp']", "no new focused window of construct; new: ['s2']", "the focus is with None, not back with construct").
Fix 866382dd: xp hears the turn of its host; at rest on another face, keys it holds on a window of a face out of sight go to the topmost window of the front face, else its icons (`kit::wm::focus` give/release through `Wm::focus`); keys of another client stay (unit test). A window whose focus the core dropped as it closed passes the keys on like a focused one; the next window is one of the front face. check.py: `flavor_looks` takes the face manager in use mode for C3, T1, T3; a hidden window or navigate mode stays red (2 unit tests). No sleep, no known row, no raised bar.
Green: xp 331/331 (`cargo test -p chora-xp`), test_check.py 53/53; gate step lines "ok: a turn takes the keys from the window of a face that does not show (core: xp-icons xp: xp - construct: {'holder': None})", "ok: the keys stay with the front face after its focused window closed (core: xp-icons ...)".

### Step 5: counts (866382dd)
- Scripted persona steps: **10/10** each: C3 pass, T1 pass, T3 pass, C2 pass with **0 I3** (co-50), T4:lock pass with 0 I3 and the core on `xp-icons` held by xp after the lock window (co-51). Old binaries: 5/5 red as in step 1.
- `tests/gate-construct-focus.sh` **10/10**; `tests/gate-construct-i3.sh` **10/10** (co-43, co-46 hold: one holder, no key to the hidden pane).

### Gate
`tests/merge-gate.sh --jobs 2` (full, no --only) on claude/co-focus merged with origin/claude/autonomous-dev-session-3w-i33pg7 (still 86e619f3, so the merge is 866382dd itself), detached: **EXIT 0, 60 cells PASS, 0 red** ("all steps passed"); construct-extra (with gate-construct-focus and gate-construct-i3) PASS 164 s. `pgrep -fa 'chora --headless'` empty after.

### Open
- co-51 demo rows (I2 "xp shows sN on VIRTUAL-2, which has no box", I3 "xp says it holds sN, the core focuses nothing" on the lock window while it maps): not reproduced in 5 runs; stays open as verify (likely the 0.15 s between `surface.created` and the core's focus of the new window).
- Super+Return on a face (co-48 text): no key of xp or of the construct there; a UX item for triage, not a focus fault.

## Review r1
Lane: cloud, read-only, claude/co-focus head 866382dd against base 86e619f3 (10 files, +358 -10), one round. Step 0 green: `git fetch --unshallow`, `cargo check --workspace`. No merge gate (the lane's gate: 60 PASS).
**Verdict: FIX** (0 BLOCK, 1 MAJOR, 4 MINOR, 1 NIT).

### Checks
1) Red first: holds. 9b0466ae has the unit tests, gate-construct-focus.sh, the test_check.py cases, and a no-op `host_turned` stub so the tests compile. `cargo test -p chora-xp hosted` at 9b0466ae: 4 ok, 2 FAILED (`a_turn_of_the_host_takes_the_keys_to_the_front_face`, `the_keys_stay_on_the_front_face_when_its_focused_window_closes`). `test_check.py FaceManager` there: 3 FAIL of 5. At 866382dd: 6/6 ok.
2) co-50: holds. `host_turned` only acts when the cube is at rest. Keys on a window of a face out of sight go to the topmost window of the front face, or to the icons, through `Wm::focus` (`kit::wm::focus` give and release). Keys that another client holds are not taken (`node_away`; unit test). Navigate mode is unchanged: the construct's look node has the keys, so the turn returns early. During a turn the leaving window keeps the keys (rcf-6, older than this diff).
3) co-51: holds for windows. A `focus.changed null` and then `surface.destroyed` passes the keys on to the front face (`dropped`). The lock stays above: while locked the core sets the keyboard to the lock surface whatever focus xp gives (crates/chora/src/state.rs:1180-1188). The escape hatch is untouched. A hole for xp's own nodes is rcf-3.
4) Contract: C3 and T3 now fail a hidden window and navigate mode (probe and unit tests). T1 passes a hidden window (rcf-1). C3 and T3 also pass a manager that holds no surface (rcf-2). I2 and I3 are unchanged (tests/acts/invariants.py not in the diff). Other contracts still use `Expect(focus=look)` with the old meaning. `with_children` (check.py:122) ties `shown` to the front face, so the shown test of C3 and T3 is a front-face test.
5) `tools/size-ratchet` ok (0 files over 1500; hosted.rs is 520 lines). `tools/cut-check` ok (8 crates; the reported lines are the existing ones). No code copied from `kit::wm::focus`; one rule is restated (rcf-5).
6) `cargo test -p chora-xp -p chora-construct`: 331 + 268 ok (rebuilt after a fresh touch). `python3 -I tests/persona/test_check.py`: 53 OK.

### Findings
- rcf-1 MAJOR tests/persona/check.py:377 (`verify_new_window`): T1 accepts any new focused window of a face manager in use mode, shown or not. Probe: `face_snap(shown=False)` gives T1 `[]`, against "a hidden window must still fail". Only C3 has a hidden-window test. I3 would still flag the focused hidden window in the same run. Fix: for an owner other than the look, require the new window to be shown in the owner's state, and add a T1 hidden-window unit test.
- rcf-2 MINOR tests/persona/check.py:354 (`focus_with_flavor`): `sid is None` lets a face manager that holds no surface pass C3 and T3 (probe: `[]` for both). That covers any xp node on any face, beyond "only for a shown window". I3 also passes a focused xp node, so a Start menu on a hidden face that holds the keys passes everything. Fix: keep `sid is None` for the look only, or require the focused node on the front face, and add a test.
- rcf-3 MINOR plugins/chora-xp/src/wm/hosted.rs:146-153 (`host_turned`): keys on an own node of xp (Start menu, context menu, Help) stay on the face that turned away when the front face has no window (`next.is_none() && window.is_none()` returns). When the front face has a window, the menu stays open out of sight. Reachable with Ctrl+Alt+Right while the Start menu is open. Fix: on a turn at rest, close xp's menus on the other faces and give the keys to the icons when the front face has no window.
- rcf-4 MINOR plugins/chora-xp/src/wm/hosted.rs:142 with :64 (`find_host`) and :194 (`host_back`): `front` starts at 0 and only a turn event sets it. Two cases go stale: xp restarted by the construct's retry (plugins/chora-construct/src/cube.rs:768 keeps the front), and a host that comes back (its first event goes to `host_back`). Until the next turn, `topmost_except` (windows.rs:180) then picks a window of a hidden face after a close or a minimize; before this diff it took the topmost of any face. Fix: set `front` from `Listing.front` in `find_host` and `host_back`.
- rcf-5 NIT plugins/chora-xp/src/wm/keys.rs:378, windows.rs:269 (`dropped`): restates the kit rule "an empty focus is no loss" (crates/chora-kit/src/wm/focus.rs:116, `Focus::moved`) in xp's own bookkeeping, not through the kit type. Not a copy (cut-check ok). Fix: note it as debt for moving xp to `kit::wm::focus::Focus` (D116).
- rcf-6 MINOR, older than this diff, plugins/chora-xp/src/wm/hosted.rs:139: while the cube turns (`turning`), typed keys reach the focused window of the leaving face. That window still shows on the turning cube, but docs/cube-modes.md States says "turning: nothing: no face takes input", and the construct blocks no keys during a turn either. Fix: a triage item to hold the keys off during a turn, in the construct or in xp.

## r2
Lane: cloud, claude/co-focus 866382dd, merged fresh origin/claude/autonomous-dev-session-3w-i33pg7 (2c1748a4) as 8216d26e. Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. Head **8a599917**.

### Commits
- c0f0f1d3 test (red): test_check.py FaceManager +4 (`face_snap(menu_on=...)`: xp holds its Start menu, drawn on that output in `scene.boxes`), hosted.rs +4 unit tests.
- 8a599917 fix: check.py (`shown_window_of`, `on_front_face`, `focus_with_flavor`, `verify_new_window`), hosted.rs (`host_turned`, `shut_menus_off_front`, `find_host`, `host_back`) and one more unit test (a cube event at rest keeps an open Start menu of the front face with its keys).

### Red at c0f0f1d3
- rcf-1: `FAIL: test_t1_takes_no_new_window_of_a_face_that_does_not_show ... AssertionError: [] == []`
- rcf-2: `FAIL: test_c3_takes_no_node_of_a_face_that_does_not_show ... AssertionError: [] == []`; `FAIL: test_t3_takes_no_node_of_a_face_that_does_not_show ... AssertionError: [] == []` (test_check.py: 57 run, 3 FAIL)
- rcf-3: `a_turn_at_rest_shuts_the_start_menu_of_a_face_out_of_sight ... FAILED: the menu stays open: [Configure { surface: "s0" ...` `left: Some("V-1")`; `a_turn_at_rest_to_a_face_without_windows_takes_the_keys_off_the_menu ... FAILED: the menu stays open: []` `left: Some("V-1")`
- rcf-4: `xp_started_on_a_hosted_cube_reads_its_front_from_the_listing ... FAILED: the front is not the listing's left: 0` (the construct's retry: xp starts on the faces with face 1 at the front); `a_host_that_comes_back_gives_xp_its_front ... FAILED: the front is not the listing's left: 0`. Both then close the focused window of face 1 and assert no `focus::give("s0")` (s0 on face 0).
- `cargo test -p chora-xp hosted`: 6 passed, 4 failed.

### Fix (8a599917)
- rcf-1: T1 takes a new focused window of a face manager only when that manager's state shows it (`with_children` ties shown to the front face); the look's own window as before.
- rcf-2: `focus_with_flavor`: the look passes as before; a manager with a surface needs a shown window; a manager with no surface needs a focused node of its own that draws (itself or a node under it) on the output of the front face in `scene.boxes`. C3 and T3 both read it.
- rcf-3: `host_turned` at rest shuts the Start menu and a context menu that are open off the front face (`close_menu`, `close_context`), then keys on a hidden window or on an own node of xp go to the topmost window of the front face, else the icons, through `Wm::focus` (`kit::wm::focus` give and release, D116). An open menu on the front face keeps its keys. The early return that left the keys on the menu when the front face had no window is gone.
- rcf-4: `find_host` sets `front` from `Listing.front` whenever it takes a host; `host_back` sets it from its listing.
- No sleep, no known row, no raised bar. `cargo fmt`, `cargo clippy -p chora-xp --all-targets`: clean.

### Green and counts
- `cargo test -p chora-xp hosted`: 11 passed. `cargo test -p chora-xp -p chora-construct`: 343 + 268 ok. `python3 -I tests/persona/test_check.py`: 57 OK.
- New tests 10/10 (hosted 11/11 and test_check 57/57 each run).
- gate-construct-focus.sh 10/10 green (`ok: a turn takes the keys from the window of a face that does not show`, `ok: the keys stay with the front face after its focused window closed`).
- gate-construct-i3.sh 10/10 green (co-43, co-46 hold).

### Gate
`tests/merge-gate.sh --jobs 2` (full, detached) on 8a599917 (claude/co-focus with fresh 2c1748a4): **62 PASS, EXIT 0**, "all steps passed". `pgrep -fa 'chora --headless'` empty after.

### New findings (not fixed, for triage)
- rcf-7 MAJOR by the r1 bar, plugins/chora-xp/src/wm/keys.rs:216 (`toggle_start`): Ctrl+Escape or Super alone opens the Start menu on `self.bars.first()`, which on a hosted cube is face 0 (V-1), whatever face is at the front: the menu and the keys go to a face out of sight. Fix: on a hosted cube, the bar of the front face.
- rcf-8 MINOR plugins/chora-xp/src/wm/panels.rs:622 (`refresh_icons`): the desktop icons lie only on the work area of the first bar (face 0). "Keys to the icons of the front face" (co-50, co-51, rcf-3) give the keys to icons out of sight on faces 1..3. The stricter C3 and T3 of rcf-2 now flag that in a persona run (the focused `xp-icons` draws on no box of the front face). Fix: icons per face, or move them to the front face on a turn at rest; a design call.

## Review r2
Lane: cloud, read-only, claude/co-focus head 8a599917, own diff 8216d26e..8a599917 (3 files, +275 -26), one round. Step 0 green: `git fetch --unshallow`, `cargo check --workspace`. No merge gate (the lane's gate: 62 PASS, EXIT 0).
**Verdict: FIX** (0 BLOCK, 1 MAJOR, 0 MINOR, 0 NIT).

### Checks
1) Red first: holds. c0f0f1d3 (tests) comes before 8a599917 (fix). `cargo test -p chora-xp hosted` at c0f0f1d3: 6 passed, 4 FAILED (`a_turn_at_rest_shuts_the_start_menu_of_a_face_out_of_sight`, `a_turn_at_rest_to_a_face_without_windows_takes_the_keys_off_the_menu`, `xp_started_on_a_hosted_cube_reads_its_front_from_the_listing`, `a_host_that_comes_back_gives_xp_its_front`); at 8a599917: 11 passed. `test_check.py` at c0f0f1d3: 57 run, 3 FAIL (T1, C3, T3 "does not show"); at 8a599917: 57 OK.
2) rcf-1, rcf-2: hold. T1 takes a face manager's new window only when `shown_window_of` (the state of the manager, with `with_children` tying shown to the front face's output). `focus_with_flavor` now passes the look; a manager with a surface only for a shown window; a manager without a surface only when its focused non-surface node (or a node under it) has a box on the front face's output in `scene.boxes` (always taken by harness.py:826; no boxes fails closed). Navigate mode still fails (`flavor_looks` gives only the look). Every branch is the same as or stricter than before; I2, I3 and the other looks are not in the diff.
3) rcf-3: holds except for one path, rcf-9. At rest, the Start menu and a context menu off the front face shut. An open menu on the front face keeps its keys (early return). Keys on a hidden window or on an own node go to the front face's topmost window, else the icons, through `Wm::focus` (`kit::wm::focus` give and release, D116). The close and the hand-off go out in one `scene.apply` batch, so no key reaches the hidden window between them. During a turn nothing changes (rcf-6, older). Lock and escape hatch are untouched (the core keeps the lock surface). But keys held by another client can be taken (rcf-9; probe below).
4) rcf-4: holds. `find_host` sets `front` from `Listing.front` each time it takes a host. A failed listing while the host's faces remain counts as `closing` and keeps host and front. `host_back` sets it from its own listing before `after_move(0)`, so `topmost_except` already reads the new front.
5) `tools/size-ratchet` ok (0 files over 1500; hosted.rs 699 lines). `tools/cut-check` ok (8 crates, 18 reported, the existing lines). No copy of kit focus logic; the hand-off goes through `Wm::focus`.
6) `cargo test -p chora-xp -p chora-construct`: 343 + 268 ok. `python3 -I tests/persona/test_check.py`: 57 OK.
Known rcf-7 and rcf-8: this diff does not touch `toggle_start` or `refresh_icons`. Turning away from a face 0 menu now leaves the keys on face 0's icons (rcf-8), where before they stayed on the face 0 menu: still out of sight, not worse.

### Findings
- rcf-9 MAJOR plugins/chora-xp/src/wm/hosted.rs:148 (`host_turned` calls `shut_menus_off_front` before the `node_away` check) with panels.rs:69 (`close_menu` sends `desktop_keys`). Another client holds the keys (the construct's look in navigate mode, or one of its nodes), xp has no focused window, and xp's Start menu is still open on a face that is no longer at the front. A turn at rest then sends `FocusNode { node: Some("xp-icons") }`. xp gets its keys through the main socket, not a client socket, so `focus_node_command` (chora-scene scene.rs:1074) moves the keyboard from the other client to xp's icons on face 0 (rcf-8), possibly out of sight. This breaks "keys that another client holds are not taken". xp does not shut its menu when another client takes the keys (`node_focused`, state.rs:110). Probe: a scratch unit test (`hosted`, `focused = None`, `core_focus = None`, `press_taskbar(Start("V-1"))`, `key_node = Some("construct")`, `host_turned(turned(1))`) gets `[Update xp-menu visible false, FocusNode { node: Some("xp-icons") }, ...]`. Before this diff the same turn sent nothing. Fix: while `node_away()`, shut the menus without any `FocusNode` (or have `desktop_keys` send no node then), and keep that probe as a unit test next to `a_turn_of_the_host_leaves_the_keys_of_another_client`.

## r3
Lane: cloud, claude/co-focus 8a599917, merged fresh origin/claude/autonomous-dev-session-3w-i33pg7 (230ea0e5) as a6c0c020. Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. Head **3a625261**.

### Commits
- 35fa4b8d test (red): hosted.rs +2 unit tests. `a_turn_that_shuts_a_menu_leaves_the_keys_of_another_client` (rcf-9, the review's probe, next to `a_turn_of_the_host_leaves_the_keys_of_another_client`): focused None, core_focus None, Start open on V-1, key_node Some("construct"), `host_turned(turned(1))`: the menu shuts, no `FocusNode { node: Some(_) }`, focused None. `the_start_key_opens_the_menu_on_the_front_face` (rcf-7): hosted, front 2: `key("Ctrl+Escape")` and Super alone (`key` + `key_released("Super+Super_L")`) open the menu on V-3 with `FocusNode xp-startmenu`; host None: on V-1 (bars.first()) as before.
- 3a625261 fix: press.rs `desktop_keys`, hosted.rs `front_face` (one helper; `on_front` and `shut_menus_off_front` use it too), keys.rs `toggle_start`.

### Red at 35fa4b8d
- rcf-9: `a_turn_that_shuts_a_menu_leaves_the_keys_of_another_client ... FAILED: xp takes the keys of another client: [Update { id: "xp-menu", ... visible: Some(false) ...}, ... FocusNode { node: Some("xp-icons") } ...]`
- rcf-7: `the_start_key_opens_the_menu_on_the_front_face ... FAILED: Ctrl+Escape: [...] left: Some("V-1") right: Some("V-3")`
- `cargo test -p chora-xp hosted`: 11 passed, 2 failed.

### Fix (3a625261)
- rcf-9: `desktop_keys` sends `FocusNode { node: None }` while `node_away()` (a node of another client holds the keys), as it already did for a window of another manager (`focus_away`). The core's none releases only a node of xp (`focus_node_command`, chora-scene scene.rs:1074), so the other client keeps the keys; `close_menu` and `close_context` in `shut_menus_off_front` still shut the menus. The core tells `focus.node` on a click, a lock and a client that went (crates/chora state.rs `tell_key_node`), so `key_node` goes None when the user clicks xp's desktop and xp takes the keys again as before.
- rcf-7: `toggle_start` opens on `front_face()` (the host's face at `front`) and falls back to `bars.first()` without a host. Through `Wm::focus` / kit::wm::focus where keys move (D116); no sleep, no known row, no raised bar. Outside the four named files: press.rs only, where `desktop_keys` lives (the brief names it).
- `cargo fmt`, `cargo clippy -p chora-xp --all-targets`: clean.

### Green and counts
- `cargo test -p chora-xp`: 345 passed (hosted 13/13). `cargo test -p chora-construct`: 268 ok.
- New tests 10/10.
- gate-construct-focus.sh 10/10 green (`ok: a turn takes the keys from the window of a face that does not show`, `ok: the keys stay with the front face after its focused window closed`).
- gate-construct-i3.sh 10/10 green.

### Note (not fixed, for triage)
- The exported `StartMenu { open: true }` call (dispatch.rs:120) still opens on `bars.first()` on a hosted cube; same shape as rcf-7, but not a key path. One line with `front_face()` if wanted.

### Gate
`tests/merge-gate.sh --jobs 2` (full, detached) on 3a625261 (claude/co-focus with fresh 230ea0e5): **62 PASS, EXIT 0**, "all steps passed". `pgrep -fa 'chora --headless'` empty after.
