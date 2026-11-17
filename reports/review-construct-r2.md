# Review construct r2 (59a16702)

Verdict: MERGE-OK

Code reading only. No cargo, no build, no gate run (merge train building, disk tight). Cells judged by reasoning plus the builder's recorded runs.

## Findings

| sev | where | finding |
|---|---|---|
| MINOR | tests/gate-construct-pull.sh:207-216 | The new cell never asserts that the cube actually stood back. The step back is `inject Alt+Escape`, then `sleep 3`, then one read of `pane_drawn`. If Alt+Escape did nothing, "drawn" is trivially true and the cell is green for the wrong reason. Red-on-base rests on the builder's 4/4 runs, not on the cell asserting the mode. A positive wait on the cube's stood-back state (or the construct-look node) would pin it. The fixed sleeps (1 s, 3 s) are sleep-as-wait, but only before a read. The final checks use bounded `until_ok`. |
| MINOR | plugins/chora-construct/src/main.rs:228-233, file.rs | `placed` is not saved with the pane pose. A pane that was pulled out is saved and loaded with `placed: false` (adopt.rs, scene.rs, wm.rs all init false). After a restart, a saved pane in front of a stood-back cube is hidden again (the N2 symptom on a rarer path). The pane is not lost, since the window count is unchanged. Not the reported case. |
| COSMETIC | doors.rs:455-470 | There is no unit test that the strip width or hit rect covers the 7th word. The layout audit is only in the builder's report (1280x800, 1024x768). |

No BLOCK or MAJOR.

## 1. Do the fixes address the findings

**N2, fixed.** `Pane::put` is the single mover for drag, pull and `pane.move` (panes.rs:216, via `move_pane`). The pull path calls `move_pane` at pull.rs:~320, so a pulled pane is `placed` before the first frame. `covers_cube` now returns false early for a placed pane (frame.rs:170-173). Checked the questions asked:
- Back faces: `back_faces` (frame.rs:199) is `covers_cube || space::shows_back`. Only the first term is skipped, so a placed pane with its back to the camera is still hidden.
- Panes of other faces: faces are cube planes, not `Pane`s. Unplaced panes (restored, fly-in, wm child) keep the old rule. The unchanged gate-construct-cube cell "the pane gives way to the cube" still covers that.
- Step-in/step-out animation: the skip is unconditional on mode, so a placed pane is shown in the animation and in the steady state, with no flicker between them.
- Behind the cube: this was already not hidden (`in_front` false), so nothing changes.
- Consumers: frame.rs:92 (the near list), frame.rs:126 (the glow) and back_faces all use `covers_cube`, so they stay consistent.
- Accepted trade-off: a placed pane overlapping the cube stays drawn over it. The builder states this in Deviation 2 and in docs/cube-modes.md. It follows from the brief ("it can be dragged back over a face").
- `drop_pane` off-output restores `at` and `yaw` directly (input.rs:532). The pane was already placed, so that is fine.

**rv5-3, fixed.**
- Localized like the rest of the strip: `("Helper", "Helfer")` in `Door::word`, with a unit test for both languages.
- Mouse path: `ALL` gives the word a hit rect (HITS layer, `InputMode::Accept`) and a text node. `door_input` routes a click through `grabbed_key(Action::Chat)`, the same path as Super+a. The injected-input gate is shared with the other doors.
- Key path: unchanged (`config.rs:859`, Super+a).
- Honesty: the word is static, but its action is the key's action, so with no helper the chat opens and says "No helper" (the existing cell `cell_helper_none` covers the key). Nothing claims a helper is running. The island keeps the state word.
- `grabbed_key` goes through `shared_key`, which shows `not_started` when `chora-chat` does not start. That is an honest failure notice.
- Docs updated (configuration.md, doors.rs header).

## 2. Red tests first

- **cccb3c98** (red, pull cell). On base, pull then Alt+Escape gives `covers_cube` true (the pane sits in front of the face and overlaps the cube), so the node is `visible:false` with no rects, and `pane_drawn` is False. It fails on base and passes after d33bbc8e. This matches the builder's 4/4 and 4/4. The caveat is the first MINOR.
  - Pixel positions: the drag target is derived from the pane rect and the face rect (fractions), not absolute. The only absolute point, [260,340] on the edge output, reuses the existing cell convention.
  - The `pane_middle` helper moved earlier in the file with the same body, so no loss.
- **8e97f969** (red, `check_helper_button`). On base `construct-door-helper` does not exist, so `until_ok 'center ...'` times out after 20 tries and the check fails. After the fix it passes. The waits are bounded `until_ok` calls, there is no sleep, and the position comes from `center` of the node (scale-safe). It starts with `chat_open && return 1`, so a leftover chat cannot give a false green. It runs first, so it does not inherit chat state from `helper-exit`.

## 3. Changes under tests/

- tests/doors/construct.sh: one check added, plus a header comment. Nothing removed.
- tests/gate-construct-pull.sh: cells and helpers added. `pane_middle` is moved, not dropped. The old "2a" comment is renumbered to "2b" with its text intact. No existing assertion is weakened.
- The Rust unit tests only gain a `placed: false` field and two new tests.

## 5. HUD label path

The diff does not touch the HUD label path (`construct-label-text`, overlay.rs, hud.rs, gate-construct-hud.sh). The red "pane ahead has no label" on base is not caused or changed here. Not a gate step. The e2e step-24 failure and the two skips are in unrelated lanes (tiling, container tools).

## What I checked

`git log` and the full diff (code, tests, docs), `covers_cube` and all its callers, every `Pane { .. }` constructor and `put` call site, the pull and drop path, the doors layout, the click dispatch and `shared_key`, the red-first reasoning for both cells, and the builder's report. The captures were not viewed.
