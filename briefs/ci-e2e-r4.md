# ci e2e r4 (rc 3, rc 4): no skip counts as a pass

Lane: same worker, branch claude/ci-e2e3. One round; setup and rules as briefs/ci-e2e-r1.md.
Items from reports/review-ci-e2e-r3.md (friction 47):
1. F3: tests/e2e-sequence.sh counts SKIP lines and missing tools; the step fails when any script
   skipped, unless the skip is listed in one explicit allowlist file with a reason (e.g. tools the
   container cannot have); the summary names every skip.
2. F1: gate-0.7-6 fails instead of skipping when $CI is set.
3. F2: e2e-sequence.sh runs multi-line run blocks too (or the font step moves into a script that the
   job calls), so gate-0.8-9 runs locally; honour env of the step where it matters.
4. F4, F5, F6: docstrings, animation.done instead of fixed sleeps where the old code waited, a check that
   the confined MCP camera call is refused, the variable scoped, the temp dir removed.
Done when: tests/merge-gate.sh passes with the e2e step reporting zero unlisted skips; the e2e
sequence passes 3 times. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines).
