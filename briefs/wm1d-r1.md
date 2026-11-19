# wm1d r1 (F2, F3): port xp to kit::wm::book

Model: sonnet, effort high (a port on fixed types; WM1 settled them, wm1b did mc, wm1c construct)
Lane: cloud, branch claude/wm1d from 1d1aa5fd (train 58 candidate: wm1b, kit2b, wm1c). First
`git fetch --unshallow`; if train 58 lands, `git merge origin/claude/autonomous-dev-session-3w-i33pg7`.
One round, stop at 250 k. Cost about 1.5 %.
Read reports/arch-plan-fable.md 2.2 and rows WM1b-d, reports/wm1b.md and reports/wm1c.md (Review r1) on
claude/session-log.
1. xp's window lifecycle (create, attach, detach, destroy, output removed, give in and out) runs through
   `kit::wm::book`; xp's own copy and its state adapter go in the same merge.
2. Run the `kit::testing::Script` of wm1b for xp (same seven steps).
3. Port cells red first: restore from meta written by the 29e2cd77 binary; taskbar order; acts unchanged.
4. tools/copies.toml: `Event::SurfaceCreated outside kit` to 0 (xp dispatch.rs:20 was the last), xp row max 0.
5. Also, each with its red cell: wm1c rw-1 (plugins/chora-construct/src/adopt.rs:206,222: a window adopted by
   token or launch while `free` is false is never claimed in the book; claim it through the book, or skip a
   window reserved for another manager) and rw-2 NIT (docs/api.md:1513 `node` is the surface id where a look
   has no groups).
6. Visible behaviour unchanged (xp acts, xp-* gate steps; known-red only shrinks). No new feature; no change
   to mc, tiling beyond compile fixes; no xp desktops (WM4).
Done when: cells red then green; merge gate once on the head (poll it in the same turn); `kit::wm`
coverage with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (if missing, a blocker; do not
install). Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/wm1d.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
