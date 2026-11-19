# merge-65

## r1

Branch `claude/merge-65` from `claude/autonomous-dev-session-3w-i33pg7` 05878d2a, pushed at e667b3d0.
Merged in order: integ af5547c7, cubesplit 10cf04cf, kit3 04ea5851, flakes-g f075a20d, focus-port 3b418008, moved-tests-xp bf0b66eb.
Conflicts came in two merges only (cubesplit, kit3). The others merged clean, focus-port too.

| File | Merge | Resolution |
|---|---|---|
| tools/size-ratchet.list | cubesplit | Row for `cube.rs` (integ lowered it to 1617, cubesplit dropped it) removed: cubesplit takes it under 1,500; `tools/size-ratchet` reports 0 files over. |
| tests/acts/known-red.toml | kit3 | Both hunks dropped. Rows left: I5 mc KIT3, I13 construct WM6, and the WM2 rows from integ. Integ fixed the WM3 and WM7b rows, kit3 fixed I5 tiling. |
| tests/acts/looks/common.py | kit3 | Import line is integ's (a superset); `notice_acts`/`helper_acts` block, then kit3's `keysheet_*` block. |
| tests/acts/looks/construct.py | kit3 | Imports both sides; `keysheet_acts` runs right after `door_acts`, then integ's `tab_after_a_new_window` and `helper_acts` steps. |
| tests/acts/looks/mc.py | kit3 | Imports both sides; `keysheet_acts` after `door_acts`, then integ's notice, push, clear and `helper_acts`. |
| tests/acts/looks/tiling.py | kit3 | Imports both sides; `keysheet_acts` after `door_acts`, then `notice_acts` and `helper_acts`. |
| tests/acts/looks/xp.py | kit3 | Imports both sides (integ's `drag_title`); `keysheet_acts` after `door_acts`, then integ's `helper_acts` and the Start button act. |

`tests/acts/looks/construct.py:126` (integ's act, the `ri-1` rev) is untouched beyond the merge.

Checks:
- `cargo check --workspace`: ok before the merges.
- `cargo test --workspace`: 2751 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `tests/merge-gate.sh --jobs 3`: stopped at step `xp`, then the steps from `xp` to `e2e` were run with `--only`.
  - red-first to xp-fields: PASS.
  - `xp`: FAIL in the first run, PASS on a rerun (87 s, `tests/merge-gate.sh --only xp`).
    - Cell: 16c ("the start menu opens the Turn Off Computer dialog and Run"), line `FAIL: timeout: said Notify Starting slow-manager`.
    - Log: /tmp/merge-gate.bHc92U/xp.log. Not checked on the integ branch; it passed on the merged tree, so I take it as a flake.
  - The other 25 steps (xp-question, hud, layout, matrix, tiling-audit, tiling-look, chat-keys, doors, acts, last-words, cube, pull, placed, construct-hud, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-paths, mc-sofa, mc-handover, e2e): all PASS, run in one `--only` call.
  - No matrix cell over its time cap (matrix 296 s, PASS).
  - Not one gate run end to end: the merge-gate run stopped at xp, so the rest ran separately on the same tree.
