# mc r6 (rc 1): a window handed to mc is visible at once

Model: sonnet, effort high (handover render path; cause unknown)
Lane: cloud, branch claude/mc; merge the session branch 4105f8d1 first. One round. Stop at 250 k.
Read reports/review-tour-v5.md (rv5-2), media/2026-11-17/tour-v5/timeline.md (handover 3:30-3:40), demo/tour/ v5 handover script on claude/media 0f941e2, design/looks/mc.md, dev-rules.md.

1. rv5-2 MAJOR: when a window is given to mc (drag or key give), mc shows it within one frame
   budget (as a panel on the page), not after about 6 s or never. Red cell first (gate cell: give a
   window to mc, the panel is drawn within 1 s; count unchanged). If the cell passes on the head, find
   why the tour differs (timing, output, script) and say so with evidence.
No MINOR work (rules.md 6). Done when: the cells pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/mc-r6.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/mc and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
