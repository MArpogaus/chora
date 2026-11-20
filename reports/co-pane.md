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
