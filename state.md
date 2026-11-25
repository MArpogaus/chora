# State

## Now (2026-11-25 14:02 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 14:02 PM 3.5 % (11-25); week about 38 % vs 63 % time; need (100-38-5)/62 h = 0.9 %/h; 6 build lanes
lanes-short: tiling lane busy (r4 waits); P2-R10-co, t141 after t140
session branch: e39448ad (t139 12:55: tiling-doors r3, tiling-minor r1; ti-55, ti-56 fixed, recheck P2-R10); 87 greens. TRAIN140 62a04be0: xp-minor r3, mc-minor r2, co-minor r2 (gate running)
burn-down: F1-F4, F6, F8 met; F5 met (series r9), rerun on final head; F7 open (v9 take r1: 82 open, 3 regr; take r2 now)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | construct co-75, -76, -79 fixed t136 (recheck next persona round); xp, mc: none; tiling ti-55 open (r10 red) |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r10 not met (T7 ti-55 red 5th round; ti-56 chip ok) |
| 3 | 3 greens, no flake | met: t130-t132 green; FLAKE-SERIES r9 5/5 c3c37c2e |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-33, sh-35, sh-36, sl-9 closed; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train140 | PM, 62a04be0 on e39448ad | gate |
| mc-minor-r3.md (rc 1; rmm2-1..6; 0.8 %) | sonnet medium, mc-minor-r3 0f68810c | running s..NMXoJFbpfCZT9oZsw |
| co-polish-r1.md (rc 1; co-44, co-45; 0.8 %) | sonnet medium, co-polish-r1 d86be5d0 | running s..LQizNrD4V4G9kNF4m |
| tour-v9-take-r2.md (F7, rc 11; 1 %) | sonnet medium, tour-v9-take-r2 e39448ad | running s..NYLg6VoKk9VmtWivs |
| tiling-minor-r2.md (rc 1; rtm1-1..3, rtd3-1..3; 0.6 %) | sonnet medium, tiling-minor-r2 e39448ad | running s..NuxhEtMVy3GKQ2Gva |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) t139 green e39448ad. TRAIN140 running: xp-minor r3, mc-minor r2, co-minor r2.
2. (rc 2) construct: co-minor r2 in t140; co-77 by design D106; co-polish r1 (co-44, co-45) running; persona recheck co-75/76/79 in P2-R10.
3. (rc 2) P2-R10-tiling: not met (ti-55 T7 5th red: persona ties 'xp look' to tag 2). After tiling-minor r2: TILING-DOORS r4 (opus high, design: a look door on the bar that is not a tag; 1 %). P2-R10-co after t140 (co-75/76/79; 0.8 %); tiling-minor r2 running. xp-minor r4 MERGE-OK 0a0850e4 -> t141.
4. (F7, rc 11) TOUR-V9 take r2 on e39448ad running; then 4 fresh-eyes judges, one per look (opus high, 2 %); no tooling round before take r2 (critic 11-25).
5. (F5, rc 3, 4, 5) after t140: FLAKE-SERIES r10 on the final head (haiku); CI-PR dry run (sonnet); rtd-5, rtd-7 MINOR backlog; harness: act.py waits 9.7 s on an unbound key > 8 s chip.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
