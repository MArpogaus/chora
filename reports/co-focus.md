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
