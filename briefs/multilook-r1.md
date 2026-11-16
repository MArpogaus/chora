# multilook r1 (rc 1): two MAJOR findings where looks share the screen

Model: sonnet, effort medium (two located bugs with a recorded reproduction)
Lane: cloud, branch claude/multilook from the session branch (66f13d68). One round.
Stop at 250 k context. Read dev-rules.md, conventions.md, reports/media-r1.md (findings table),
media/2026-11-16/tour-v3/timeline.md rows of chapters 5 and 6, and demo/tour/ch5.py, ch6.py on
claude/media (00225164) for the exact steps.

1. sh-23 MAJOR: xp and tiling in two panes. The tiling pane stays black; after a click into it,
   Super+Return opens the xp Start menu in the other pane (keyboard focus stays with xp).
   Expected: the pane shows tiling; a click into a pane moves the keyboard to that pane's look.
2. co-16 MAJOR: construct, stepped-back view. After a window drag the cube leaves the screen to
   the right and the floor stays empty ("XP 2 face 2 of 4") until the step-in. Expected: the cube
   never leaves the view; a drag that ends off the cube snaps back or lands on the nearest face.
3. ti-20 verify: a click on Apps in the tiling bar must open the launcher within 1.5 s on the
   headless setup (CI e2e has the cell; run it 10 times here). Real failure: MAJOR, fix it;
   else close it as not real with the 10 passes.

For each fix: a red gate cell or unit test first (its own commit), then the fix; no tolerance or
retry. Done when: the new cells pass 10 times, tests/merge-gate.sh passes once. Report to the PM
session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 8 lines) and as reports/multilook-r1.md on
claude/session-log: each claim with its gate cell and a capture. Rules as briefs/flakes-r2.md,
push only claude/multilook and that report; never end a turn to wait for a long background job.
