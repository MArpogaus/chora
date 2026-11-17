# mv2-xp r1: moves without logic in xp

Branch claude/mv2-xp, from c78eca98 (head 690a787a). Not merged.

## Result
plugins/chora-xp/src/wm.rs went from 4,794 lines to 1,391 (870 production lines, the rest the unchanged `mod tests`). The 3,414 lines of `impl Wm` moved into nine child modules, each one move one commit:

| file | lines | | file | lines |
|---|---|---|---|---|
| wm/startup.rs | 298 | | wm/press.rs | 371 |
| wm/dispatch.rs | 305 | | wm/pointer.rs | 502 |
| wm/windows.rs | 630 | | wm/panels.rs | 621 |
| wm/output.rs | 119 | | wm/choices.rs | 249 |
| wm/keys.rs | 370 | | | |

Largest file: windows.rs, 630. All files are far under 1,500 production lines. The module is named `choices`, not `settings`, because `settings` is the chora_kit import of wm.rs. Free functions, types, constants and the existing wm/ children stay where they were; the target names of arch-plan 2.1 (book, focus, give, ...) are chora-kit modules, so none applies in xp.

## Review of the moves
`git diff --color-moved=zebra 54731ad7..690a787a -- plugins/chora-xp/src` (and per commit `git show --color-moved=zebra <sha>`).
Lines changed in the nine move commits: 5,143. Lines not moved: 423 (new-file headers, `use` lists, `impl Wm {`, the `mod x;` lines, and each moved `fn` whose visibility changed to `pub(super)`, counted as a removed and an added line).
No item was renamed and no body or signature changed; `use super::*` was replaced by explicit imports by clippy.

## Deviations
- Two prep commits before the moves (54731ad7, a37708ec): the pre-commit hook `no-inline-comments` rejects inline comments in moved lines, so the 44 inline `//` comment lines inside the moved methods were deleted first (one design note, the fade origin, became a doc line of `fade_step`), and the trailing `// one arm per event` of an `#[allow]` went into the doc comment of `handle`. No code changed. The brief's "moves only" holds for the nine move commits; these two commits lose 44 lines of comments.

## Evidence
- `cargo test -p chora-xp`: 188 passed. `cargo clippy -p chora-xp --all-targets -- -D warnings`: clean.
- `tests/merge-gate.sh` on 690a787a: 39 steps PASS (build, test, clippy, ci-check, 2d-guard, xp, matrix, virtual, xp-faces, xp-paperclip, xp-fields, xp-question, xp-notepad, hud, layout, tiling-audit, tiling-look, cube, pull, placed, construct-hud, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-*...). Last step `e2e` FAIL in cell 24, `tests/gate-tiling-apps-click.sh`: "a click on the Zebra row did not start zebra". The same cell fails alone on c78eca98 built in a clean worktree, so it is red on the base and not from this branch (this branch changes only plugins/chora-xp). Not run: desktop-file-validate and the fkeys cell SKIP in this container.
- Gate attempts: the first stopped at 2d-guard (shallow clone lacked commit 8742dee; fetched), the second was killed at `placed` by my own background job timeout (all earlier steps PASS), the third is the run above.

## Refactor proposals
- plugins/chora-xp/src/wm.rs:338 (`struct Wm`, about 90 fields): split into state groups (taskbar, menus, dialogs, icons) that the modules own; gain: the impl modules stop sharing one record; L.
- wm/windows.rs and wm/panels.rs still mix book-keeping, layout and drawing; the kit::wm port (arch-plan 1.1) takes the book part; M.
- Test module wm.rs:871 (520 lines) could move to wm/tests.rs; S.
- tests/gate-tiling-apps-click.sh red on c78eca98: owner of tiling lane to look; S.
- tests/merge-gate.sh 2d-guard needs a full clone (pinned 8742dee); setup.sh should unshallow; S.
- Cut check: xp uses chora-kit, chora-scene, chora-draw; no new breach seen in this round.
