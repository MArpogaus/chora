# ci e2e r3 (rc 4): every script of the e2e job green

Lane: same worker as ci e2e r1/r2, new branch claude/ci-e2e3 from the session branch (d349a6da).
One round; setup and rules as briefs/ci-e2e-r1.md. Never end a turn to wait for a long job.
On PR #106 (run 43656958796, job 132637503881) nested.sh now passes; the job then fails in
tests/agent.sh step 2 "tiling.get_layout and tiling.set_layout switch the layout": "FAIL: timeout:
xs_are 0 640" (backlog V12 says it fails on the session branch too; likely the same geometry bug
as nested.sh step 2: x read from transform.matrix of surfaces under tc-* parents instead of .rects).
Items:
1. Run, in order, every script the e2e job of .github/workflows/ci.yaml runs after nested.sh, on a
   headless sway host locally; fix each failure at its root cause (test bug or product bug, red first
   for a product bug), until the whole e2e job sequence passes locally. Include backlog V12
   (voice-acp.sh, voice-events.sh) if the job runs them.
2. CI4: add one merge-gate step (or a gate profile) that runs the same e2e sequence as ci.yaml, so a
   CI-only failure shows locally first.
Done when: the e2e sequence passes locally three times; tests/merge-gate.sh passes. Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 12 lines): per script the cause and the commit.
