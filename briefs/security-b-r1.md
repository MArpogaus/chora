# security b r1 (rc 6): the MEDIUM security findings of review-plugins-r1, services/voice

Lane: opus, cloud, new branch claude/security-b from the session branch
claude/autonomous-dev-session-3w-i33pg7 (newest). One round. Stop at 250 k context and hand over.
Read dev-rules.md and conventions.md (with the skills it names) first, then the lines of these
ids in reports/review-plugins-r1.md (session-log): VO-A-01, VO-A-02, VO-A-07, VO-B-01, VO-B-02, VO-B-06 (services/voice).
design/rc.md 6: every security MEDIUM fixed. For each id:
1. Check it still holds on the newest session branch (code moved since 498d9de). If it no longer
   holds, say why with file:line; no change.
2. Else a red test first (own commit; unit test where possible, compile-time check where the class
   can be closed by a type or a clippy rule), then the smallest fix. KISS: no new framework; a
   privilege flag, a cap, a mode, a check. No behaviour change beyond the finding; a user-visible
   change keeps an honest state (rules of the look).
3. Name the earliest gate that would have caught it; if a cheap compile-time gate closes the whole
   class (e.g. one helper that every method table must use), add it in this lane.
Before handback: merge origin/claude/autonomous-dev-session-3w-i33pg7 again (a harness train lands
meanwhile; its hooks apply to you at once). Output: the full tests/merge-gate.sh once (serial,
background job with a 2 h limit), green; tests/ci-local.sh once; reports/security-b-r1.md on
claude/session-log (per id: holds or not, red commit, fix commit, gate). Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines).
Rules: push only claude/security-b and claude/session-log (report); author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines; never --no-verify; never end a
turn to wait for a long background job; CARGO_TARGET_DIR inside your worktree only; install nothing
(a missing tool is a blocker).
