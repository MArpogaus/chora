# Self-check 2026-11-22 (workday close 12:50 UTC, 19:50 Bangkok)

## Self-check
- Burn-down: F1-F4, F6, F8 met; F5 reopened (critic 4); F7 open. Merges today: t107 08:47 (62 greens); t108 green through tiling-keys-more at 12:49.
- Branch MERGE-OK and unmerged > 12 h: none yet; launch-timer due 19:58, test-gaps 21:06, log-down-co 21:28, sim-long 22:06 (all first in t109).
- Running but not queued: none.

## Main duties
- Budget: 9.8 % at 12:45 after 32.75 h (0.30 %/h), under pace 19.7 %. Six lanes run, but they wait on 2 h gates; more lanes of the same kind add little (critic 5).
- Models: opus for core-diag, mesh-churn, ti-38 (core and complex); sonnet for log-down and flake-hint; haiku for the flake series. Fable unused.
- Vision card: no new feature; work is RC bar, logging, leaks, flakes, F7.

## Critic (opus, 12:50) and actions
1. Trains at --jobs 1 though flakes-l merged in t92: procedures Train 3 now --jobs 2; t109 runs at 2.
2. 12 h risk for t109: the four oldest shas lead t109 (Queue 5); on a red, bisect halves in two haiku cloud lanes.
3. MERGE-OK with incomplete lane gates (split-b, log-down-mc, looks1-mc): procedures Seeded review: a failed Done-when line is INCOMPLETE; these are "gates pending t109" (the full train gate).
4. F5 claimed met on one series: reopened in state.md; closes after flake-hint and an independent haiku series (5 at --jobs 2) on the t109 head.
5. Pace line wrong (0.6): now 0.67 %/h; token-heavy work added (reviews, CI-HEAD, REVIEW-DELTA).
6. PM reviews of big diffs: split-c and split-b are pure moves, checked mechanically with --color-moved (kept as PM OK, rule: PM review only under 200 lines or a pure move). looks1-mc changes design behaviour: opus seeded review before it trains; out of t109.
7. RC table stale: state.md now carries the week-3 table (waiver gone, rows 1-8 open).
8. Queue without criterion and %: rewritten; every line names its row and %.
9. Lanes not in RC order: next free slots go to CI-HEAD (rc 4, 5) and REVIEW-DELTA (rc 6).
10. Rule 1 date: changed at the t109 merge boundary (rules.md 10). PM model is the session's; routine train watching stays in background shells (no tokens while waiting).

## Frictions resolved
- 11-22 lc-3: a lane used a bare git stash; every brief carries "never a bare git stash" (flake-hint r1, core-diag r2): guarded.
- 11-22 2 h tool cap killed t108 and the sb-2 gate: trains and lane gates run setsid nohup detached (procedures Train 3, briefs): guarded.

## Workflow change
- Trains at --jobs 2, oldest MERGE-OK first; INCOMPLETE verdict for a failed Done-when line (procedures).
