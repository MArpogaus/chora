# core watchdog r1 (rc 3, rc 4): sh-22 / F23, likely cause of F25

Model: sonnet, effort high (core safety code; cause and gate are named in the review)
Lane: cloud, new branch claude/core-watchdog from the session branch
claude/autonomous-dev-session-3w-i33pg7 (2d9f306f or newer; merge it again when train 17
lands). One round. Stop at 250 k context and hand over. Read dev-rules.md and conventions.md
(with the skills it names) first, then reports/review-flakes-r5.md (M1), reports/ci-f25-r1.md,
findings/shared.md sh-22. Setup as briefs/flakes-r2.md.

Items, in order:
1. Diagnostic (test only, small): tests/agent.sh step 6, on a mismatch print to the log the
   host pixel before and after the capture, and the watchdog lines of chora.log (effects
   dropped or not). No wait, no tolerance change. One commit.
2. sh-22: under the software renderer the frame watchdog (crates/chora/src/safety.rs 14-22,
   186-198, 336) counts the renderer threads' CPU wait as scene cost and drops all effects of
   all clients. Red first: a unit test on the watchdog frame check that feeds a frame whose
   cost is host wait (not scene work) and expects no drop. Then the fix (measure scene cost
   only, or ignore frames the host starved; KISS, keep the escape hatch for a truly costly
   scene: a second unit test proves a costly scene still drops effects).
3. Then remove the 500 ms relaxation of tests/gate-tiling-audit.sh:43 (default budget) and run
   the audit at 2560x1600@2 three times under 8 busy loops on 4 cores (say the load).
Done when: both unit tests pass, the audit passes 3/3 with the default budget, and
tests/merge-gate.sh passes once. Say whether item 2 explains F25 (glow at 5,5 doubled on the
CI host frame) or not; do not claim it without evidence.
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and as
reports/core-watchdog-r1.md on claude/session-log, with the earliest gate per item.
Rules as briefs/flakes-r2.md, with push only claude/core-watchdog and that report; never end
a turn to wait for a long background job.
