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
