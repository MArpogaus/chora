# doors r2 (rc 1: shared RC bar), claude/doors at dd8669a4

Worker: Fable lane, cloud. Stopped at the 250 k context limit: one cell and item 4 are left (see "Left").
Builders (sonnet, one worktree each, local branches merged into claude/doors): xp, mc, construct. Tiling, the
shared helpers, the review and the merges are the worker's own.

## Result
`tests/gate-doors.sh`: 79 of 80 cells pass, plus the check `xp log-off`. Red: construct `helper-none`.
Merge gate on the merged tree: every step passes except `doors` (the one cell). Run in parts, since the gate
stops at `doors`: red-first to chat-keys in one run at 12949587; last-words, cube, winit, wm-pane, walk, keys and
pixels with `--only`; pull and cube again after dd8669a4. The cube step (F14) passed here. Not rerun as one whole
gate after the last two commits (bd165227 test only, dd8669a4 construct only: its unit tests, clippy, pull, cube
and the construct cells ran).

## Table action x look (key / door; every cell green unless marked)
| Action | xp | tiling | mc | construct |
|---|---|---|---|---|
| Key list | Super+F1, Super+/ / Start > Help and Support > Keyboard shortcuts | Super+F1, Super+/ / bar Keys | Super+F1, Super+/ / key row "All keys" | Super+F1, Super+/ / strip Keys |
| F1 free | not bound | not bound | not bound | not bound |
| Launcher | Super+Space / Start > Run... ; row Chat with a helper | Super+Space / bar Apps | Super+Space / foot Programs | Super+Space / strip Programs |
| Helper | Super+a / tray clip, tooltip word | Super+a / bar word (`chora-chat --door`) | Super+a / chat door with word | Super+a / island |
| No helper | tooltip "No helper" | bar "No helper" | door "No helper" | RED: island says "Listening" |
| Lock | Super+Shift+Esc / Start footer Lock | Super+Shift+Esc / power menu Lock | Super+Shift+Esc / foot Lock | Super+Shift+Esc / strip Lock |
| Power | Super+Shift+e / Turn Off Computer (Lock, Log Off, Stand By, Turn Off, Restart) | Super+Shift+e / bar icon | Super+Shift+e / foot Power | Super+Shift+e / strip Power |
| Screenshot | Super+Print / Start > Screenshot | Super+Print / Apps entry | Super+Print / foot Screenshot | Super+Print / strip Screenshot |
| Volume | XF86Audio keys / tray speaker | XF86Audio keys / bar word | XF86Audio keys / foot "Volume N%" | XF86Audio keys / strip "Volume N%" |
| Clock | tray | bar | header | strip |

Cells: tests/doors/{xp,tiling,mc,construct}.sh, 20 each; names in tests/doors/lib.sh.

## Commits on claude/doors (after ea47e27b)
- a2db73a9 test(doors): cells for tiling (red: keys-key, keys-names, helper-door, helper-none); lib helpers
- fb743d9d, 5dc29b11 feat(tiling): Super+F1 key list, "Volume keys" row; helper word in the bar
- 071e0e3b test, 138a7c22 feat(xp); merge eefb96df. Red before: 14 cells
- 503e0c4c test(doors): the gate fails on a `check LOOK NAME fail` line
- 6ee86764 test, 3345aa78, 121158d7, f681757a feat(mc), 35bb2613 fix(mc); merge 8c6b2f22. Red before: 14
- 3a1b1b50, 3c3f049d test, 2c5d810a fix, 8db1e1ba, 734c7885 feat(construct); merge 12949587. Red before: 20
- bd165227 test(doors): each xp cell closes leftovers first (3 xp cells flaked in the merge gate; 3 of 3 after)
- dd8669a4 fix(construct): the capsule hides while a cube is open. Red: gate-construct-pull.sh failed after the
  construct merge (the capsule took the press on a title bar of a face); it passes on ea47e27b and again now

## Review findings of r1
- dr-1: done, the four cell files exist. dr-2: done except the one cell. dr-3: Word::NoHelperDoor is used by
  xp, mc and `chora-chat --door`. dr-4: `shared::command` and `shared::data_dirs` now have callers in xp, mc and
  construct; `shared::script` is called only by `command` and could become private. dr-5 accepted. dr-7: each look
  documents its doors in its own section of docs/configuration.md; no shared section yet.
- xp-1: does not reproduce at this head (the dialog opens); Log Off ran `choractl quit`, now the shared script
  (check `xp log-off`). xp-3: F1 is no binding in any look. ti-1: not verified by a real fuzzel click (stand-in).

## Left
1. construct `helper-none`: the island of the HUD host shows the voice state, never "No helper", and its glass
   chat view has no readable text. Needs chora-assistant: Word::NoHelperDoor on the island when no helper
   exists, and the header text of the chat view in the node meta. mc draws the NoHelper line itself meanwhile;
   with a project but no helper that line can be wrong. Remove it when the host has the text.
2. Item 4 not started: the session branch did not move during the round (0 commits ahead of claude/doors), so
   claude/xtc-fix is not merged and xr-3/dr-6 has no code to fix yet.
3. No independent review of the builders' Rust beyond the worker's read of diffs, cells and captures. The
   capsule bug above came from that gap: review the construct capsule over panes of a window manager too.
4. The xp cell file is slow (about 200 s): Esc does not always close the Start menu, and each cell waits for it.

## Deviations and gaps to decide
- xp power words stay XP words (Log Off, Turn Off), not "Log out", "Shut down". `session.*: null` now runs the
  shared script; it no longer ends or restarts chora-xp itself.
- mc grabs the volume keys only with `--manage-windows` (the core refuses XF86 grabs otherwise). mc's own door
  words and notices are English only, as its key words were.
- construct: Super+F1 from a pane steps out first; the key list shows raw `XF86Audio...` caps (tiling: "Volume
  keys", mc: Vol+, Vol-, Mute): three spellings of one row.
- construct `helper-door` reads the island's voice word, weaker than the other looks (name and state).
- tiling: Esc does not close a chat opened from the bar word (the cell closes it with Close); the bar tooltip
  covers the chat header while the pointer rests there.
- `power.sh logout` without a logind session ends all sessions of the user (as in tiling before).

## Red before this lane, not counted
gate-tiling-power (fuzzel 1.9.2), gate-tiling-bar "no separator after Apps", gate-mc-host "the card covers the
hint", gate-construct-hud "the pane ahead has no label" (each fails the same way on a build without the change).

## Captures at 1x (in the worker, not committed)
/home/user/wt/captures/doors-xp.png, doors-tiling.png, doors-mc.png, doors-mc-keys.png, doors-construct.png,
doors-construct-pane.png, doors-construct-keys.png
