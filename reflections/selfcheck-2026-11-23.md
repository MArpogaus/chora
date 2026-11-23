# Self-check 2026-11-23 (workday close 12:50 UTC, 19:50 Bangkok)

## Self-check
- Burn-down: F1-F4, F6, F8 met (audit r4 of 11-20, to recheck: audit-f r5 after t115); F5 open (sh-32, sh-33, sh-34); F7 open (xp-readable, mc-tidy). Merges: t114 green 11:10 (68 greens); t115 green through tiling-theme at 12:45.
- MERGE-OK unmerged > 12 h: none (co-focus, tiling-flake, record-v9 in t115).
- Running but not queued: none (6 lanes: flake-series, sim-long, tiling-disc, xp-readable, mc-tidy, leak).

## Main duties
- Budget: tokens.py 12:38 PM 3.9 % today; week about 18 % vs 34 % pace (0.6 %/h; the 0.67 figure was wrong). Under-spent; the 6-lane cap binds, so spend goes to opus reviews outside the cap (co-48..51 verify, audit-f r5) and DOCS-RC r2 at the next slot.
- Models: opus for leak (complex), co-focus r3; sonnet for xp-readable, mc-tidy, tiling-disc, record-v9; haiku for flake series and sim-long. Fable unused.
- Vision card: no new feature; work is RC bar, leaks, flakes, F7 looks.

## Critic (opus, 12:47) and actions
1. record-v9 r4 test and fix in one commit, review NIT: kept in t115 (tour harness only); friction entry; Seeded review now rates it MAJOR.
2. tiling-flake r2 red commits not real: same rule; H-RF lane queued (Queue 2).
3. F met on old audit: audit-f r5 (opus) on the t115 head (Queue 1).
4. F5 Queue premature; run-2 keys red not in ledger: sh-34 added; F5 needs 5/5 and sh-32..34 closed.
5. Rule 6 on mc/xp: verified no open BLOCK/MAJOR in findings/xp.md, mc.md. PM decision: a shared core MAJOR (sh-33 leak) blocks only the look whose code holds it; mc-57 stays MINOR (a test fake, not a user path).
6. Budget under pace: opus reviews outside the build cap; DOCS-RC r2 not parked to Thu.
7. RC row 8 stale: fixed (DOCS-RC r1 merged 54c30d16); design/rc.md table rewrite with the next RC change.
8. Queue without %: rewritten, each line % and criterion.
9. Frictions 11-22 21:30, 11-23 09:30, 09:42: resolved (12:40).
10. Budget line refreshed 12:38; sim-long id in sh-33.

## Frictions resolved
- 11-22 21:30, 11-23 09:30, 11-23 09:42, 11-23 12:50 (see friction.md).

## Workflow change
- Seeded review: a fix with no red test commit before it is MAJOR (procedures.md), so the verdict computes FIX.

## Addendum 12:53 UTC (trigger fired after the close was done)
- The close ran ahead of the 12:50 trigger; no second critic (same 24 h, budget). t115 green 12:46 (bc153929, 69 greens); Queue 1 started 12:49: p2-r7-co (sonnet medium), audit-f r5 (opus high). Frictions: none open.
