# Review: construct r1 (reviewer, sonnet)

Reviewed sha: 81ff7b60 (branch claude/construct), base 6a067e39. Disk 20 G free at start, so I built (debug) and ran gate cells.
Verdict: MERGE-OK (no open BLOCK or MAJOR).

## Findings

| id | sev | where | finding | evidence |
|---|---|---|---|---|
| rv-1 | LOW | docs/cube-modes.md:198, plugins/chora-construct/src/config.rs:~499 | The only way left to pull a window across faces is the title drag past the face edge in use mode (then drop the pane over another face). It is documented in cube-modes.md, but the key list (rows are keys) does not mention it, and stood back there is no pull at all. Acceptable for the RC (a drag is no key; no lost window), backlog a gesture if wanted (builder deviation 2, already stated). | read; pull cell 2/2a pass |
| rv-2 | LOW | tests/gate-construct-pull.sh | The stood-back variant of "a plain drag of a pane over a face gives the window back" and the `press_on_a_pane` stood-back cell were dropped, so that path has no stood-back cell. Honest in the report. | diff of pull.sh |
| rv-3 | LOW | tests/merge-gate.sh (bba1ce11) | The red commit f59df8fb adds `gate-construct-out.sh` before the `out` step exists in merge-gate (it arrives 2 commits later), so the red could not be run by the gate at that commit. I ran the script by hand at f59df8fb: FAIL as claimed (`s1 xp face` -> `s1 construct HEADLESS-1`, label "face 1 of 4"). Red-first step passes. | manual run; `--red-first-check` PASS |
| rv-4 | COSMETIC | plugins/chora-construct/src/modes.rs:~452 (drag_act doc) | Doc comment line is over-long after the edit (reflow). cube.rs/modes.rs docs otherwise match. | read |
| rv-5 | COSMETIC | key list capture | "Back out one step Esc" (Everywhere) and "Close the cube, stood back Esc" (Cube) both show Esc; not wrong, slightly redundant. | construct-81ff7b60-...-keys.png |

## Holds (verified)

1. co-18 BLOCK fixed: red f59df8fb (fails on the pre-fix code, run by me), fix 929b64e7; cell `out` cell 1 PASS (window count, owner, output equal before/after; label "face 2 of 4"). Capture drag-turn.png seen path-wise in the log reports dir.
2. co-19 fixed: out cell 2 PASS; capture keys.png shows the list with "Show these keys, stood back F1" and "Close the cube, stood back Esc". F1 is plain, navigate mode only, no modifier, so use-mode and pane F1 still reach the program (rule 25 text updated in docs/construct-style.md).
3. co-20 fixed: out cell 3 PASS: first Esc closes the list, second closes the cube, every window still there (owner not an ended xp, or on the screen).
4. sh-24 fixed: red 573f788c (unit tests), fix da5a9c00 + 92dcc66f; `cargo test -p chora` 179 pass incl. the four new grab tests; `pane-keys` PASS. Logic read: a held-back grab swallows the key (no stray key to the client); a held-back grab only applies if another grab is not free; injected keys skip the gate except in the `injected_press_is_real` test mode; escape hatch untouched; no unsafe code added.
5. POLS/RC bar: Escape closing the cube is an exit, not a trap; `Return` steps in, documented in cube-modes.md, configuration.md and the key list. With `fly_out: Escape` + grant, Escape steps back and Return steps in (cube cell PASS). Closing the cube loses no window (out cell 3).
6. K12 removal is consistent: Dragged::Pull, plane_window, pull_plane removed; no dead code (clippy -D warnings PASS), take_pulled still used by the use-mode pull. Plain drag turns the cube wherever it starts, so no drag can move a window out stood back (no lost-window path).
7. Gate cells run by me on 81ff7b60 (default target in the checkout): build, clippy, cube, pull, out, wm-pane, pane-keys, walk, keys all PASS. `cargo test -p chora-construct` pass.
8. Not run by me: doors, e2e, pixels, mc-paths, winit (builder reports PASS, doors a known flake F16/F26).
9. Note on method: with the target dir outside the checkout (/tmp/rev-target) the `test` step (shared::the_checkout_of_the_binary...) and the `cube` cell failed (shared scripts not found), which is environmental, not a product fault; both pass with the target in the checkout.
