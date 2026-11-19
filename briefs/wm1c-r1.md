# wm1c r1 (F2, F3): port construct to kit::wm::book

Model: sonnet, effort high (a port on fixed types; WM1 settled them, wm1b did mc)
Lane: cloud, branch claude/wm1c from 0ffc53a8 (wm1b head, MERGE-OK). First `git fetch --unshallow`, then
`git merge origin/claude/autonomous-dev-session-3w-i33pg7` (mv3 file moves). One round, stop at 250 k. Cost about 1.5 %.
Read reports/arch-plan-fable.md 2.2 and rows WM1b-d, reports/wm1b.md on claude/session-log.
1. construct's window lifecycle (create, attach, detach, destroy, output removed, give in and out) runs through
   `kit::wm::book`; construct's own copy and its state adapter go in the same merge.
2. Run the `kit::testing::Script` of wm1b for construct (same seven steps).
3. Port cells red first: restore from meta written by the 29e2cd77 binary; face order; acts unchanged.
4. tools/copies.toml: `Event::SurfaceCreated outside kit` lowered by construct's count, construct row at max 0.
5. Visible behaviour unchanged (cube, pull acts; known-red only shrinks). No new feature; no change to
   mc, tiling, xp beyond compile fixes; no pane flag change (WM6).
Done when: cells red then green; merge gate once on the head (poll it in the same turn); `kit::wm`
coverage with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (if missing, a blocker; do not
install). Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/wm1c.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
