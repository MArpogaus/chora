# Self-check 2026-11-21 (workday close 12:50 UTC, 19:50 Bangkok)

## Self-check
- Burn-down: F1-F4, F6, F8 met; F5, F7 open (carried). No merge today since t98 08:46: t99, t99b, t99c, t99d red (acts, agent, doors, doors).
- Branch reviewed and unmerged > 12 h: none (handover-mo r3 MERGE-OK 11:59; guide-checks, ti-29, mc-minor since 08:5x, blocked by red trains).
- Running but not queued: none.

## Main duties
- Budget: 5.2 % at 12:51 over 8.9 h (0.59 %/h). On pace by the number, under-filled in fact: 1 lane of 6, most spend is the PM loop. Not acceptable (critic 1).
- Models: every lane today ran sonnet as its brief said; haiku unused, opus only for the PM loop and the critic (critic 4).
- Vision card: no new feature; all work F5, F7 and RC bar.

## Critic (opus, 12:50) and actions
1. Under-spending again: start flk-doors-chat (sonnet), flake-series r4 (haiku), week-plan (opus planning) now; 4 lanes.
2. "one build at a time" read as a lane limit: it limits trains on the PM machine only; F5 series moves to a cloud lane (flake-series r4), trains no longer blocked 5 h.
3. No week plan, Queue about 3 %: week-plan r1 (opus, 2 %) writes plans/week-3.md with ranked slices to about 85 %.
4. PM loop on opus: noted; the PM keeps routine in briefs to sonnet and tests to haiku; the PM model is the session's, not mine to switch mid-session; reported to owner.
5. Hand-picked --only gates: guard in tools/start_lane.sh, a Gate line with --only needs "no product code".
6. MERGE-OK without full gate on the final sha: covered by 5; from now merge-OK needs the lane's full-gate run on its merge with the session head.
7. RC table stale: week-plan r1 recomputes it; state.md RC rows get the result.
8. rules.md rule 1 old deadline: draft in week-plan r1, changed at the next merge boundary (rules.md 10).
9. Flake called early: second sighting in t99d (same row), now a lane with 5-run repro per base.
10. Owner reset not verbatim: the reset reached the PM as a budget reset (week line); no owner text exists after 11-20 14:39, so nothing to record; vision card checked in week-plan r1 step 3.

## Frictions resolved
- 11-20 06:17 conflict selftest; 11-21 t95; 11-21 12:06 three red trains (guard now in start_lane.sh).

## Workflow change
- start_lane.sh refuses a product-code brief whose Gate runs --only (critic 5, friction 11-21 12:06).
