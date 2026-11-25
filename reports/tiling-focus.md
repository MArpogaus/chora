# tiling-focus

## r1

Lane: cloud, branch claude/tiling-focus-r1. Base 79ed970f (t143 + act-wait r1 + session-layout r1), head 6261d3e0. ti-57 MAJOR (p2-r11-tiling "## r11": ricer L3 red 2 of 2 "expected the focus with tiling, holders ['xp']").

Commits: red 69feb0e0 (test: tiling unit tests, xp given-in cell, tests/gate-looks-move.sh in the step handover), then fix 990d57cf, test a04ae9b5 (the cell polls, sleep ratchet), test 7e2e1644 (cut-check: focus.changed from the kit), fix 0ff61ca2 (only a move from a menu keeps the keys: the handover matrix), fix 6ace1a0f (a passing node of the giver keeps nothing: matrix construct xp drag/key), docs 6261d3e0.

Cause: a give hands the window with a remove with to; the core drops the focus, and every receiver focuses a window given in (kit Book::raise_given, and xp adopt). xp on HEADLESS-2 took the keyboard while the person still looked at tiling on HEADLESS-1; tiling held nothing, so its keys and the key list worked on a desktop the keyboard had left.

Rule (D119 amended, docs/api.md "Window managers"): a window that the person sends from a menu of a look to a look on another output does not take the keyboard with it; the keyboard stays with the look the person acted in, on its next window or, without one, on its desktop (its root node), until the person clicks on that output. The menu passes stay to window.give (give.sh of the Looks door and the Apps rows, choractl give TO --stay); the giver keeps the keys in a transaction before the give when no claim of the receiver overlaps the output of the window (kit wm::give::claims_over, Focus::keep); a receiver gives a window given in no focus while a window or the desktop of another look holds the keys (Focus::takes_given, Book::confirm_keys; xp). A give without stay goes with the window as before (give keys, drag, automation, voice, MCP): the handover matrix holds that contract (each look on its own output, moved window focused), so the brief's general sentence is narrowed to the menu. Open for the PM: should the give keys (Super+Alt+N) keep the keys too; xp as giver to tiling on another output does not keep them yet.

Holder (tiling+xp, foot on tiling, "Looks: xp" -> "Move window to the xp desktop"):
- before (79ed970f): before the move tiling/s1; after the move tiling none, xp/s1 (core focus s1 of xp on HEADLESS-2); after Super+2, Super+1 the same, check.py "task L3 FAIL: expected the focus with tiling, holders ['xp']"; after Super+F1 and Escape the keys went back to s1 of xp.
- after (6261d3e0, scripted run, round.sh snapshot of the head): before the move tiling/s1; after the move tiling (desktop), xp none, no surface focused; "task T7 pass", "task L3 pass"; Super+F1 opens tiling-keys, Escape closes it, holder tiling, xp none. Two terminals (gate cell): after the first move tiling/s1 (the one left), xp none.

Tests: red on 69feb0e0's tree (own worktree): tiling a_window_handed_to_a_look_on_another_output_leaves_the_keyboard_with_tiling and the_last_window_handed_to_another_output_leaves_the_keyboard_with_the_desktop FAIL (only the remove, no focus), xp a_window_given_in_leaves_the_keyboard_with_another_holder FAIL (xp focused s1); gate-looks-move on 79ed970f binaries: "FAIL: the first move: xp took the keyboard: tiling {holder:null}, xp {holder:xp, surface:s2}". After: all green; kit wm_focus (keep, takes_given) and wm_give (claims_over, confirm_keys) cells added. Tiling alone, xp alone, mc, co cells and the handover matrix unchanged and green.

Captures: media/2026-11-25/tiling-focus-r1/before (002 picker, 003 key list, 004 after Escape, holders.txt), after (before-move, after-move-1, after-move-2, keys, after-keys from the gate cell; scripted-keys, scripted-after-escape, scripted-T7-L3.txt); each under 500 KB.

Diffstat 79ed970f..6261d3e0: 27 files changed, 729 insertions(+), 47 deletions(-) (kit wm focus/book/give and give.rs, choractl give --stay, flavors/tiling/give.sh, tiling drag/wm/overlay/main, xp lifecycle/windows/state/wm, Give pattern fields in mc/construct/xp, docs/api.md, docs/decisions.md, tests/gate-looks-move.sh, tests/merge-gate.sh).

Gate: CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 on 6261d3e0 -> EXIT 0, 66 steps PASS, "all steps passed; no --push, nothing is pushed" (matrix PASS 278 s, handover PASS 67 s with gate-looks-move). An earlier run on the same head stopped at step test: crates/chora words::tests::a_backtrace_into_a_full_pipe_waits_for_its_reader (words.rs:594, a timing test of the core, which this branch does not touch; 5 of 5 green alone, green in the rerun). pgrep 'chora --headless' empty at the end.

Last 20 lines of the gate:
```
mc-column  PASS    43s /tmp/merge-gate.obA62L/mc-column.log
launcher-click PASS    16s /tmp/merge-gate.obA62L/launcher-click.log
voice-events PASS     1s /tmp/merge-gate.obA62L/voice-events.log
voice-acp  PASS    12s /tmp/merge-gate.obA62L/voice-acp.log
agent      PASS    36s /tmp/merge-gate.obA62L/agent.log
tiling-keys-more PASS    39s /tmp/merge-gate.obA62L/tiling-keys-more.log
tiling-system PASS    14s /tmp/merge-gate.obA62L/tiling-system.log
handover   PASS    67s /tmp/merge-gate.obA62L/handover.log
mc-scale   PASS    10s /tmp/merge-gate.obA62L/mc-scale.log
watchdog-starved PASS    48s /tmp/merge-gate.obA62L/watchdog-starved.log
tiling-windows PASS   166s /tmp/merge-gate.obA62L/tiling-windows.log
xp-run     PASS    12s /tmp/merge-gate.obA62L/xp-run.log
demo       PASS   180s /tmp/merge-gate.obA62L/demo.log
tiling-bar PASS   151s /tmp/merge-gate.obA62L/tiling-bar.log
tiling-theme PASS    18s /tmp/merge-gate.obA62L/tiling-theme.log
tiling-disc PASS    71s /tmp/merge-gate.obA62L/tiling-disc.log
leak       PASS   460s /tmp/merge-gate.obA62L/leak.log
mc-host    PASS    19s /tmp/merge-gate.obA62L/mc-host.log
e2e        PASS   265s /tmp/merge-gate.obA62L/e2e.log
all steps passed; no --push, nothing is pushed
```
