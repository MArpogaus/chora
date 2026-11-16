# construct r1 (rc 1): no lost window, keys overlay, a way out, no stray keys

Model: sonnet, effort high (one BLOCK in drag and scene code, one core input follow-up)
Lane: cloud, branch claude/construct (from claude/multilook bb8df12a, MERGE-OK); merge the session branch first. One round.
Stop at 250 k context. Read findings/construct.md (co-18..20), findings/shared.md (sh-23, sh-24),
reports/journey-1.md (C1, C2), reports/review-multilook-r1.md, design/looks/construct.md, dev-rules.md.

1. co-18 BLOCK: a drag on the stepped-back cube must turn it, and no window may ever leave the
   scene (lost window). Red cell first (tests/gate-construct-*.sh: drag on the stepped-back cube,
   window count before and after, face of each window); if the cell passes on the head, the fault
   is the tour script: say so and fix the script cell instead.
2. co-19 MAJOR: F1 in the stepped-back view shows the bindings overlay. Red cell first.
3. co-20 MAJOR: with the cube open, Escape (and the overlay) closes the cube; documented in the
   overlay and docs. Red cell first.
4. sh-24 MAJOR (core input.rs:433, 466-471): a grab held back by the pointer gate is swallowed,
   never passed to the client as a raw key; injected keys skip the pointer gate; key_place only
   when a grab matched. Red unit test first; keep the escape hatch untouched.
No MINOR work (rules.md 6). Done when: the cells pass, tests/merge-gate.sh once on the head.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/construct-r1.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/construct and that report; never end a turn to wait
for a long background job.
