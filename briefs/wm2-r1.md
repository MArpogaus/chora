# wm2 r1 (F3, F2): kit::wm::focus and the core grab fix

Model: opus, effort high (reason: a hard cause in the core input path; the pointer-gated grab at
input.rs:461-519 before mv3, now in the input child modules, sends stray keys and splits the focus)
Lane: cloud, branch claude/wm2 from the session branch after train 56 (mv3 split input.rs). One round,
stop at 250 k. Cost about 4 %. Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md 2.2, 3.3 and slice row WM2, reports/wm1.md (Policy at WM2) and
reports/wm1b.md on claude/session-log.
1. Red first, one commit: cells for I3 (two panes hold the focus, sh-23), I11 (a probe client with the
   focus gets a key a look binds, sh-24) and keyboard focus after a stale pointer.
2. Core fix of the pointer-gated grab: the cause named in the report with the line; the escape hatch
   stays above every plugin.
3. `kit::wm::focus` with a `Policy` (focus_after) on the book; pilot tiling and mc (both on book);
   construct and xp only compile fixes.
4. tools/copies.toml rows for focus code lowered by what the ports delete.
5. Visible behaviour unchanged except the fixed bugs; known-red only shrinks.
Done when: the three cells red then green; merge gate once on the head (poll it in the same turn);
`kit::wm` coverage with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (if missing, a
blocker; do not install). Refactor proposals ranked.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/wm2.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
