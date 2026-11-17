# tiling-look r4 (rc 1): Super+Print gives a visible sign and has a mouse path

Model: sonnet, effort medium (one bounded fault, review-found twice)
Lane: cloud, branch claude/tiling-look; merge the session branch aa8aa8c0 (or newer) first. One round. Stop at 250 k.
Read reports/review-tour-v6.md (j1-5), reports/tiling-look-r3.md, design/looks/tiling.md, dev-rules.md.

1. j1-5 MAJOR: in tour v6 (1:40-1:42) Super+Print shows nothing. Find why the r3 notice does not show in the
   real session (not only in the gate cell): name the cause in the report. Then: a notice with the saved path,
   or why not, visible at least 3 s; red cell first that runs the real key path and asserts the notice pixels.
2. Same item: a Screenshot entry a mouse user can reach (the power menu or a bar pill, KISS: one place), with a cell.
No MINOR work (rules.md 6; N-5 bar on the handover screen goes to the next round).
Done when: the cells pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Refactor proposals: list up to 3 in the report (file, why, size), do not do them.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/tiling-look-r4.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/tiling-look and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
