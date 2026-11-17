# arch r1 (foundation F1): the refactor and architecture plan

Model: Fable (claude-fable-5-2), effort high, because the owner ordered one Fable architecture lane (11-17 10:47); time-boxed 1.5 h; cost about 5 % of the week
Lane: cloud, read-only on the session branch (head after the rc tag); writes only the log files named here. Stop at 250 k.
Read: design/foundation.md, vision-card.md, reports/priorities-sprint1.md, reports/rework-audit.md,
reports/proposal-glitch-gates.md, reports/plan-after-foundation.md (re-plan 14:24), reports/architecture-rc.md (the opus plan, your starting point, not a draft to redo), reports/debt-rc.md,
docs/decisions.md (D1-D95; D87-D95 are proposals you may revise), dev-rules.md.

Goal (owner 11-17): a target architecture that makes future work easier, not cleaner code for a moment.
1. Answer the five questions of priorities-sprint1.md, first: should window-management policy (focus,
   stacking, workspaces/desktops, the cube as a service of construct, DB12) become one shared core lib
   with per-look policy on top? Also settle Fluent vs kit word tables (backlog:167 vs architecture-rc:146).
2. Target structure: crates, kit modules, plugins and their APIs (signatures, not code), the cut gate.
3. Slices: merge-sized steps in order, each with its red test or gate, the look it touches, size, model
   (sonnet default, opus for hard cuts), and which priorities rows and rework clusters it removes.
   Rank by the work unblocked. Mark what the week (foundation F2-F6, budget about 38 %) can reach.
   Mark the scene effect hook S3 needs (F3).
3b. Safety net (owner 11-17 14:24, F7): tour v8 is no baseline. Design the act contracts and the
   per-act invariants of proposal-glitch-gates.md 1-2 (API, where they run, first invariant list from
   the v3-v8 glitch classes) as the first slice, so every refactor slice is checked by them. Name the
   hooks the later simulated user test and the visual checks against design/looks need.
4. Reviews: spawn two independent opus reviewers (effort high, read-only): one on feasibility and
   migration risk (order, gates, what breaks, what the invariants catch), one on product fit against
   priorities-sprint1.md. One round; revise once; keep their reports.
Output on claude/session-log: reports/arch-plan-fable.md (under 400 lines), reports/review-arch-plan-risk.md,
reports/review-arch-plan-fit.md; proposed decision records as a list in the plan (the PM files them).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
