# tiling-look r3 (rc 1): bar gap, screenshot notice, lock and power by mouse

Model: sonnet, effort medium (three bounded bar faults)
Lane: cloud, branch claude/tiling-look; merge the session branch 4105f8d1 first. One round. Stop at 250 k.
Read reports/review-tour-v5.md, reports/media-r3.md, reports/journey-1.md, design/looks/tiling.md, dev-rules.md.

1. j1-4 MAJOR: the bar shows "No helperVolume 40%"; every bar module has a visible gap. Red cell first
   (tests/gate-tiling-*.sh: text boxes of adjacent modules do not touch).
2. j1-5 MAJOR: Super+Print gives a notice (saved path, or why not), as the other looks do. Red cell first.
3. rv5-1 MAJOR: a mouse path to lock and power: the power icon the key list names, in the bar, opening
   lock, log out, power off with confirm. Red cell first (click the icon, the menu shows).
4. Fold tiling rv-5: add tests/gate-tiling-power.sh to tests/merge-gate.sh if it is not there.
No MINOR work (rules.md 6). Done when: the cells pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/tiling-look-r3.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/tiling-look and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
