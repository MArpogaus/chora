# ci e2e r5 (rc 4): close the review of r4

Lane: same worker (sonnet), branch claude/ci-e2e3. One round, fixed. Read reports/review-ci-e2e-r4.md.
Start: merge origin/claude/autonomous-dev-session-3w-i33pg7 (train 11 or newer) into the branch.
Items (test/CI-only: commit type `test:` or `ci:`, a red selftest case first where one fits):
1. MAJOR 1: a skip never passes in CI. `skip` in tests/gate-lib.sh exits non-zero when CI is set
   (print the reason). Red: a case in gate-lib-selftest.sh: skip under CI=1 fails, without CI exits 0.
   Check every e2e step in ci.yaml then either runs or fails; a step whose tool CI lacks gets the tool
   in the CI image or leaves ci.yaml with a line in docs/known-issues (none silently skipped).
2. MAJOR 2 (PM decision 11-15 03:02): the confined rule (docs/api.md:1075) stays; no agent writes
   under screen:*. Restate gate 0.8-10 as what it checks: the roadmap line (docs/roadmap.md:236),
   the script header and the ci.yaml step name say a user client puts a text node under
   screen:NAME; the gate also asserts a confined create under screen:* is refused.
3. MINOR: an empty command list fails the sequence; an allowlist entry needs a reason and matches
   the script name literally; in CI a missing font fails tests/e2e-fonts.sh (fix its comment).
4. COSMETIC if cheap: merge-gate.sh:24 comment; one free_apply shared by gate_mcp.py and mcp-client.py.
Done when: tests/merge-gate.sh passes as one run (a known flake F16-F19: rerun once, name it).
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and reports/ci-e2e-r5.md on
claude/session-log. Rules as briefs/ci-e2e-r4.md.
