# wm1b r1 (F2, F3): port mc to kit::wm::book

Model: sonnet, effort high (a port on fixed types; WM1 settled them)
Lane: cloud, branch claude/wm1b from the session branch (27bd60c7, train 54). One round, stop at 250 k.
Cost about 1.5 %. Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md 2.2 and slice rows WM1, WM1b-d, and reports/wm1.md (refactor proposals 2, 3)
on claude/session-log.
1. mc's window lifecycle (create, attach, detach, destroy, output removed, give in and out) runs through
   `kit::wm::book`; mc's own copy and its state adapter for it go in the same merge. mc's detach policy (put
   the window back) is a look decision on a book proposal.
2. Move the settle-loop mini-look of wm_book.rs to `kit::testing::Script` (wm1 proposal 2) and run the same
   script for mc: create, refused attach, detach, destroy, output removed, give in, refused give out.
3. Port cells red first: restore from meta written by the 29e2cd77 binary; mc taskbar order; acts unchanged.
4. tools/copies.toml: `Event::SurfaceCreated outside kit` lowered by mc's count (2), mc row at max 0.
5. Visible behaviour unchanged (door acts, NET2 acts; known-red only shrinks). No new feature; no change to
   tiling, construct, xp beyond compile fixes.
Done when: cells red then green; merge gate once on the head (poll it in the same turn); `kit::wm`
coverage with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (setup.sh installs it; if it is
missing, name it a blocker, do not install). Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/wm1b.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
