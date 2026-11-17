# mv2-mc r1: moves without logic in chora-mc

Branch claude/mv2-mc at 20fac444, from c78eca98. Seven commits, one per move.

main.rs had 3,456 production lines; it now has about 1,350 (tests stay in main.rs).

| new file | lines | content |
|---|---|---|
| lifecycle.rs | 201 | timers, start, grabs (impl Mc) |
| build.rs | 383 | apply, build, recolor, panels, type metrics |
| layout.rs | 577 | geometry, chapter, place, panel, align, measure |
| show.rs | 272 | write, show, launch scene |
| windows.rs | 201 | adopt, fit, reattach, forget |
| content.rs | 220 | texts, set_panel, typing |
| acts.rs | 130 | drag, act, perform, turn |
| exports.rs | about 190 | exported_methods |

Check: `git diff --color=always --color-moved=zebra c78eca98 HEAD -- plugins/chora-mc/src`.
Not moved: 203 added and 135 removed lines (mod lines, `use` lists, `impl Mc {` wrappers,
`pub(crate)` on 150 or so fn lines, inline comments turned into doc lines); 1,979 lines moved.

Deviation: the pre-commit hook no-inline-comments rejects every added `//` line, moved code
included, and `--no-verify` is forbidden. So the inline comments inside moved methods
became `///` lines of the method (text kept). No other change.

Results: cargo test -p chora-mc 126 passed; clippy -D warnings passes for the whole workspace
(pre-commit hook on each commit).
Gate (one run, head 20fac444, after `git fetch --unshallow`, needed for 2d-guard's base commit):
all steps PASS up to and including mc-paths, mc-sofa and mc-handover; the step e2e FAILED:
"gate-tiling-apps-click: a click on the Zebra row did not start zebra" (tiling look, no mc
code; log /tmp/merge-gate.FwndXb/e2e.log, /tmp/e2e-sequence.YbPdPH). Not re-run (brief: one run);
I think it is a flake, not shown.

Open: region.rs has 2,271 production lines (over 1,500); not in this brief, a candidate for the next move.
