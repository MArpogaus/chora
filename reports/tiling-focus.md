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

## Review r1

Lane: cloud, read-only on code; claude/tiling-focus-r1 head 6261d3e0, base 79ed970f (27 files, +729 -47), one round. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. No merge gate run (the lane's: EXIT 0, 66 PASS).

Checks:
1. Red first: 69feb0e0 is an ancestor of 990d57cf. At 69feb0e0: chora-tiling `a_window_handed_to_a_look_on_another_output_leaves_the_keyboard_with_tiling` (main.rs:2006) and `the_last_window_handed_to_another_output_leaves_the_keyboard_with_the_desktop` (main.rs:2029) FAIL, chora-xp `a_window_given_in_leaves_the_keyboard_with_another_holder` FAIL (lifecycle.rs:184). At 6261d3e0: tiling 4 passed, xp 1 passed, kit wm_focus 11 and wm_give 10 passed (`--features testing`). Pass.
2. Rule: D119 and docs/api.md "Window managers" match the tiling code for the menu give (give.sh --stay, `keep_keys` in a transaction before the give, `claims_over`, next window else the desktop root; xp `takes_keys` yields to a window or the desktop of another look). The core focuses None before it sends `surface.detached` to the receiver (state.rs Effect::Reserved), so a give without stay of the focused window still lands with the receiver; I found no path for the menu give where no look holds the keyboard. The general sentence of D119 ("a window that the person sends from a menu of a look ...") does not hold for xp's menus (rtf1-2).
3. Narrowing: the give keys of tiling send the keyboard to xp on the other output (rtf1-1, MAJOR by the brief's bar); xp as giver keeps nothing and tiling as receiver does not yield (rtf1-2, MAJOR).
4. Other looks: mc, construct and xp only destructure the new field (`..`, `stay: false`); gate-handover.sh and the matrix cells are untouched; gate-looks-move.sh polls with until_ok/settled, no new sleep. Ratchets on 6261d3e0: `ok: sleep-ratchet (sleep 778, sleep-all 929)`, `ok: size-ratchet (0 files over 1500)`, `ok: lint-ratchet (allow 360, no-reason 0, eprintln 46)`; no .list changed.
5. Captures media/2026-11-25/tiling-focus-r1/: before/holders.txt (holder xp after the move, L3 FAIL) and after/scripted-T7-L3.txt (holder tiling desktop, xp none, T7 and L3 pass, key list opens and Escape closes it, holder stays tiling); all 12 files under 500 KB. Pass.

Findings:
- rtf1-1 MAJOR plugins/chora-tiling/src/drag.rs:46 (`hand`, the give keys via kit give::KEYS): Super+Alt+2-4, which the key list of tiling shows as "Move window to another desktop", hands the focused window without stay, the core focuses None and xp on HEADLESS-2 focuses it (`takes_keys` true), so a person who looks at tiling types into an unseen output, the ti-57 stray by the key instead of the menu (pinned by `a_give_without_stay_leaves_the_keyboard_to_the_receiver`). Fix: the give keys of a person keep the keyboard as the menu does (call `keep_keys` in `hand` when no grab) and the matrix cells across outputs expect the giver as holder.
- rtf1-2 MAJOR plugins/chora-xp/src/wm/choices.rs:81 (rows of control.rs:249): xp's Control Panel rows "Super+Alt+1: window to tiling" are a menu give of xp without stay, xp keeps nothing, and tiling receives with `Book::confirm` (keys on; plugins/chora-tiling/src/lifecycle.rs:43), so the keyboard goes to tiling on HEADLESS-1 while the person is in xp on HEADLESS-2, against D119 as amended; the give menu of the construct (list.rs) is the same class. Fix: xp's menu rows pass stay and keep the keyboard (next window or its desktop) and the kit receivers yield through `Focus::takes_given`, or D119 names tiling's menu only and the rest is filed.
- rtf1-3 nit crates/chora-kit/src/wm/focus.rs:278: `Focus::takes_given`, which D119 and api.md cite as the receiver rule, has no caller outside tests (xp has its own `takes_keys`; tiling, mc and construct call `Book::confirm`). Fix: use it in the kit receivers (with rtf1-2) or drop it from the docs.
- rtf1-4 nit plugins/chora-tiling/src/drag.rs:423: `keep_keys` moves the keyboard before `hand_dragged`; when the receiver then refuses, the window stays in tiling without the keyboard (next window or desktop holds, same output, no stray). Fix: on an error of `hand_dragged` after a keep, focus `surface` again.
- rtf1-5 nit plugins/chora-xp/src/wm/lifecycle.rs:81: `takes_keys` also yields for a give without stay of a window that is not focused (`choractl give xp --surface ID`, MCP, voice) while a window of another look holds the keyboard; D119 and api.md say a give without stay goes with the window "as before". Fix: say in D119 that a receiver yields to a holding window for any give, or limit the yield to stay.

Counts: BLOCK 0, MAJOR 2 (rtf1-1, rtf1-2), nit 3. Verdict: **FIX** (open MAJOR). Known, not findings: words.rs:594 a_backtrace_into_a_full_pipe, FLAKE-CUBE r1, FLAKE-SOFA.
