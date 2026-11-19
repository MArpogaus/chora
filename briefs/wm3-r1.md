# wm3 r1 (F3): kit::wm::open, the open shell surfaces, in every look

Model: sonnet, effort high (a kit module on the pattern WM1 and WM2 set; the design is arch-plan row `open`)
Lane: cloud, branch claude/wm3 from 1d1aa5fd (train 58). One round, stop at 250 k. Cost about 2.5 %.
Run `git fetch --unshallow` first. When the session branch moves (trains 59, 60),
`git merge origin/claude/autonomous-dev-session-3w-i33pg7`. Read reports/arch-plan-fable.md 2.1 (row `open`),
3.3 (I4) and slice row WM3; reports/wm1.md, reports/wm2.md on claude/session-log.
1. Red first, one commit: the I4 cell "Start menu open through a drag" (sd-h2): an xp act opens the Start menu,
   drags a window; `open` after the act must equal `open` before plus what the act declared (menu shut).
   Plus one unit cell in chora-kit: a give or a drag shuts every open surface not declared to stay.
2. `chora_kit::wm::open`: the open surfaces (menu, popup, dialog, sheet, launcher), who opened each, which is
   modal, one Esc order (newest first), shut at a drag or a give. Typed, small, unit-tested; feeds the
   `open` list of `kit::wm::state` (no second json path).
3. Ports: xp (menus, dialogs) and tiling (launcher, popups) use it; mc and construct only where they hold
   an open surface today, else a compile fix. Visible behaviour unchanged except the fixed bug.
4. tools/copies.toml rows for open/close code lowered by what the ports delete; known-red only shrinks.
Done when: cells red then green; `cargo test -p chora-kit --features testing`, `-p chora-xp`, `-p chora-tiling`,
`-p chora-mc`, `-p chora-construct` green; merge gate once on the head (poll it in the same turn); `kit::wm`
coverage with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (missing: blocker, do not install).
Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/wm3.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
