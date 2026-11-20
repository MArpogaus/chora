# co-pane (RC 2, construct round 3 fails)

## r1
Lane: cloud, branch claude/co-pane from bbaad12b (co-fix r2). Step 1 merge of origin/claude/autonomous-dev-session-3w-i33pg7: already up to date. Step 0: unshallow, `cargo check --workspace` green. Head of the branch: f10842d8. No attribution lines, identity Marcel Arpogaus on every commit.

| Item | Red | Fix | Test | Cell | Capture |
|---|---|---|---|---|---|
| 1 pane drag, I3 (MAJOR candidate) | a6e3a1b5 | 57e6d4c2 | `state::tests::a_look_away_from_the_focused_pane_takes_the_focus_off_it`, `a_small_look_keeps_the_focus_on_the_pane_in_sight` | live persona session (construct 1280x800, probe window, PERSONA_CHECK=1): drag, look, drag; scene read by `choractl state construct` | media/2026-11-20/co-pane-r1/item1-lookaway.png, item1-putback.png |
| 2 key sheet | 918d4a3c | b62c2a83, cac31e1b (1e92ccf5 reverted by f10842d8) | `cube::tests_construct::the_key_sheet_takes_the_keyboard_from_a_face_and_closes_on_escape`, `a_press_on_the_close_of_the_key_sheet_closes_it`, `the_key_sheet_does_not_focus_a_pane_that_is_out_of_sight_on_close` | live: cube, terminal given to xp, Super+F1, Escape (PARTIAL, see below) | item2-sheet-open.png, item2-after-escape.png, item2-sheet-before-close.png |
| 3 checker reads the xp child | a3327873 | 9c3434c1 | `tests/persona-selftest.sh` case "the checker counts the windows of the xp child of a cube as shown (no I1, no I3)" | persona-selftest, construct session with a terminal given to xp | - |

### Item 1: what the drag does
Scene read first: after the drag `choractl state construct` says `windows s1 shown:false` and `focus {holder: construct, surface: s1}`, so the breach is the product's, not the checker's. The drag is a plain **look** (no modifier: `press` starts `Drag`, `look` turns the camera; a drag down pitches the view down, so the pane rises out of the picture), not a pane move and not a pull. The pane never moved; it left the view and kept the keyboard focus, so a key went into a window no one saw. Fix: `unseen_focus_to_walk` (camera.rs) runs after a look and at the end of a glide: when the pane of the focused window is out of sight the keyboard goes back to the walk (release the surface, focus the look node), so the focus is never an unseen window. The sight test is one method `in_sight`, shared with `state.get`. After the fix, same cell: drag, look, drag: `focus {holder: construct}` without a surface, `windows s1 shown:false`, **0 breaches** in breaches.jsonl; a drag up brings s1 back (`shown:true`, 0 breaches).
Note for the PM (not changed): the look is FPS-style (drag down looks down, the scene moves up), so Max's three put-back drags, all downward (y 70 to 300, 104 to 300, 95 to 250), looked further down to the clamp at pitch -1.2. A drag up puts the pane back. Whether the look should follow the hand (grab the world) is a design question.
Also seen: in a fresh session the first drag after the start did not look (the press went to the window and focused it); the look came with the second drag, after a `look` action.

### Item 2: key sheet (PARTIAL)
- The sheet now takes the keyboard from the window that has it in a face of the cube (use mode) or in a pane, and gives it back when it closes (`help_takes_keyboard`, `help_gives_keyboard`, overlay.rs), only to a window that shows (cac31e1b, I3 on a hidden pane window seen in the first live cell).
- The Close of the footer is a button: a hit node `construct-help-body-foot-N-hit` (command.rs), a press closes the sheet (unit test). Live check not possible with act.py: the plugin ignores injected pointer events by design, so the persona's click on Close at (648,472) could never reach it (it also hit a key row, not the footer: Close is at about (1035,755) at 1280x800).
- Escape: with no program stealing the keyboard Escape closes the sheet (live, round 3 of the cell: sheet=1 focus=construct-look, Escape: sheet=0, keyboard back on s1; before the fix, with a program of the face focused, Escape went to it).
- **Residual, not fixed**: with xp holding the focus on a program (the demo case), Super+F1 itself opens the XP Start menu: the release of Super is a tap for xp, which opens its menu and takes the keyboard (`xp-menu`) after the sheet took it. Then Escape closes the menu first (focus xp-icons) and the sheet stays; Super+F1 closes the sheet. Same in 3 of 3 cells with an xp program focused (item2-sheet-open.png: sheet and Start menu together). That is the "Start menu unasked". A bare Escape grab was tried (1e92ccf5) and reverted (f10842d8): the core refuses the grab of a key without a modifier (code 4, missing capability, in chora.log). Swallowing the lone Super tap while the sheet shows over a cube was tried and did not change the injected chord (act.py injects the chord in one batch, before the plugin can regrab); not committed. Candidate for the PM: swallow `Super+Super_L` while a cube is in use mode (this is `swallow_for_window`, which skips a cube today), at the price of the XP Start key by a Super tap inside a face; or xp opens its Start menu on a tap only while it has the keyboard focus.

### Item 3: checker
Run.snapshot adds the state of xp, tiling or mc (`CHILDREN`) when it is connected and not a look of a construct session (`with_children`, check.py). Selftest case first (a3327873, red: "breach I1 in look: the toplevel s1 is owned by xp, which is no look", "breach I3 ... no look says it holds the focus"), then the fix (9c3434c1, green). C2 "windows lost ['s2']" of the demo watcher is not reproduced: C2 passes in the selftest case; the demo's red was the end state after the lock, not confirmed as a loss; left as is.

### Gate
`tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,pull,placed,construct-hud,construct-extra,walk,wm-pane` at f10842d8:
```
pre-commit PASS     5s
gate-lib   PASS    74s
ci-check   PASS    82s
build      PASS    20s
test       PASS   169s
clippy     PASS    12s
cube       PASS   111s
pull       PASS    76s
placed     PASS    14s
construct-hud PASS   104s
construct-extra PASS   155s
wm-pane    PASS    21s
walk       PASS     4s
```
`tests/persona-selftest.sh`: all `ok`, ending `ok: the checker counts the windows of the xp child of a cube as shown (no I1, no I3)` and `ok: persona session`.

### Needs the PM
- Item 2 residual (Start menu from the Super tap in a face): decide the swallow above or leave it to xp.
- Item 1 look direction (FPS-style) if a persona is to put a pane back by dragging "the picture".

## Review r1
Seeded review of claude/co-pane f10842d8 against bbaad12b (10 files, +288 -18), cloud lane, read-only on code. Step 0: `git fetch --unshallow`, `cargo check --workspace` green at f10842d8. `cargo test -p chora-construct`: 251 passed, 0 failed. `bash tests/persona-selftest.sh` (after `cargo build --workspace`): all `ok`, ending `ok: persona session`. No merge gate run (the lane ran it).

**Verdict: FIX** (one open MAJOR, rv-1). Counts: BLOCK 0, MAJOR 1, MINOR 6, NIT 1. If the PM accepts rv-1 as a known issue (docs/known-issues.md), the rest is MERGE-OK.

### Findings
- rv-1 MAJOR plugins/chora-construct/src/camera.rs:542: a construct chord with Super in use mode of a cube (Super+F1 and every other grabbed Super chord) leaves a bare Super press and release for the xp program that has the keyboard, which opens the XP Start menu unasked; `swallow_for_window` is skipped when `self.cube.is_some()`. Fix: in `regrab`, call `self.swallow_taps()` (already split out in f10842d8) while a cube is in use mode, at the price of the Start key by a Super tap inside a face.
- rv-2 MINOR plugins/chora-construct/src/overlay.rs:316: `help_takes_keyboard` takes the keyboard only `in_face || in_pane()`; in the walk with a pane window that was pressed (`keys_with_pane`, not flown) the sheet opens while the program keeps the keyboard, so Escape and the typed search go to the program (read from code, not run live). Fix: `if !(in_face || self.keys_with_pane())`.
- rv-3 MINOR plugins/chora-construct/src/overlay.rs:342: `help_gives_keyboard` activates the noted face window with `activate_in_face` and no check that its face is still at the front and the cube in use mode (`give_the_keyboard_back` goes through `Host::take_focus`, which checks); a turn or step back while the sheet shows gives the keyboard to a window no one sees (an I3 case). Fix: note the window with `host.note_focus` when the sheet opens and give it back through `take_focus` on close.
- rv-4 MINOR plugins/chora-construct/src/command.rs:224: `footer` is shared with the window list (command.rs:280), so its last item (`Esc Close`) also gets an `input: accept` hit node that nothing handles (input.rs:693 matches only `construct-help`). Fix: pass a flag so only the key sheet makes the hit, or handle the hit of the list.
- rv-5 MINOR plugins/chora-construct/src/camera.rs:428: `unseen_focus_to_walk` at the end of a glide runs from `animation_frame`, outside `Plugin::event`, so no `regrab` follows and the chords swallowed for the window (`keys_with_pane` is now false) stay until the next regrab; the look path (camera.rs:321) is covered by the wrapper; no test covers the glide path. Fix: `self.regrab()` after `keys_to_the_walk` in `unseen_focus_to_walk`, and a test of a step that takes the pane out of sight.
- rv-6 MINOR plugins/chora-construct/src/camera.rs:32: a drag down looks down (pitch minus dy) while the cube follows the hand (docs/cube-modes.md:131) and F5 (plans/construct-requirements.md:106) names no direction; Max's three downward put-back drags went further away, as designed but against the expectation of a drag-to-look viewer (grab the world), so MINOR POLS, not MAJOR (the focus is no breach any more and a drag up puts the pane back). Fix: decide the direction in F5 and docs, invert the sign of `dy` (and `dx`) in `turn` or add a setting. The report's "first drag after a fresh start does not look" is undiagnosed and has no test (a press on a window of a pane is taken for the window, input.rs:302); fix: reproduce it with a test or write it as by design.
- rv-7 MINOR tests/persona/check.py:89: `with_children` takes the state of xp, tiling or mc whenever it is connected in a construct session; it does not check that the child is inside a construct face, and a window the child calls shown counts as shown even when its face is not at the front (the I3 of rv-3 would pass), and the selftest has no negative case. Fix: take the child only when `cube.list` names it as the manager of a face, keep the front-face test for `shown`, and add a case where the child is not in a face and still breaches I1.
- rv-8 NIT plugins/chora-construct/src/camera.rs:557: after the revert f10842d8 leaves two things of 1e92ccf5 that nothing needs: `swallow_taps` has one caller (it is the helper of rv-1 if that fix is taken, else inline it again) and `self.regrab()` in `toggle_help` (overlay.rs:303) was added for the Escape grab; `Grab::CloseHelp`, `grab_help_escape` and the grab test are gone. Fix: take rv-1 or inline the loop; keep the regrab only with a reason.

### Checks
1) Look away: the keyboard goes to the walk after a look and after a glide that ends with the pane out of sight, never in a pane (`in_pane`), a cube or an overlay. No loop with the glide: it acts only when `core_focus` is an own window, and sets it to None. A click gives the keyboard back (input.rs:302); a look back does not, the keyboard stays with the walk until a click (the doc comment says so). `cargo test -p chora-construct`: 251 ok. rv-5.
2) Key sheet: takes it in a face (use mode) and in a flown pane, gives it back to a shown pane window; the face path has no front check (rv-3) and the walk with a pressed window is missed (rv-2). Close is a hit node `construct-help-body-foot-N-hit` at the drawn footer (rect from the same x and top as its item; the test presses it); it also lands on the window list (rv-4). The revert: rv-8.
3) Residual: MAJOR, rv-1. The key list is the headline key of the shared bar ("no stray keys into clients"), the default cube has xp faces, and the Super tap is by construction the same for every grabbed Super chord (the client never sees the chord key), so it is not only F1. Escape closes the menu first and the sheet stays: two presses. I did not reproduce it live (act.py injects the chord in one batch, as the report says). Smallest fix: construct swallows the lone Super tap in use mode (rv-1).
4) Look direction: MINOR, rv-6.
5) Checker: the selftest case is green and red-first; the counting is by connection, not by face; rv-7. No hidden I1/I3 masked in the tested case.
6) Red first, run by me: a6e3a1b5 (on bbaad12b) fails `a_look_away_from_the_focused_pane_takes_the_focus_off_it` at state.rs:246, left `{"holder":"construct","surface":"s1"}`, right `{"holder":"construct"}`: the named reason. 918d4a3c (on 57e6d4c2) fails `the_key_sheet_takes_the_keyboard_from_a_face_and_closes_on_escape` (no surface keeps the keyboard: left None, right Some(None), tests_construct.rs:827) and `a_press_on_the_close_of_the_key_sheet_closes_it` (tests_construct.rs:858: no hit node). a3327873: `bash tests/persona-selftest.sh` fails with `construct: the checker reads no state of the xp child: breach I1 in look: the toplevel s1 is owned by xp, which is no look` and `breach I3 ... no look says it holds the focus`; green at 9c3434c1 (head).

## PM note r1 (review)
Computed verdict FIX (rv-1 MAJOR open); not accepted as a known issue (shared bar, default cube has xp faces). co-pane r2 (briefs/co-pane-r2.md) fixes rv-1 first, then rv-2..rv-8. rv-6: PM decides "grab the world" (docs/cube-modes.md:131). Earliest gate that could have caught rv-1: none (act.py sends a chord in one batch): backlog harness item.

## r2
Lane: cloud, branch claude/co-pane from f10842d8, head 628bb930 (pushed). Step 0: unshallow, `cargo check --workspace` green. No attribution lines, identity Marcel Arpogaus on every commit. Each item: red test commit first, then the fix.

| Item | Red | Fix | Test | Cell |
|---|---|---|---|---|
| rv-1 MAJOR lone Super tap | fabe366e | e73e9982 | `cube::tests_moved::a_cube_in_use_mode_swallows_the_lone_super_tap` | unit (grabs after `regrab`: both taps held in use mode, none after step back, other chords stay with the face manager); doc: price in docs/cube-modes.md |
| rv-2 sheet in the walk with a pressed window | 984ba2fb | 13fd5ce6 | `the_key_sheet_takes_the_keyboard_from_a_pressed_pane_window` | unit |
| rv-3 give back through `take_focus` | 0207d17b | ea501689 | `the_key_sheet_gives_the_keyboard_back_only_to_the_front_face` | unit (still front: one activate; turned: none) |
| rv-5 regrab after a glide | 7ac19649 | a6b906b8 | `state::tests::a_step_that_takes_the_pane_out_of_sight_lets_go_of_the_chords` | unit |
| rv-7 checker reads a child only as face manager | ff4969f1 | 13552608 | `tests/test_acts.py` class `WithChildren` (3 cases, run from `tests/persona-selftest.sh`) | selftest: positive live case still green; negative cases: client not in `cube.list` stays no look (I1), face behind the front shows nothing (I3) |
| rv-4 Close hit only on the key sheet | 6e4bffed | f0ca2654 | `command::tests::only_the_key_sheet_makes_the_close_a_hit` | unit |
| rv-6 look follows the hand | b22397b9 | 4463a068 | `tests::a_drag_takes_the_scene_along_with_the_hand` (projects the pane: right drag moves it right, down drag down) | unit; F5 in plans/construct-requirements.md |
| rv-6 first drag | - | d762d10f (pin) | `a_press_on_a_pane_window_starts_no_look` | by design: a press on the window of a pane focuses the window and starts no look (input.rs:296), so a drag that begins on a shown window does not look |
| rv-8 NIT | - | - | `the_key_sheet_lets_go_of_the_chords_of_a_pane_window_and_takes_them_back` (628bb930) | the `toggle_help` regrab stays: without it the test is red (the sheet moves the keyboard between the pressed window and the walk, so the chords held for the window must go and come); `swallow_taps` now has two callers |

### Not done
- rv-1 gate step: tests/persona/act.py sends a chord as one `shortcut` batch (no separate press and release), so no persona step can show the lone tap; unit test only. Harness backlog item stays.
- Not run live: no persona drag of the new look direction; the gates with a drag (3d, k2a, mcpane, placed) do not read its direction and pass.

### Gate
`tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,pull,placed,construct-hud,construct-extra,wm-pane,pane-keys,walk,keys` at 628bb930:
```
pre-commit PASS     4s
gate-lib   PASS    73s
ci-check   PASS    72s
build      PASS    22s
test       PASS   175s
clippy     PASS    13s
cube       PASS   112s
pull       PASS    78s
placed     PASS    14s
construct-hud PASS   107s
construct-extra PASS   158s
wm-pane    PASS    20s
pane-keys  PASS     8s
walk       PASS     5s
keys       PASS    37s
```
`bash tests/persona-selftest.sh`: all `ok`, ending `ok: the checker reads a child only as a face manager, and shows only the front face (negative cases)` and `ok: persona session`. `cargo test -p chora-construct`: 259 passed; `python3 -I tests/test_acts.py`: 151 passed.

## Review r2
Seeded review of claude/co-pane 628bb930 against f10842d8 (13 files, +434 -38), cloud lane, read-only on code. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. `cargo test -p chora-construct`: 259 passed, 0 failed. `bash tests/persona-selftest.sh` (after `cargo build --workspace`): all `ok`, ending `ok: persona session`; `python3 -I tests/test_acts.py WithChildren`: 3 ok. No merge gate run, no live run (act.py sends a chord as one batch).

**Verdict: FIX** (one open MAJOR, rw-1). Counts: BLOCK 0, MAJOR 1, MINOR 1, NIT 1. rv-2, rv-3, rv-4, rv-5, rv-7, rv-8 are closed; rv-1 is closed for the lone tap after a chord but opens rw-1; rv-6 is closed in code, open in the docs (rw-2).

### Findings
- rw-1 MAJOR plugins/chora-construct/src/camera.rs:549: the swallowed `Super+Super_L` / `Super+Super_R` is matched on the press of Super itself (core, `crates/chora/src/input.rs:307` and the test at input.rs:1611-1618: the grab of the tap is `pressed` on the Super key down), and an intercepted key is not forwarded and its modifier change is not sent (smithay `input_from_source`: `mods_changed` goes only with `input_forward`), so in use mode the program of a face never sees Super go down: a Super chord of the program that construct does not grab (xp Win+E, Win+R, Win+D) arrives as the bare key and types `e`, `r`, `d` into the focused field, a stray key into a client (the headline of the shared bar); before r2 the program got Super and the chord. docs/cube-modes.md names only the lone tap as the price. Read from code, not run live. Fix: swallow the tap only for the tap that a grabbed chord leaves (grab it on the press of a construct chord and drop it at the release of Super), or take the grab out and accept the Start menu, and say in docs/cube-modes.md which Super keys of the program stay.
- rw-2 MINOR docs/cube-modes.md:47, docs/configuration.md:920: rv-6 asks that docs and F5 say the drag direction in the same words; only F5 (plans/construct-requirements.md:106) does, and it cites docs/cube-modes.md for it, where the walk-and-look drag (line 47) names no direction and the only "follows the hand" is the cube turn (line 139); configuration.md:920 says "A drag there looks around". Fix: add the F5 sentence (the scene follows the hand: right turns the view left, down tilts it up; a press on a pane window starts no look) at cube-modes.md:47 and configuration.md:920.
- rw-3 NIT plugins/chora-construct/src/overlay.rs:310: the doc comment line of `help_takes_keyboard` runs past the width of the others after the edit. Fix: rewrap it.

### Checks
1) rv-1: `regrab` swallows the lone taps only while the cube is in `Mode::Use`, and `mode_act`, `open_modes` and `close_modes` regrab on every mode change, so standing back drops them (the unit test shows it); only the two tap bindings are grabbed, typing and the other chords stay. The unit test `a_cube_in_use_mode_swallows_the_lone_super_tap` pins construct's grab map (`rt.action`), not the key stream of the client, and the lane says so; the stream is what rw-1 shows. No gate step possible (act.py).
2) rv-2, rv-3: the sheet takes the keyboard also for a pressed pane window (`keys_with_pane`), notes a face window with `note_focus` and gives it back through `take_focus` (front check); a pane window goes back only while flown or in sight. Both tests pass; nothing found.
3) rv-4: `footer` takes a `button` flag, only the key sheet makes the hit (test counts hits for sheet and list). rv-5: `unseen_focus_to_walk` regrabs, the state test steps a glide to the end and the chords are gone. Nothing found.
4) rv-6: `turn` inverts dx and dy, the projection test checks the four directions on the screen; the "first drag does not look" is pinned by design (`a_press_on_a_pane_window_starts_no_look`, d762d10f) and written in F5. The persona and gates read no drag direction (grep). Docs: rw-2. POLS against the look of the cube: the same hand-follows direction, consistent.
5) rv-7: `with_children` takes a child only when `cube.list` names it as a face manager and counts `shown` only for the output of the front face; the three `WithChildren` cases are the negative cases (no manager: I1; face behind the front: I3), and the live positive case of the selftest is still green.
6) Red first, run by me on the red commit: fabe366e (rv-1) fails `a_cube_in_use_mode_swallows_the_lone_super_tap` at tests_moved.rs:163 (no tap grabbed); 0207d17b (rv-3) fails `the_key_sheet_gives_the_keyboard_back_only_to_the_front_face` at tests_construct.rs:910 (left 1, right 0: a window of a face that is not at the front got the keyboard); b22397b9 (rv-6) fails `a_drag_takes_the_scene_along_with_the_hand` at tests.rs:203 ("right with the hand"). All three are the named reasons; each fix commit follows its red commit. rv-8: the regrab stays with a test that is red without it (reading of the test; not run without the line).
