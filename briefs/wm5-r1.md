# wm5 r1 (F3): kit::wm::stack rules and the scene layers (D89) with `held`

Model: opus, effort high, because (a core change: the scene layer order is the escape hatch's ground, D89, and
mc's root reorder (D83) goes; a wrong order hides the lock or a modal in every look)
Lane: cloud, branch claude/wm5 from 4548feb3 (train 59). One round, stop at 250 k. Cost about 3 %.
Run `git fetch --unshallow` first. When the session branch moves (train 60: mv4, wm2; later trains),
`git merge origin/claude/autonomous-dev-session-3w-i33pg7`. Read reports/arch-plan-fable.md 2.1 (row `stack`),
3.3 (I12), decision 12 (D89 kept, extended) and slice row WM5; docs/decisions.md D73, D83, D89; reports/wm2.md.
1. Red first, one commit: I12 cells (no bar or panel box over a fullscreen window; the held window on top; a modal
   above its parent) and a core unit test of the layer order background < normal < held < overlay < lock.
2. Core: scene layers per D89 with `held`; inside a layer the look orders. The lock layer stays above every plugin
   (escape hatch); name each changed line in the report.
3. `chora_kit::wm::stack`: two rules only (a modal above its parent, the held window on top); typed, unit-tested.
4. Ports: mc drops its root reorder (D83) for the layer; tiling, xp, construct use the stack rules where they
   order today, else a compile fix. Visible behaviour unchanged except the fixed bugs (B5 cover, F29 bar over
   fullscreen if it is this cause); known-red only shrinks.
5. docs/decisions.md D89 updated (held layer, D83 retired by this branch); tools/copies.toml rows lowered.
Done when: cells red then green; `cargo test -p chora` and each look crate green; merge gate once on the head
(poll it in the same turn); `kit::wm` coverage with `cargo llvm-cov -p chora-kit --features chora-kit/testing`
(missing: blocker, do not install). Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/wm5.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
