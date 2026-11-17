# construct r4 (rc 1): the cube seams close

Model: sonnet, effort high (projective geometry fault; the reason for high: one hard cause, no breadth)
Lane: cloud, branch claude/construct; merge the session branch (head after train 40) first. One round. Stop at 250 k.
Read findings/construct.md co-21, media/2026-11-17/tour-v7/stills/04-12-pane-stepped-back.png and 04-13-back-on-face.png,
reports/review-construct-r3.md, design/looks/construct.md, dev-rules.md.

1. co-21 (MAJOR, owner): stepped back, the faces of the cube meet: adjacent faces share their edge
   (corner pixels within 2 px at 1x and 2x), the near face is not larger than the face in front unless
   the turn puts it nearer, no face edge vanishes, the floor reflection lines up under the faces.
   Red cell first: a gate cell (gate-construct-cube or a new gate-construct-seams.sh) that steps back,
   turns 0, 45 and 90 degrees, and asserts the shared edges from the face transforms and from pixels.
   Then the fix; name the root cause (sign, pivot, focal distance, order of transforms) in the report.
2. Do not touch the xp cube (plugins/chora-xp/src/cube.rs); DB12 merges the two cubes in sprint 1.
Done when: the cell is red on the base and green on the head, the merge gate passes locally, captures at
0, 45, 90 degrees in reports/construct-r4/. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with
send_message (at most 6 lines) and as reports/construct-r4.md on claude/session-log, ending with Refactor
proposals. Push only claude/construct and the log files named here.
