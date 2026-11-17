# construct r3 (rc 1): drag-back puts the pane on the face, placed survives a restart

Model: sonnet, effort medium (two bounded faults with a named file each)
Lane: cloud, branch claude/construct; merge the session branch aa8aa8c0 first. One round. Stop at 250 k.
Read reports/review-tour-v6.md (N-1), reports/debt-rc.md (B1, B4), reports/construct-r2.md, design/looks/construct.md, dev-rules.md.

1. N-1 (lean MAJOR): a pane pulled into the space and dragged back over a face lands on that face:
   the face owns the window again, it shows in that face's taskbar, no pane stays in front of the cube.
   Red cell first (extend tests/gate-construct-pull.sh: pull, step back, drag back, assert the window
   is on the face and no pane node is drawn). If the cell passes on the head, the fault is the tour
   script: say so, fix the script, and name the frame where the tour went wrong.
2. DB1 / B1: `placed` is saved with the pose (file.rs) or replaced by an "on a face" state, so a
   pulled pane stays visible after a restart. Red cell first (save, restart, assert drawn).
3. B4: triage gate-construct-hud.sh (red "the pane ahead has no label"): fix product or script; wire it
   into tests/merge-gate.sh when green. The other unreferenced construct scripts: list, do not fix.
No MINOR work (rules.md 6). Up to 3 refactor proposals in the report (file, cause, size).
Done when: the cells pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/construct-r3.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/construct and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
