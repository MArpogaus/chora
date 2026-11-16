# multilook r1 report (rc 1), 2026-11-16

Branch claude/multilook at bb8df12a (pushed), on the session branch 66588880. Sonnet worker, one round.
Build: debug, headless, software rendering.

## Done
| Item | Result | Gate cell | Commits |
|------|--------|-----------|---------|
| sh-23 keyboard in two panes | fixed | `tests/gate-construct-pane-keys.sh` (new, step `pane-keys` of the merge gate) | 7601f87 red, 24e7beb, a291e45 |
| co-16 cube leaves the view | fixed | cell "a drag that begins beside the cube leaves it where it is" in `tests/gate-construct-cube.sh` | 7af4e36 red, bb8df12 |
| ti-20 Apps in the tiling bar | not real | cell `launcher-door` of `tests/gate-doors.sh tiling` | none |

## sh-23
Cause: xp grabs `Super+Super_L` (tap opens Start), tiling grabs `Super+Return`; the core gave a grabbed key to
the first grabber whatever the pointer was over. After a click into the tiling pane the Super tap opened the xp
Start menu, the menu took the keyboard, which ended the "chosen manager" of the click, and the terminal of
Super+Return opened in xp. Red: `FAIL: a tap of Super after a click into the pane of tiling opens the Start menu of xp`
(7601f87). Fix in the core (`crates/chora/src/input.rs`): a key grab of a client whose claims all lie on virtual
outputs (a manager in a pane or on a face) waits while the pointer is in the claim of another manager. The
construct claims its own screen, so its keys (Alt+Escape) are not affected; a first version without that
distinction broke them and is replaced by a291e45. Unit test:
`offscreen::tests::a_manager_of_virtual_outputs_gives_way_to_the_one_under_the_pointer`.
Capture after the fix: `reports/multilook-r1/panes-after-fix.png` (Super+Return in the tiling pane: the terminal
of tiling fills the pane, xp stays on its desktop). The capture before the fix (xp Start menu and a terminal in
the xp pane) was overwritten by the green run and is not kept; the red gate line above is the record.
Not changed: a tiling pane with no window is black. Wallpaper and bar are companions that the session script
starts (`waybar.sh`, `wallpaper.sh`), not part of the pane; with a window the pane shows tiling. Open as a
design question: should the pane start the wallpaper itself?

## co-16
Cause: a press on the empty space beside the stood-back cube (x 900 of a cube that spans 416 to 864) started the
free look of the construct, which turned the camera while the cube stood in a fixed view. Red:
`FAIL: a drag that began beside the cube moved it out of the view: face 0 is 0 px wide, at rest 376` (7af4e36).
Fix (bb8df12, `plugins/chora-construct/src/input.rs`): with a cube open, a press on the space starts no look.
A drag that starts on the cube and ends off it already settled on a face (existing cell in the same gate).
Capture after the fix: `reports/multilook-r1/cube-after-fix.png` (the tour drag of chapter 6 from (900,400):
the cube stays at the front, label "XP 2 face 2 of 4").

## ti-20
`launcher-door` of tiling, wait cut from 5 s to 1.5 s (6 polls) in a scratch edit, not committed: 10 passes of 10.
With the CI wait of 5 s: 10 of 10. Not real on the headless setup; closed.

## Runs
- `gate-construct-pane-keys.sh`: 10 of 10 pass. `gate-construct-cube.sh` (with the new cell): 10 of 10 pass.
- `tests/merge-gate.sh`: all steps passed (third run, all 29 steps, 2d-guard base fetched with `git fetch --unshallow`).
  First run: 2d-guard failed for the missing base commit in the shallow clone. Second run: `chat-keys` failed once
  ("the text typed before the pick did not reach the picked helper"), 3 of 3 pass alone and pass in the third run;
  not related to this diff as far as I can tell, not investigated.
- No test was skipped.
