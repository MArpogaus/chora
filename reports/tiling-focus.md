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

## r2

Lane: cloud, claude/tiling-focus-r2. Base 6261d3e0 (r1 head), head 712a01f5. Diffstat 6261d3e0..712a01f5: 15 files changed, 472 insertions(+), 87 deletions(-).

Commits: 08f3b514 test (red on 6261d3e0), 7407db98 fix, a8994b9b docs (D119, api.md), 11a0bb2a test (kept cells without new sleeps, sleep-ratchet), 1e655fef test (focus_changed of the kit, cut-check), 712a01f5 test (gate-handover-virtual: `xp tiling key` and `tiling xp key` kept, between two virtual outputs).

Rule (D119 amended, id kept; api.md "Window managers"): a give the person makes, a menu row or a give key, keeps the keyboard with the giver (next window, else its desktop, the root node of the look); the receiver yields through `Focus::takes_given`; automation, the voice and MCP give without stay and go with the window as before when they give the focused window. One kit path: `give::focused` hands through the new `Windows::hand_kept` (default: `hand`), and a give key on a window of another look calls its `window.give` with `stay: true`.

Red first (08f3b514 fails on 6261d3e0): kit `a_window_of_another_manager_is_given_by_its_owner` (forward carries stay); tiling `the_give_key_leaves_the_keyboard_with_tiling` (rtf1-1) and `a_window_given_in_while_xp_holds_the_keyboard_comes_without_it` (rtf1-2, receiver); xp `the_row_window_to_tiling_leaves_the_keyboard_with_xp` (rtf1-2, giver: next window, then the xp root node, before the remove); matrix EXPECTED `tiling xp key kept`, `xp tiling key kept` (new result kept: owner is the receiver, the window has no focus, a node or window of the giver has it, and holds a second); gate-looks-move moves a third terminal with Super+Alt+2 first.

Findings:
- rtf1-1 fixed: tiling `Windows::hand_kept` -> `Tiling::hand_kept` (keep_keys, then hand_dragged); `window.give` with stay goes the same way.
- rtf1-2 fixed: xp `give_kept`/`keep_keys` (lifecycle.rs) for the give keys and the Control Panel rows (`give::focused`) and for `window.give` with stay; tiling yields as a receiver (`lifecycle.rs` settle: `Focus::takes_given`, `Book::confirm_keys`, `Focus::lose`).
- rtf1-3 fixed: `Focus::takes_given` now has callers in tiling and xp; xp hears the focus into a kit `Focus` (`heard`) in place of its own `desk_away`/`key_node_id`.
- rtf1-4 fixed: a refused give after a keep focuses the window again (tiling `a_refused_give_of_the_person_gives_the_keyboard_back`; xp the same in `give_kept`).
- rtf1-5 settled in D119 (option 1): a give of a window that does not hold the keyboard leaves it with its holder, for every give.
- Open, filed: chora-mc and the construct do not keep as givers (default `hand_kept`) and take a window given in with the keyboard as receivers (`Book::confirm`); matrix cells `tiling|xp -> mc|construct key` stay pass (giver keeps, receiver takes). Construct was out of the lane (CO-DROP r1); its list.rs give menu stays filed.

Holder before (6261d3e0 binaries, gate-looks-move with Super+Alt+2): `FAIL: the give key: xp took the keyboard: tiling {"holder":null}, xp {"holder":"xp","surface":"s3"}`.
Holder after (fix): give key `tiling {"holder":"tiling","surface":"s2"}, xp {"holder":null}`; first menu move `tiling s1, xp null`; second move `tiling {"holder":"tiling"}` (desktop), xp null; `task L3 pass`; Super+F1/Escape keeps it off xp. Matrix key cells (ONLY): tiling xp kept, xp tiling kept, the other ten pass.

Captures: media/2026-11-25/tiling-focus-r2/ (before/: holders.txt, before-key.png, after-key.png; after/: scripted-T7-L3.txt, matrix-key-cells.txt, seven png; each under 500 KB, largest 392 KB).

Virtual outputs: xp and tiling on two virtual outputs shown in panes on one screen count as other outputs (claims_over per output), so the give keys between them keep the keyboard too; if the PM wants a pane on the same screen to count as the same place, that is a new rule.

Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` on 712a01f5: EXIT 0, 66 PASS, 0 FAIL, 0 SKIP, "all steps passed; no --push, nothing is pushed" (handover 86 s incl. gate-looks-move, matrix, virtual). No foreign flake. Two earlier runs failed on my own changes and were fixed: ratchets (sleep 781 of 778, kept_by) and ci-check (cut-check, Event::FocusChanged in a tiling test), then virtual (the key cells between virtual outputs measured gap). `pgrep -fa 'chora --headless'` empty after the run.

## Review r2

Lane: cloud, read-only on code; claude/tiling-focus-r2 head 712a01f5, base 6261d3e0 (15 files, +472 -87), one round. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. No merge gate run (the lane's: EXIT 0, 66 PASS).

Checks:
1. Red first: 08f3b514 is an ancestor of 7407db98. At 08f3b514 (`cargo test --no-fail-fast -p chora-kit -p chora-tiling -p chora-xp`): kit `a_window_of_another_manager_is_given_by_its_owner` FAIL (give.rs:545), tiling `the_give_key_leaves_the_keyboard_with_tiling` and `a_window_given_in_while_xp_holds_the_keyboard_comes_without_it` FAIL, xp `the_row_window_to_tiling_leaves_the_keyboard_with_xp` FAIL; nothing else red. At 712a01f5: kit 336, tiling 218, xp 365 passed, 0 failed. Pass.
2. rtf1-1, rtf1-2: tiling `give::focused` -> `Tiling::hand_kept` (drag.rs:32: keep_keys, hand_dragged, focus back on an error); xp give keys (keys.rs `Acting::hand_kept`) and Control Panel rows (choices.rs:81 `give::focused`) -> `Wm::give_kept` (lifecycle.rs:115); `window.give` with stay and no grab goes the same way in both; a give key on a window of another look forwards `stay: true` (give.rs:251). Receivers: tiling `settle` (lifecycle.rs:45) and xp `takes_keys` (lifecycle.rs:89) both call `Focus::takes_given`; xp now hears the core focus into a kit `Focus` (dispatch.rs:26). Refused give (rtf1-4): the core refuses a `remove` to a name with no connection (transaction.rs:347) inside `conn.give_dragged`, which both `hand_dragged` and xp `give_dragged` call before they return, so the rollback runs for a core refusal too; the keyboard flicks to the next window and back on the same output, no stray. The next window is one that shows on the output of the window (tiling `focused_without`, xp `topmost_except`). Between tiling and xp I found no path where no look holds the keyboard or keys reach the unseen output. Pass for tiling and xp; mc and the construct see rtf2-1.
3. A give without stay (automation, voice, MCP, `choractl give` without `--stay`) still calls `hand`/`give_dragged` (drag.rs:60, dispatch.rs:168 via `give_call`); the core focuses none at the give of the focused window, so `takes_given` (core none, no desk away) takes it; a window that does not hold the keyboard comes without it to tiling and xp while a window or desktop of another look holds (rtf1-5 as D119 now says). D119 and api.md "Window managers" match the code, except the path named for xp's rows (rtf2-2).
4. Default `hand_kept` for mc and the construct: not safe by the RC bar; keys reach a window on an output the person does not see (rtf2-1, MAJOR by the bar of rtf1-1). Matrix (four outputs, each look on its own): `tiling mc key`, `xp mc key`, `tiling construct key`, `xp construct key` pass with owner=receiver focus=true (the giver keeps, then the receiver's `Book::confirm` takes the keyboard), and `mc tiling key`, `mc xp key`, `construct tiling key`, `construct xp key`, `mc construct key`, `construct mc key` pass (the giver keeps nothing, the core focuses none, the receiver takes); virtual: `tiling mc key`, `xp mc key`, `mc tiling key`, `mc xp key` pass. Not a regression of r2 (the same before), and D119/api.md call it open.
5. `kept`: matrix `kept_by` waits with `until_ok` for owner=receiver, window not focused, a focused node of the giver, surfaces clean and on top, then `holds` it 1 s; virtual `kept_now` the same with the rect on the receiver's output; a `kept` cell that is not kept falls to `arrived` and is a mismatch. It asserts the holder, no new sleep. On 712a01f5: `ok: sleep-ratchet (sleep 778, sleep-all 929)`, `ok: size-ratchet (0 files over 1500)`, `ok: lint-ratchet (allow 360, no-reason 0, eprintln 46)`, `ok: cut-check (8 crates, 16 reported)`.
6. `git merge-tree --write-tree 6daaaab8 712a01f5`: clean (tree d62d2978). Merged tree: `cargo check --workspace` green, the three ratchets ok with the same counts. co-drop r1 changes construct give-from-a-pane focus (pull.rs `given_from_a_pane`) through its own `give`, not `give::focused`/`hand_kept`; no clash of rules with D119 (the construct stays "hand as before").
7. Captures media/2026-11-25/tiling-focus-r2/: before/holders.txt `FAIL: the give key: xp took the keyboard: tiling {"holder":null}, xp {"holder":"xp","surface":"s3"}`; after/scripted-T7-L3.txt give key s2 tiling / xp null, first move s1, second move desktop, L3 pass, Super+F1/Escape off xp; after/matrix-key-cells.txt two kept, ten pass. As the report says; 12 files, largest 399197 B. Pass.

Findings:
- rtf2-1 MAJOR plugins/chora-mc/src/windows.rs:66 (receiver `Book::confirm`; giver: default `Windows::hand_kept`, crates/chora-kit/src/give.rs:88): mc neither keeps the keyboard as giver nor yields as receiver, so Super+Alt+3 in tiling or xp (whose keep r2 now runs) and Super+Alt+1/2 in mc send the keys to a window on another output the person does not look at, the rtf1-1 class, against D119 as now stated for every give of the person; cells `tiling mc key`, `xp mc key`, `mc tiling key`, `mc xp key` (matrix and virtual) expect it; the construct cells are the same class and stay filed with CO-DROP. Fix: mc implements `hand_kept` (claims_over, keep on its next window or root node, back on refusal) and adopts a given window with `confirm_keys(id, focus.takes_given(..))`, and those four cells expect `kept`; or the PM accepts the gap for the RC and files it.
- rtf2-2 nit docs/decisions.md:168, docs/api.md:441: both list the "window to" rows of xp's Control Panel under `window.give` with `stay`, but they run `give::focused` -> `Windows::hand_kept` (plugins/chora-xp/src/wm/choices.rs:81), as the give keys do. Fix: name them under the give-key path.

Counts: BLOCK 0, MAJOR 1 (rtf2-1), nit 1. rtf1-1..5 fixed for tiling and xp. Verdict: **FIX** (open MAJOR). Known, not findings: FLAKE-CUBE r1, FLAKE-SOFA, FLAKE-PERSONA-NOTEPAD r1, words.rs:594, construct list.rs give menu (filed).
