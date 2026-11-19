# Moved tests

## xp r2

Lane: moved-tests-xp (F5, rank 5 of reports/audit-f.md), branch claude/moved-tests-xp,
commit bf0b66eb on integ 0113008b. Tests: plugins/chora-xp/src/wm/tests_moved.rs (new,
on the fake core of wm.rs, 49 tests); production code unchanged; one `#[cfg(test)] mod
tests_moved;` line in wm.rs.

Line coverage of the xp modules added since fe8ba6d0 (`env -u XDG_RUNTIME_DIR cargo llvm-cov
-p chora-xp --features chora-kit/testing`; before = integ 3619ccb3, after = bf0b66eb's base
before the merge of integ 0113008b):

| module | before | after | at 70 %? |
|---|---|---|---|
| choices | 42.1 | 91.4 | yes |
| dispatch | 37.6 | 71.9 | yes |
| hosted | 70.9 | 73.3 | yes |
| keys | 24.9 | 79.8 | yes |
| lifecycle | 95.9 | 95.9 | yes |
| output | 69.2 | 99.1 | yes |
| panels | 10.3 | 40.7 | **no** |
| pointer | 23.5 | 60.4 | **no** |
| press | 17.0 | 71.9 | yes |
| startup | 89.7 | 89.7 | yes |
| state | 90.2 | 90.2 | yes |
| surfaces | 68.1 | 89.4 | yes |
| windows | 63.1 | 74.6 | yes |

Not at 70 %, and why this round stopped there:
- panels.rs: the dialog clicks (Display Properties list and apply, Effects, Date, Help topics,
  release_panel, tray and icon paths) need the node geometry of each dialog page; not reached
  in this round. Not gate-only.
- pointer.rs: pull_out (cube faces, virtual outputs with an owner), press_tray, press_icon,
  the drop on a far output and the launch paths (start_program spawns processes, left out).

Tests: `cargo test -p chora-xp` 247 passed on bf0b66eb with the merged integ; the two known
tests (wm::state a_give_shuts_the_start_menu, a_give_with_the_drag_shuts_the_start_menu)
pass there too. `cargo clippy -p chora-xp --all-targets`: 0 warnings. `cargo fmt --check`: clean.
merge-gate: see the line below.

merge-gate `tests/merge-gate.sh --jobs 3` on bf0b66eb: GATE-EXIT=0, 40 of 40 steps PASS (log: /tmp/merge-gate.mmWJGp/).

Not pushed with the author the hook asked for: bf0b66e is authored by Marcel Arpogaus as the brief requires; GitHub shows it Unverified (no signature, not a noreply@anthropic.com committer).

## Review xp r2 (PM, 15:20 UTC; tests only, PM not the builder)
Diff 0113008b..bf0b66eb: wm.rs +3 (cfg(test) mod), wm/tests_moved.rs +1032; production code unchanged. 49 tests.
- mt-1 MINOR: `a_node_asks_xp_for_an_update` (tests_moved.rs:211) asserts nothing; it only checks no error. Assert the reply or the recorded methods.
- mt-2 MINOR: `a_missing_shared_script_is_told_in_a_balloon` (:761) asserts no balloon; the name claims one.
- mt-3 MINOR: `the_desktop_fades_while_a_dialog_is_open` (:773) asserts only under `if wm.fade.is_some()`; vacuous when no fade starts. Assert the fade first.
Verdict (computed): BLOCK 0, MAJOR 0, MINOR 3: MERGE-OK; mt-1..3 go to the moved-tests rest lane. Merges after integ (based on integ 0113008b).

## moved-tests-ct r1

Lane: moved-tests-ct (F5, rank 5 of reports/audit-f.md), branch claude/moved-tests-ct from af5547c7
(integ head), commits b25a52f7 (tiling) and c4c4f5d5 (construct). Tests in new files only: tiling
src/methods/tests_moved.rs (7) and src/overlay/tests_moved.rs (10), construct src/cube/tests_moved.rs
(9); one `#[cfg(test)] mod tests_moved;` line in each module file. Production code unchanged.

Line coverage of the modules added since fe8ba6d0 (`env -u XDG_RUNTIME_DIR cargo llvm-cov -p
chora-construct -p chora-tiling --features chora-kit/testing --summary-only`; before = af5547c7,
after = c4c4f5d5):

| module | before | after | at 70 %? |
|---|---|---|---|
| construct cube/construct.rs | - | - | not at HEAD: no such file (cube.rs is modified since fe8ba6d0, not added); re-measure after the cube split |
| construct cube/hand.rs | 33.3 | 96.7 | yes |
| construct cube/role.rs | 71.8 | 74.4 | yes |
| construct cube/start.rs | 46.6 | 92.2 | yes |
| construct lifecycle.rs | 95.5 | 95.5 | yes |
| construct notice.rs | 100.0 | 100.0 | yes |
| construct state.rs | 97.8 | 97.8 | yes |
| tiling commands.rs | 87.6 | 87.6 | yes |
| tiling drag.rs | 84.5 | 84.5 | yes |
| tiling lifecycle.rs | 95.9 | 95.9 | yes |
| tiling methods.rs | 0.0 | 100.0 | yes |
| tiling overlay.rs | 30.9 | 74.7 | yes |
| tiling state.rs | 100.0 | 100.0 | yes |
| tiling windows.rs | 84.0 | 84.0 | yes |

Not reached in this round: overlay.rs apply_theme (writes the theme files of the user and signals
waybar or runs systemctl --user) and the answer branch of follow_system (it applies a theme), both
left out on purpose; the waybar branch of hide_bar_for_fullscreen needs a panel process.

Tests: `cargo test -p chora-tiling` 176 passed; `cargo test -p chora-construct` 202 passed.
`cargo clippy -p chora-tiling -p chora-construct --all-targets`: 0 warnings.
merge-gate `tests/merge-gate.sh --jobs 3` on c4c4f5d5: GATE-EXIT=0, 41 of 41 steps PASS (log in the
session scratchpad). The first run, on 75387408, failed in ci-check (gate-0.8-plugin-runtime): three
construct tests named the raw method `output.list` outside an item that allows
clippy::disallowed_methods. They now carry that allow, the commit was amended, the gate was green.
Commits carry the author Marcel Arpogaus and no attribution lines (the commit hook refuses them).

## Review ct r1 (PM, 16:06)
Base af5547c7..c4c4f5d5, 6 files, +789; production change only `#[cfg(test)] mod tests_moved;` in cube.rs, methods.rs, overlay.rs.
Checks: every `#[test]` fn has an assert (only helpers on_core, count, virtual_outputs, properties, required, calls, one_window, on_scene, drawn have none); no assert under an `if`; names match the asserts (spot check cube/tests_moved.rs: hand, release, manager_runs, face_manager, reopen); lane gate 41/41 on c4c4f5d5.
Note: cube.rs may conflict with cubesplit 10cf04cf in train 65; resolve by moving the mod line to the split module.
Counts: BLOCK 0, MAJOR 0, MINOR 0. Verdict: MERGE-OK.

## moved-tests-mk r1

Lane: moved-tests-mk (F5, rank 5 of reports/audit-f.md), branch claude/moved-tests-mk at
31df2bb9 (base bf0b66eb + origin/claude/integ af5547c7). Production code unchanged.
Tests: plugins/chora-mc/src/acts/tests_moved.rs (new, 16 tests, one `#[cfg(test)] mod
tests_moved;` line in acts.rs); crates/chora-kit/tests/moved_cells.rs (new, 6 cells);
plugins/chora-xp/src/wm/tests_moved.rs (mt-1..3 fixed).

Line coverage, `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-mc -p chora-kit -p chora-xp
--features chora-kit/testing --summary-only` (before = integ tree before this lane's tests,
after = 31df2bb9):

| module | before | after | at 70 %? |
|---|---|---|---|
| mc acts.rs | 15.6 | 77.1 | yes |
| mc lifecycle.rs | 60.5 | 60.5 | **no, not reached this round** |
| mc region/hits.rs | 60.9 | 60.9 | **no, not reached this round** |
| xp wm/panels.rs | 40.7 | 40.7 | **no, not reached this round** |
| xp wm/pointer.rs | 61.1 | 61.1 | **no, not reached this round** |
| kit cube.rs | 91.4 | 91.4 | yes (cell added, already covered) |
| kit wm/focus.rs | 96.1 | 96.1 | yes (wm_focus.rs already a cell) |
| kit wm/open.rs | 100.0 | 100.0 | yes (cell added) |
| kit wm/give.rs | 100.0 | 100.0 | yes (cell added) |
| kit wm/kept.rs | 97.3 | 97.3 | yes (cell added) |
| xp wm/tests_moved.rs | 99.3 | 99.4 | yes |

Total lines of the three crates: 88.95 %.

Not at 70 %, and why this round stopped there: panels.rs needs the node geometry of each dialog
page (Display Properties, Effects, Date, Help, tray and icon paths); pointer.rs needs pull_out
(cube faces, virtual outputs with an owner) and the launch paths; lifecycle.rs and hits.rs were
listed by the measure but their missing lines were not analysed. acts.rs leaves the claim
hand-over of drag (release on another window manager's claim, lines 33-43) unreached: a claim
node of another owner is not placeable through the fake core's public scene API.

Kit cells are plain calls of the public API (no FakeCore): cube quarters, Book::give with
refused and confirmed, pass_on, Open, kept::read. wm focus is not added: wm_focus.rs covers it.

Tests: `cargo test -p chora-mc -p chora-kit -p chora-xp --features chora-kit/testing` all green.
`cargo clippy -p chora-mc -p chora-kit -p chora-xp --all-targets --features chora-kit/testing`:
0 warnings. `cargo fmt --check` clean. Red-first does not apply (tests of existing behaviour).
merge-gate `tests/merge-gate.sh --jobs 3` on 31df2bb9: GATE-EXIT=0, 41 of 41 steps PASS.

mt-1..3: a_node_asks_xp_for_an_update asserts the empty answer of requested_look for a window xp
does not hold; a_missing_shared_script_is_told_in_a_balloon counts one methods.call (the notice);
the_desktop_fades_while_a_dialog_is_open asserts the fade starts before its checks (no `if`).

Lane notes: setup.sh was refused by the auto-mode classifier and was not run (all tools were
present at step 0). Author Marcel Arpogaus, no attribution lines, as the brief says.

## Review mk r1 (PM, 16:12)
Own commit 31df2bb9: 4 files, +402 -14, production change only the `mod tests_moved;` line in mc acts.rs. Every `#[test]` in acts/tests_moved.rs and kit tests/moved_cells.rs asserts; no assert under an `if`; mt-1..3 fixed as named. Lane gate 41/41 on 31df2bb9.
Done criterion short: lifecycle 60.5, hits 60.9, panels 40.7, pointer 61.1 below 70 %; lifecycle and hits "not analysed" is no reason. That is scope left, not a fault in the tests: r2 (haiku) takes the four.
Counts: BLOCK 0, MAJOR 0, MINOR 0. Verdict: MERGE-OK (tests); F5 module coverage open until r2.

## moved-tests-mk r2

Lane: moved-tests-mk r2 (F5, the four modules under 70 %), branch claude/moved-tests-mk2 at
4904ef79 (on 31df2bb9 = r1; the autonomous-dev branch was already contained). Production code
unchanged. step 0: all tools present, setup.sh not run; cargo check --workspace clean.

Line coverage, `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-mc -p chora-xp --features
chora-kit/testing --summary-only`; before = 31df2bb9, after = 4904ef79:

| module | before | after | at 70 %? |
|---|---|---|---|
| mc lifecycle.rs | 60.5 | 97.0 | yes |
| mc region/hits.rs | 60.9 | 96.3 | yes |
| mc acts.rs | 77.1 | 84.4 | yes |
| xp wm/pointer.rs | 61.1 | 63.3 | **no** |
| xp wm/panels.rs | 40.7 | 40.7 | **no, not reached this round** |

New tests (18): mc lifecycle/tests_moved.rs (8), mc region/hits/tests_moved.rs (8), mc
acts/tests_moved.rs (+1: a release on another manager's claim reserves the window for it,
reached with a second connection on the fake core), xp wm/pointer/tests_moved.rs (1: a release
on no claim keeps the window with xp).

Open, with the reasons:
- xp pointer.rs, 31 more lines needed: start_program, start_named and check_launches (484-529)
  start programs through the launcher, which spawns a process; not reached without one. The
  release and drag branches (237-291, 306-332, 349-369) need window geometry per case; not done.
- xp panels.rs, 155 more lines needed: the dialog pages (display, effects, date, help, taskbar,
  tray) and their open, click, release and redraw paths; the page geometry per dialog is what a
  test must set up. Not analysed line by line this round.

Tests: `cargo test -p chora-mc -p chora-xp --features chora-kit/testing` green (167 and 250).
`cargo clippy -p chora-mc -p chora-xp --all-targets --features chora-kit/testing`: 0 warnings.
`tests/merge-gate.sh --jobs 3` on 4904ef79: GATE-EXIT=0, all steps passed (a first run was cut
by the 590 s limit of the shell at 7 steps; the second run ran to the end).
Red-first does not apply (tests of existing behaviour).

## Review mk r2 (PM, pending)

## Review mk r2 (PM, 17:03)
Base 31df2bb9..4904ef79, one commit, 7 files, +436/-1: three `#[cfg(test)] mod tests_moved;` lines, the rest new test code; 53 asserts, none under an `if` that may skip it (read).
Checks: mc lifecycle 97.0, hits 96.3, acts 84.4 (drag hand-over 33-43 reached): met. Worker's gate GATE-EXIT=0, tests and clippy clean (worker's report; the train gate re-runs it).
Open: xp panels 40.7 (unchanged, "not analysed", which the brief excluded as a reason) and pointer 63.3: not met; r3 (haiku) owns it, with the line table first.
Verdict (computed): no open MAJOR in the diff: MERGE-OK for 4904ef79 (tests only). F5 module coverage stays open until r3.

## moved-tests-mk r3

Lane: moved-tests-mk r3 (F5, xp wm/panels.rs and wm/pointer.rs to 70 %), branch
claude/moved-tests-mk3 from 4904ef79 (r2), merged with origin/claude/autonomous-dev-session-3w-i33pg7
(85764974, it had moved). Step 0: cargo, sccache, cargo-llvm-cov, foot, python3 present; `cargo check --workspace` clean.

Step 1, uncovered line ranges before any test (`env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-xp
--features chora-kit/testing --show-missing-lines`, 253 tests):
- wm/panels.rs: 40.68 % lines (526 lines, 312 missed; 70 % needs 369 covered, 155 more)
- wm/pointer.rs: 63.30 % lines (455 lines, 167 missed; 70 % needs 319 covered, 31 more)

Ranges, by function (class: UT = unit test on the fake core; GATE = the path spawns a program or
a shared script, or needs the user's files):

wm/panels.rs
| lines | function | class | reason |
|---|---|---|---|
| 33-37 | dismiss_notice (Err arm) | UT | dismiss on a dropped connection is logged |
| 40-60 | tell, incl. 56-60 | UT | Change::{Starting,Failed,Fallback,Done} as the launcher sends them |
| 73-77 | run, Action::Run | GATE | start_program spawns a program |
| 79-82 | run, LogOff/TurnOff/Set/Help | UT | opens the dialog, sets, opens help |
| 84-85, 88-89 | run, Lock and Screenshot | GATE | run_shared spawns the shared script when it is installed |
| 92 | run, end of match | UT | follows the arms above |
| 126-132 | open_help, help open already | UT | raises the open help and shows its topic |
| 158 | open_date_dialog, raise | UT | second open raises the dialog |
| 172 | open_display_dialog, raise | UT | second open raises the dialog |
| 188-191 | open_display_dialog, output not listed | UT | bar output unknown to the core |
| 224 | with_display, no dialog | UT | nothing to change |
| 233-286 | click_display (Effects and Display arms, Apply, List, Effects) | UT | Hit::Control names of the dialogs |
| 289-307 | open_display_list | UT | list click of Display Properties |
| 310-321 | open_effects | UT | Effects click, and a second open raises |
| 329, 334-335 | close_panel, Display tab kept, focus of the topmost window | UT | |
| 346-347 | change, settings refused | UT | settings.set error (invalid value) |
| 351 | change, settings not saved | GATE | needs the settings file to be unwritable |
| 381-402 | press_tray, speaker and clock double click | UT (clock); GATE (speaker: mixer command) | |
| 406-467 | release_panel, the hits of each dialog kind | UT; Help "Helper" arm GATE (start_program of chat, lines 428-429) | |
| 472-490 | redraw_panel, Taskbar, Clock, Help | UT | |
| 506-521 | set_taskbar, on_top change and open taskbar dialog | UT | |
| 531-542 | refit | UT | maximized window of the old work area |
| 550-559 | auto_hide | UT | |
| 564-580 | slide_frame | UT | |
| 598-606 | press_wallpaper | UT | |
| 610-621 | press_icon | UT | |
| 625-645 | open_icon, every arm | GATE | each arm starts a program (open_with, xdg_open, start_named, start_program) |

wm/pointer.rs
| lines | function | class | reason |
|---|---|---|---|
| 14, 18, 20, 22 | press_left, tray, menu, desktop icon, wallpaper | UT | |
| 34-71 | press_decoration, arms and early returns | UT | |
| 60-61 | press_decoration, resize start | UT | window edge |
| 78-95 | press_icon_of, system_below_icon | UT | single click opens the system menu, double click closes |
| 130-131, 141, 154 | press_title (maximized), press_taskbar (no bar), Speaker | UT | |
| 156-157 | press_taskbar, Helper | GATE | start_program of chat |
| 176-208 | dragged: dialog drag, desktop drag, resize drag, same point, no drag, removed window | UT | pull_out at 205 UT only if the core takes the give |
| 237-251 | release: context row, no act, hand over, far output | UT | |
| 260-270 | release: dialog group wobble, resize, dialog, icons | UT | |
| 273-291 | release: dialog choice, button, menu row | UT | |
| 306-332 | drop_on_far_output, release_icons | UT | |
| 349-369 | pull_out, give refused, grab | UT | |
| 380-392 | hand_over with a claim, claim error, no index | UT (claim error GATE if the fake core cannot fail) | |
| 464, 476 | release_button, no index, maximize | UT | |
| 484-511 | start_program, start_named's notepad chain | GATE | spawns the program |
| 515-520 | start_named | GATE | spawns the program |
| 523-529 | check_launches, the loop with a change | 523-525 UT; 526-528 GATE | a change needs a started program |

Step 2, the tests (branch claude/moved-tests-mk3, commit c00dcf3e; new files only:
wm/panels/tests_moved.rs (35 tests), wm/pointer/tests_moved.rs (+19 tests), one `#[cfg(test)] mod`
line in wm/panels.rs; no production code changed). The panels were written first. Pointer was
over 70 % after its first batch, so the ranges left there are named, not written.

| file | before | after | at 70 %? |
|---|---|---|---|
| wm/panels.rs | 40.68 % (312 missed) | 93.35 % (35 missed) | yes |
| wm/pointer.rs | 63.30 % (167 missed) | 79.12 % (95 missed) | yes |

Ranges left (after), with the reason for each:
- panels 54: the Done arm of the second match in `tell`; unreachable, `tell` returns for Done above it.
- panels 58: `launches.noted`; the note id is private launcher state, so only a real start reaches it.
- panels 75-77, 618, 625-630, 632, 634-637, 640-642, 644-645, 427-430: gate only: each starts a program (start_program, start_named, xdg-open, the chat helper).
- panels 84-85, 88-89: gate only: Lock and Screenshot run a shared script (session::run) when it is installed.
- panels 386: gate only: the speaker sends a job to the volume backend.
- panels 263, 296: unreachable: Apply comes from Display only; open_display_list is called from Display clicks only.
- pointer 156-157, 484-511, 515-520, 527-528: gate only: start_program, start_named, check_launches with a finished start.
- pointer 251, 306-308: gate only: a drop on an output that shows no part of the window needs a second output; the fake core lists one.
- pointer 60-61, 185-190, 205, 237-241, 264, 273-277, 284-291, 328, 354-369, 380-392: unit test, not written (the file is over 70 %); the next lines to take, in this order: start_resize and the resize drag (60-61, 185-190), the context row and the dialog choice (237-241, 273-277), hand_over with a claim or a refused claim (380-392).

The raise returns of open_date_dialog, open_display_dialog and open_effects (158, 172, 312) are covered: a test asserts the single Move command they return.

Gates: `cargo clippy -p chora-xp --all-targets`: 0 warnings (clippy --fix on the three test files for `assert!(is_empty())`, and one reasoned allow on a cast). `cargo fmt -p chora-xp` applied. `cargo test -p chora-xp`: 301 passed (253 before). `tests/merge-gate.sh --jobs 3` on c00dcf3e: GATE-EXIT=0, 41 steps PASS, nothing pushed by the gate (log: the gate's run directory /tmp/merge-gate.TBhufP).

Notes for the PM:
- The branch merges origin/claude/autonomous-dev-session-3w-i33pg7 (85764974) as step 0 asked; that merge is in the branch, so the branch is ahead of 4904ef79 by the merge and the tests.
- A commit hook (no inline comments) refused the first commit; the comment became a doc comment, no hook was skipped.
- Commits carry no attribution lines, as the brief says.

## Review mk r3 (PM, 18:03; builder haiku; c00dcf3e, own commit 3 files +989 -3)
Checks: the own commit touches only `tests_moved.rs` files plus one `#[cfg(test)] mod tests_moved;` line in panels.rs (no production code);
no sleep or #[ignore] added; lane gate on c00dcf3e GATE-EXIT=0 (41 steps); cargo test -p chora-xp 301 (253 before);
coverage claims panels 93.35 %, pointer 79.12 % (both over the 70 % target; figures are the lane's llvm-cov run, the merge gate re-runs the tests).
Verdict (computed): no open MAJOR -> MERGE-OK c00dcf3e. Train 69.
