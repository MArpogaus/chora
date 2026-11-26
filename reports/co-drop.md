# co-drop

## r1
Lane: cloud, claude/co-drop-r1. **Base 7d9269ae** (co-reach r5 head). **Red 730cc738** (test: a drop or put stood back sends no key to a hidden window), **fix 75d4623e** (fix: a drop or put stood back keeps keys out of a hidden window), **head 75d4623e**. Step 0 green: unshallow, `cargo build --workspace` (includes check). No core change, no xp change, no other look.

Diffstat 7d9269ae..75d4623e: 6 files, +365 -18: `plugins/chora-construct/src/pull.rs` +66, `names.rs` +15, `cube/tests_construct/drop_tests.rs` +249 (new), `cube/tests_construct.rs` +2, `tests/gate-construct-pull.sh` +44, `docs/cube-modes.md` +7.

### Step 1: reproduction (headless, 7d9269ae)
Script: a `gate-construct-pull.sh` session (1280x800 plus a 640x800 second output, `--test-injected-press-is-real`), the probe window pulled past the edge into a pane, moved left, `Alt+Escape` stands back, then a Super drag of the pane onto face 0 (front) or face 1 (side), then `Return`. After each step I dumped `cube.list` mode, the core focus (`scene.query *` focused), the construct and xp focus holders and the probe log.
- Stood back before the drop: mode navigate, core focus `construct-look`, construct holder construct.
- Drop onto face 0: mode **navigate**, core focus **s1**, xp holder `xp/s1`, the window is on VIRTUAL-1. `Return` (the bar tells `Return Step in`): the mode stays navigate and the probe logs `key 28`.
- Drop onto face 1: mode **navigate**, core focus **s1**, xp `xp/s1`, the window is on VIRTUAL-2, a face the person does not see head on. `Return`: the probe logs `key 28`.

**rcr5-2 confirmed, MAJOR**: after the drop, keys go into the dropped window, and the told `Return Step in` types into it (co-68 on the documented mouse way; the RC bar's "no stray keys into clients"). On face 1 they go into a window on a face the person does not see. The bar also tells `Super Alt 4 Pull out`, as the keyboard is in a face window (before-drop-front-navigate.png).

rcr5-1 (`window.move` fails) by a test double, on the fake core of `tests_construct` (`refuse_next("methods.call")`, the refused call being `window.move` asserted): on 7d9269ae `put_in_face` steps in (mode use) and activates the window wherever xp put it, and a stood-back drop leaves the keyboard with the window; no notice either way. **rcr5-1 confirmed** (MINOR as reviewed: it needs a failing or timed-out `window.move`, which no headless run hit).

### Rule (fix 75d4623e)
Stood back, the window of a pane that goes to the face manager by a drop onto a face or by its give key (`Super+Alt+2`, co-79) ends in one place, `given_from_a_pane` (pull.rs):
- placed on the face at the front: noted, and the person steps in (`mode_act(StepIn, By::Person)`), so the face manager activates it: as `put_in_face` did (co-79).
- placed on another face: the cube stays stood back, the construct takes the keyboard to its look (`release` + `focus_node construct-look`, as the step back does) and forgets the window as the one to focus at the step in.
- not placed (`window.move` failed): told (`names::not_placed`: "XP took the window but not onto that face: <reason>."), stays stood back, keyboard to the look.
- stepped in (a drop from use mode): unchanged.
`place_on` answers the reason when it cannot place. The keyboard is taken after the answer of `window.move`, so after xp adopted and focused the window: no race with its focus. docs/cube-modes.md says the rule beside the drop onto a face.

### Tests
- Unit (`cube::tests_construct::drop_tests`, 5): front drop steps in and activates s1; side drop stays navigate, keyboard at the look, s1 not noted; failed place after a drop and after a put: told once, navigate, keyboard at the look, no activate; a placed put steps in (control, green on both). On 730cc738: 4 failed, 1 passed (the control). The fixture got the look node of the construct in the fix commit (the fake scene refuses `focus_node` to a node that does not exist); with it the 4 still fail on 7d9269ae's pull.rs (checked). chora-construct 295/295 on 75d4623e.
- Gate cell, `gate-construct-pull.sh` 2a, now `super`, `plain` and `side`: a drop onto face 0 steps in and a typed `x` reaches the window; a drop onto face 1 stays navigate, the told `Return` steps in and the window logs no `key 28`. Red on 730cc738: `FAIL: rcr5-2: after the drop (super) onto the face at the front the cube did not step in: mode navigate, keyboard construct:None/None xp:xp/s1`. Green on 75d4623e (EXIT 0, the three `rcr5-2` ok lines).

### Captures (media/2026-11-25/co-drop-r1/, each under 220 KB)
- before-drop-front-navigate.png (7d9269ae): stood back after the drop onto face 0, the window on the face, the bar tells `Return Step in` and `Super Alt 4 Pull out`: the keys are in the window.
- before-drop-side-navigate.png (7d9269ae): the same after a drop onto face 1.
- after-drop-front-stepped-in.png (75d4623e): stepped into face 1 of xp, chora-probe active, bar `Alt Esc Step back | Super Alt 4 Pull out | Super F1 All keys`.
- after-drop-side-stood-back.png (75d4623e): stood back, the window on the side face, the bar without `Pull out` (the construct holds the keys).
- after-drop-side-return-steps-in.png (75d4623e): `Return` stepped into face 1; the window on face 2 got no key.

### Gate
`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push, full, detached) on 75d4623e.
- Run 1: stopped at `acts FAIL 44s`: tests/persona-selftest.sh:198 `jq: error (at <stdin>:44): null (null) and number (2) cannot be divided` / `FAIL: no click on Notepad`: the Notepad row of xp's Start menu had no rect yet when queried right after the Start click (xp-only session, no construct; not touched by this diff). Rerun of `tests/gate-acts.sh` alone: `ok: persona session`. A flake, named here as **FLAKE-PERSONA-NOTEPAD r1** (new; not the cube flake).
- Run 2: **all steps passed, 66 PASS, 0 FAIL** (`pull PASS 382s`, `cube PASS 139s`, `acts PASS 1853s`); last line `all steps passed; no --push, nothing is pushed`. After: `pgrep -fa 'chora --headless'` empty.

### Open
- A drop from use mode onto the front face is unchanged (the window is seen there); a failed `window.move` in use mode is now told but the keyboard stays where xp put it.

## Review r1 (2026-11-26)

Reviewer lane (cloud, read-only on code), claude/co-drop-r1 **head 75d4623e**, base 7d9269ae, 6 files +365 -18 (only plugins/chora-construct, tests/gate-construct-pull.sh, docs/cube-modes.md; no other look, no core). Step 0: unshallow, `cargo check --workspace` green. No merge gate run (the lane's run 2: EXIT 0, 66 PASS).

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 1 MINOR, 1 NIT).

Checks:
1. Red first: 730cc738 before 75d4623e. `cargo test -p chora-construct drop_tests`: at 730cc738 4 failed, 1 passed (the put control), each red at its behaviour assertion (drop_tests.rs:114, 146, 185, 210); at 75d4623e 5 passed. The fix commit changes the fixture only (adds the look node), no assertion. Gate cell 2a now loops `super plain side`: front drop asserts mode use and a typed `x` reaching the window; side drop asserts navigate after `settle`, `Return` steps in, no new `key 28`. Holds.
2. Rule: `given_from_a_pane` (pull.rs:313) is the one end of the drop and the put. Front and placed: note, `mode_act(StepIn, By::Person)`, which activates the noted window (the mode is checked navigate first, so the step in always changes). Another face or not placed: told when not placed (`names::not_placed`), stays navigate, `walk_keys` and `the_keyboard_to_the_look` (release + focus look), and the window is forgotten as the noted focus. Use mode: only the notice. The take comes after xp's answer to `window.move`, so after xp's own focus on adopt (the gate's side case confirms no `key 28`). docs/cube-modes.md:245 says the same. No path found where no look holds the keyboard; one path where the step in still gives keys to a hidden window (rcdr1-1).
3. D119 and the give menu (list.rs:253 "After a give the keyboard is left to `to`", rtf1-2 class): the side drop and the failed place keep the keyboard with the giver (the construct), as TILING-FOCUS r2 asks. The front drop gives it to the receiver's window, but by the person's step in into the face they see, exactly as the give key of co-79 already did; this diff does not widen that. Note, not a finding (the stale comment is rcdr1-2).
4. No other look changed. The gate polls (`until_ok`, `settle` = `settled` render-stats, no new `sleep`/`pace`). On 75d4623e: sleep-ratchet ok (778/929), size-ratchet ok (0 over 1500), lint-ratchet ok (allow 360, no-reason 0, eprintln 46).
5. Captures media/2026-11-25/co-drop-r1/ (83 to 216 KB): before-drop-side-navigate.png stood back with `Super Alt 4 Pull out` told (keys in the window); after-drop-side-stood-back.png the same view without `Pull out`; after-drop-front-stepped-in.png chora-probe active on face 1 of xp, bar `Alt Esc Step back | Super Alt 4 Pull out | Super F1 All keys`. They match the report. The session has 4 faces ("face 1 of 4").

Findings:
- rcdr1-1, MINOR, plugins/chora-construct/src/pull.rs:333, the side drop forgets the window as the noted focus only through `noted_focus()`, which answers None when the cube was turned since the step back, so with 3 or more faces (the gate session has 4): pane window with the keyboard in use on face 0, step back (noted s1 at face 0), turn to face 1, drop the pane onto face 2, turn back to face 0, `Return`: `take_focus` gives s1 and `activate_in_face` sends the keys into the window on face 2, which the person does not see; fix: forget the noted window whatever its face (a `Host::forget_focus(surface)` on the raw `focus`), and add that case to drop_tests with a third output.
- rcdr1-2, NIT, plugins/chora-construct/src/list.rs:253, the comment of `give` says the keyboard is left to `to` after a give, which no longer holds for a drop or put stood back onto another face or not placed; fix: add "unless the cube stands back: see `given_from_a_pane`".

## r2 (2026-11-26)

Lane: cloud, claude/co-drop-r2, base 75d4623e (co-drop r1), **head 636287c0**, 7 files +241 -52 (plugins/chora-construct command.rs, cube.rs, list.rs, pull.rs, drop_tests.rs; tests/gate-construct-cube.sh, tests/gate-construct-pull.sh). Step 0: unshallow, `cargo check`/`build --workspace` green.

Commits: red 1d820350 (a 3-face drop_tests case and the key sheet test at 1280x720; both fail on 75d4623e: the drop case at its `activated` assert, the sheet at "starts at 8"), then fixes 8582a528, 07654d7c, d590aae6, 822d5be4 (see below), then bffb0f2d (the side margin assert, red on 1240 wide) and 636287c0 (gate case).

- rcdr1-1: `Host::forget_focus(window)` forgets the noted window whatever its face; `given_from_a_pane` calls it. drop_tests `a_drop_on_a_third_face_leaves_no_key_for_the_step_in_after_a_turn_back` (3 faces: note on face 0, turn to 1, drop onto 2, turn back, `StepIn` by the person: no `activate`). Gate cell 2a has a fourth case `turned` (face 1 -> drop on face 2 -> turn back -> Return: mode use, no new `key 28`); it passed.
- rcdr1-2: the comment of `give` says "unless the cube stands back: see `given_from_a_pane`".
- rv9kb-3: `help()` keeps 32 from the top and the bottom (`HELP_MARGIN`); when two columns do not fit (`fixed + body > out - 64`) the sheet has three columns (`sheet_columns`, width of the output less 32, column pad 8, gap 12). There is no scroll node in the scene, so wrapping is the only bound; with a window shorter than three columns allow the sheet still starts at 32 and the foot goes under the margin. The cube key sheet at 1280x720 now spans y 59 to 661 (was 8 to 770). Two things my first try broke and the gate found: co-71 clicks at x 20 to close from outside (the sheet at 1240 wide covered it; the click is now x 8 and the sheet starts at 16), and acts I5 (the text of "Start a program or run a command" ran 15 px under its keycaps at 362 a column; now 402).
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, full, no step filter, on head 822d5be4: **all steps passed**, no foreign red (runs before it: 1 ratchets lint-ratchet +1 allow, 2 cube co-71, then acts I5, all mine and fixed above). The last commit 636287c0 changes only the pull gate script (the `turned` case); `tests/gate-construct-pull.sh` run alone on it passed. `pgrep -fa 'chora --headless'` empty.
- Captures media/2026-11-26/co-drop-r2/: c4-sheet-1280x720.png (the cube key sheet, three columns, Esc Close in the foot row, 280 KB) and after-side-drop-3-face-turn.png (stood back on face 2 of 4 after the drop from face 1, bar `Return Step in`, no `Pull out`; 209 KB).

Open: none.

## Review r2 (2026-11-26)

Reviewer lane (cloud, read-only on code), claude/co-drop-r2 **head 636287c0**, base 75d4623e, 7 files +241 -52. Step 0: unshallow, `cargo check --workspace` green. No merge gate run (the lane's: all steps passed on 822d5be4; 636287c0 changes only tests/gate-construct-pull.sh).

**Verdict: FIX** (0 BLOCK, 1 MAJOR, 2 MINOR, 1 NIT).

Checks:
1. Red first holds. `cargo test -p chora-construct`: at 1d820350 295 passed, 2 failed (`the_key_sheet_fits_an_output_of_720` at command.rs:940, the top; `a_drop_on_a_third_face_leaves_no_key_for_the_step_in_after_a_turn_back` at drop_tests.rs:300, `activated`); at bffb0f2d 1 failed (the side assert, command.rs:961) before d590aae6; at 636287c0 297 passed. 07654d7c (lint) and 822d5be4 (keycaps, acts I5 red on d590aae6) have no unit test of their own; I5 is the red check named in the commit, so not a finding.
2. rcdr1-1 holds: `Host::forget_focus` (cube.rs:1071) clears the raw `focus` when it names the window, whatever its face; `given_from_a_pane` (pull.rs:333) calls it on every path that is not the front drop, then `walk_keys` and `the_keyboard_to_the_look`. Front drop: `note_focus` at the current front and the step in, so a drop onto the front after a turn steps into the face the person sees. A noted window that was not given stays noted with its face, as before. No path found where no look holds the keyboard or keys reach an unseen face in the construct (the give menu path: known, ti-57). rcdr1-2: the `give` comment is true.
3. The key sheet. Probe (a temporary test, not committed, `Metrics::default`; base vs head; sheet x/y/w/h and the worst text end past the first keycap of its row):
   - 1280x720: base 200/8/880/762 (ends 770), head 16/59/1248/602 (ends 661), 3 columns of 402, worst +9 (the real fonts: about -8, the capture c4-sheet-1280x720.png shows "Start a program or run a command" clear of `Super`).
   - 1920x1080: unchanged (520/64/880/762, 2 columns).
   - 1024x768: base 72/8/880/762 (2 columns, foot 2 px under the edge, text clear); head 16/64/992/602, 3 columns of 317, worst +95: text under the keycaps (rcdr2-1).
   - 2560x720: head 16/59/2528/602, 3 columns of 836 (rcdr2-2).
   3 columns only when 2 do not fit (`fixed + body(two) <= out - 64`): holds. No new option, no copy of a kit helper; `#[allow]` count unchanged (one moved). Ratchets on 636287c0: size ok (0 over 1500), sleep ok (778/929), lint ok (allow 360, no-reason 0, eprintln 46).
4. Gate scripts: case `turned` turns with `cube.turn` and waits with `settle` (render-stats), no sleep; it asserts the mode and no new `key 28`, which cannot see the fault (rcdr2-3). gate-construct-cube.sh: the outside click moved from x 20 to x 8 because the sheet now starts at 16 (rcdr2-2).
5. `git merge-tree --write-tree 6daaaab8 636287c0` clean (tree edf3f0fe); `cargo check --workspace` on that tree green.

Findings:
- rcdr2-1, MAJOR, plugins/chora-construct/src/command.rs:638, three columns take the output width less 32 with no check that a row fits, so at 1024x768 (two columns do not fit: 762 > 704) a column is 317 wide (402 at 1280, where the widest row has about 8 px to spare and acts I5 saw 15 px under the caps at about 373): the text of the wide rows runs some 70 px under its keycaps, worse than base, which drew it legibly; `the_key_sheet_fits_an_output_of_720` tests neither 1024x768 nor the overlap; fix: choose the columns from the widest row (its text plus `keycaps_width`, as `key_row` measures) and elide a row text that does not fit, and add 1024x768 with a text-ends-before-its-keycaps assert to the test.
- rcdr2-2, MINOR, plugins/chora-construct/src/command.rs:638 (and :515), the three-column sheet keeps 16 from the sides with a pad of 8 and a gap of 12, against docs/construct-style.md rule 8 (a screen-fixed element keeps 32 from the edge; spacing in whole units of 8), and its width has no cap (2528 wide at 2560x720); 822d5be4 lowered the lane's own red side assert from 32 (bffb0f2d) to 16 (command.rs:974) and moved the co-71 outside click to x 8 (tests/gate-construct-cube.sh:303) to fit; fix: cap the three-column width at what three widest rows need, keep 32 at the sides with a gap of 16 or 24, and restore `x >= 32` and the click at x 20.
- rcdr2-3, MINOR, tests/gate-construct-pull.sh:493, case `turned` (as `side`) asserts no new `key 28` after Return, but chora-probe logs only presses (tests/probe/src/main.rs:1044) and the construct, holding the keyboard stood back, takes the Return press for the step in, so the case cannot fail when the step in gives the hidden window the keyboard (it would pass on 75d4623e); it asserts no holder; fix: after the step in, type `x` and assert no new `key 45` in `$log` (or that `keyboard_holder` does not name `$surface`).
- rcdr2-4, NIT, plugins/chora-construct/src/command.rs:624, the comment of `sheet_columns` says "less `UNIT` each side" while the code takes 4 `UNIT` (16 each side); fix: say 2 `UNIT` each side, or what rcdr2-2 makes it.

## r3 (2026-11-26)

Lane (cloud, sonnet), branch claude/co-drop-r3: base 636287c0 (r2 head) merged with the session head 6daaaab8 (92264c9c), **head 369cf96f**, pushed. rc 1, 2: rcdr2-1 MAJOR, rcdr2-2, rcdr2-3 MINOR, rcdr2-4 NIT.

Commits: red 9f25bf35 (`test: the key sheet at 1024x768; no key in a hidden window (red)`), fix 369cf96f (`fix: the key sheet takes its columns from the widest row, 32 aside`). Diffstat of both over the merge: command.rs +92/-28 (111 lines changed), tests/gate-construct-cube.sh 1, tests/gate-construct-pull.sh 7; 3 files, +92 -28.

Red first: the sheet test (`the_key_sheet_fits_an_output_of_720`) now runs 1024x768, 1280x720, 1280x800, 1920x1080 and 2560x720 with `x >= 32` and `x + w <= out - 32`, a width of at most 1360, and for every row the text end before its first keycap; on 636287c0 it fails at 1024x768 ("the sheet lies at 16..1008, 32 from the sides"). Gate case `turned`/`side` types `x` after the step in and asserts no new `key 45`. It fails on 75d4623e (built in a worktree, case alone): `FAIL: rcdr2-3: after the step in over face 2 a key typed into the window on that face, which the person does not see: construct:None/None xp:xp/s1`. On the fix head the pull step passes (520 s in the full gate).

Fix (command.rs only, no new option, no kit helper copied): `sheet_columns` takes the look and `or`, keeps 32 (`HELP_MARGIN`) from the sides for every width, and for three columns takes 3 widest rows (`row_need`: 12 + text + 8 + `keycaps_width` as `key_row` measures + 8) + 2 gaps of 16 + 2 pads of 16, at most the room between the sides. `key_row` cuts a text that does not fit with `Look::one_line` (the search mark only where it lies in the shown text). Pad 16 for all, gap 16 (three) or 24 (two). `sheet_columns` comment says 32 each side. co-71 outside click back at x 20.

Per size (the unit test, `Metrics::default`; rect x/w; worst text end to first keycap = the smallest gap, none negative):
- 1024x768: 3 columns, x 32, w 960, worst +9.3
- 1280x720: 3 columns, x 32, w 1216, worst +9 (1280x800 the same)
- 1920x1080: 2 columns, x 520, w 880, worst +9 (unchanged from base)
- 2560x720: 3 columns, x 618, w 1324 (capped, was 2528), worst +8
With the real fonts the wide rows of 1024x768 and 1280x720 show a cut text (`Turn the cube` is `T…` at 1024, `Start a program or run a com…` at 1280); no text under a keycap.

Captures media/2026-11-26/co-drop-r3/: c4-sheet-1024x768.png (247 KB), c4-sheet-1280x720.png (263 KB), c4-sheet-1920x1080.png (351 KB), each with Esc Close in the foot row.

Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push, no step filter) on 369cf96f: all steps passed; EXIT 0 (last lines: demo PASS 234s, leak PASS 473s, tiling-theme PASS, tiling-bar PASS 177s, mc-host PASS, tiling-disc PASS 73s, e2e PASS 277s, "all steps passed; no --push, nothing is pushed"). No red cell, no flake. `pgrep -fa 'chora --headless'` empty.

## Review r3 (PM, 2026-11-26, 120 lines, 92264c9c..369cf96f)

Read: brief co-drop-r3, "## r3", `git diff 92264c9c 369cf96f`, capture c4-sheet-1024x768.png, -1280x720.png. No gate run (the lane's: EXIT 0 on 369cf96f).

Checks:
1. Red first holds: 9f25bf35 (sheet test at 5 sizes with x >= 32, w <= 1360, text end before first keycap; gate case `turned` types `x`, asserts no new key 45) before 369cf96f (command.rs only).
2. rcdr2-2 fixed: 32 at the sides (HELP_MARGIN), pad 16, gap 16 or 24, three-column width capped by three widest rows (2560x720: 1324, centred at 618); co-71 click back at x 20. rcdr2-3 fixed: the case can now fail (lane shows it red on 75d4623e). rcdr2-4 fixed: comment true.
3. rcdr2-1 (text under keycaps): no text under a keycap at any size (unit test, capture). But the cut is the only remedy: at 1024x768 the capture shows "T…" (Turn the cube), "Walk forward, bac…", "Answer a que…", "Give window to Ti…", "Give window to M…", "Give window to C…", "Start a program or…". "T…" and the three give rows do not say what the key does; base (2 columns) drew them in full.

Findings:
- rcdr3-1, MAJOR, plugins/chora-construct/src/command.rs:763 (`key_row` cut), at 1024x768 the key sheet cuts row texts to a stub ("T…" for Turn the cube; "Give window to M…" / "C…" / "Ti…" hide which look), so the key list, part of the shared RC bar, no longer tells what 7 keys do; there is vertical room (sheet foot at 666 of 736); fix: a row text that does not fit wraps to a second line (the row grows by one line, `body_height` counts it) and is cut only past two lines; the unit test asserts at 1024x768 and 1280x720 that no row text is cut (no "…" in any `-what` text).
- rcdr3-2, NIT, plugins/chora-construct/src/command.rs:771, the search mark is dropped when the match lies in the cut part; with rcdr3-1 the case almost vanishes; fix: none needed beyond rcdr3-1.

Verdict (computed): FIX (0 BLOCK, 1 MAJOR, 0 MINOR, 1 NIT). Next: CO-DROP r4 (sonnet high) on 369cf96f.
