# construct r2 (rc 1): the pulled window stays visible, a helper button

Model: sonnet, effort high (scene state across step-back; cause unknown)
Lane: cloud, branch claude/construct; merge the session branch 4105f8d1 first. One round. Stop at 250 k.
Read reports/review-tour-v5.md, reports/media-r3.md (N2, still 04-12), reports/construct-r1.md, design/looks/construct.md, dev-rules.md.

1. N2 MAJOR: after a window is pulled off a face into the space, stepping back (Alt+Escape) shows
   that pane, and it can be dragged back over a face. Red cell first (tests/gate-construct-pull.sh:
   after step-back the pane node exists and is drawn; window count unchanged). If the cell passes on
   the head, the fault is the tour script: say so and fix the script.
2. rv5-3 MAJOR: the HUD has a helper button (as Programs, Keys, Lock); click opens the chat. Red cell first.
No MINOR work (rules.md 6). Done when: the cells pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/construct-r2.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/construct and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
