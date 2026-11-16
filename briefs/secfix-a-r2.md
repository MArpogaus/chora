# secfix a r2 (rc 6): the two MEDIUM of the review of secfix a r1

Model: sonnet, effort medium (two bounded fixes in one file pair, with tests)
Lane: cloud, branch claude/secfix-a (head bb237e8); merge the session branch first. One round.
Stop at 250 k context. Review findings: reports/review-secfix-a-r1.md (on claude/session-log).

1. FIFO hang: crates/chora-kit/src/icons.rs:92 `is_small_png` opens the sender path with a
   blocking open before any type check; a FIFO hangs the notify thread. Open with
   O_NONBLOCK (OpenOptionsExt::custom_flags), then fstat and require is_file(). Red test: a
   FIFO path returns the fallback within 1 s.
2. TOCTOU: for_sender validates the path, then card.rs:249 `picture.foreign(path, rect)` re-opens
   it; the sender can swap the file in between. Read the bytes once (bounded 512 KiB), validate
   them, and hand the renderer the validated bytes or a private copy the sender cannot write. If
   the renderer takes only a path, use a private copy under $XDG_RUNTIME_DIR (0600), cleaned up.
   Red test: the file is swapped for an SVG after the check; the render does not read the SVG.
3. Add the missing boundary tests: exactly 512x512 and 512 KiB pass, 513 and 512 KiB+1 fail, an
   empty file falls back. Update docs/known-issues.md sh-16 (the words are now consistent).
Done when: each red test fails before and passes after, tests/merge-gate.sh once on the head.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/secfix-a-r2.md on claude/session-log. Rules as briefs/flakes-r2.md, push only
claude/secfix-a and that report; never end a turn to wait for a long background job.
