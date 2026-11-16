# harness rc r2 (rc 3, 4): run the F16 red test in CI; signal by name

Model: sonnet, effort low (two named edits, no search)
Lane: cloud, branch claude/harness-rc (74a6b037); merge the session branch first. One round.
Stop at 250 k context. Read reports/review-harness-rc-r1.md (FIX: 1 MAJOR, 1 MINOR).

1. MAJOR: add tests/gate-tiling-bar-early-signal.sh to the CI e2e job (.github/workflows/ci.yaml,
   beside the Apps cells at :178-181), so tests/merge-gate.sh runs it too. Show it runs in the
   merge-gate e2e step log.
2. MINOR: plugins' waybar.sh:83 traps signals by name (RTMIN+1, USR1, USR2), not 35; set the
   ignore before the pid file is written, so no signal can arrive in between.
Done when: the early-signal cell passes, tests/merge-gate.sh once. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines) and as reports/harness-rc-r2.md on
claude/session-log. Rules as briefs/flakes-r2.md, push only claude/harness-rc and that report;
never end a turn to wait for a long background job.
