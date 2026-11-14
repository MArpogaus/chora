# ci e2e r2 (rc 4): make the two nested checks strict again

Lane: same worker, branch claude/ci-e2e (at 4300c2d4). One round; setup and rules as briefs/ci-e2e-r1.md.
Items from reports/review-ci-e2e-r1.md:
1. C1 MAJOR: step 21 must not pass on the frame left by the previous shear step. Either wait for a
   frame rendered after the depth step (a frame counter or render.stats sequence) before sampling,
   or sample a point well inside the green plane where filtering gives pure green, and keep a check
   that the stale red/green blend fails. Show the old frame failing the new check.
2. R1: assert three surfaces and non-empty rects for each before the overlap check.
3. C2: comment the helper with the why and keep it next to its step.
Done when: tests/nested.sh passes on sway three times; tests/merge-gate.sh passes. Report to the
PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 8 lines).
