# mv2-tiling r1 (F4): moves without logic in tiling

Branch claude/mv2-tiling (from c78eca98), six commits, pushed. Head bcf430ed.

| module | content | lines |
|---|---|---|
| focus.rs | ShownFocus | 57 |
| commands.rs | scene command helpers (taken, frame_nodes, changes, ...) | 366 |
| methods.rs | exported method tables | 102 |
| overlay.rs | impl Tiling: bar, theme, chip, key list | 310 |
| drag.rs | impl Tiling: handover and drag | 182 |
| windows.rs | impl Tiling: adopt, relayout, forget, apply | 258 |
| main.rs | 1,098 production lines (was 2,281) | |

wm.rs (1,206 production lines) was not touched. Every file is under 1,500 production lines.

Review: `git diff --color-moved=zebra --color-moved-ws=allow-indentation-change c78eca98..HEAD -- plugins/chora-tiling`
(or one commit at a time). About 268 lines are not moved: `use` blocks, `mod` lines,
`pub(super)` on moved items and fields, `impl Tiling {` wrappers, module headers, three test imports.

Deviation: the hook no-inline-comments rejects moved lines. Nine inline `//` comments
(drag.rs one, windows.rs eight) became extra doc-comment lines on the enclosing fn; no logic change.

Done: cargo test (141 pass) and clippy (hook) pass per commit.
tests/merge-gate.sh, one run on bcf430ed: 39 steps PASS, e2e FAIL at
tests/gate-tiling-apps-click.sh ("a click on the Zebra row did not start zebra").
That script fails alone the same way on c78eca98 (the base), so it is not caused by the moves.
Not rerun (one run per brief). Note: the gate's 2d-guard needs commit 8742dee, so the clone must be unshallowed.
