# secfix c r1 (rc 1, 6): sanitize and guard the voice confirm question (sh-26, MAJOR)

Model: opus, effort medium because it is a security fix across voice, mc and construct (rule 9)
Lane: cloud, new branch claude/secfix-c from claude/autonomous-dev-session-3w-i33pg7. One round.
Stop at 250 k context. Review: reports/review-security-confirm.md (on claude/session-log).

1. Red tests first (one commit), as named in the review section 5.
2. services/voice: sanitize prompt, detail and name in mc.rs confirm() with chora_kit::chat::sanitize;
   refuse a tool call whose detail changes under sanitize (chat.rs and agents.rs, next to too_long),
   with an honest reason to the model; mc.rs:351 fallback refuses instead of truncating.
3. services/voice core.rs confirm_button: ignore a press within 400 ms of shown_at (same rule for
   a replaced question); key and button share one guard check.
4. Defence in depth: mc agent.rs:663 and construct hud.rs:1059 sanitize what they show.
   Do not migrate them to kit Question/Card now (sprint 1, DB6); list it under Refactor proposals.
5. docs: docs/known-issues.md or security notes only if behaviour a user sees changes.
Done when: each red test fails before and passes after, tests/merge-gate.sh once on the head.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/secfix-c-r1.md on claude/session-log, ending with Refactor proposals. Rules as
briefs/flakes-r2.md, push only claude/secfix-c and that report; never end a turn to wait for a
long background job.
