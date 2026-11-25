# State

## Now (2026-11-25 16:02 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 16:02 PM 4.0 % (11-25); week about 40 % vs 64 % time; need (100-40-5)/60 h = 0.9 %/h; 6 build lanes
session branch: 62a04be0 (t140 14:49: xp-minor r3, mc-minor r2, co-minor r2); 88 greens. TRAIN141 e6a17d59: xp-minor r4, tiling-minor r2, co-polish r1 (gate)
burn-down: F1-F4, F6, F8 met; F5 met (series r9), rerun on final head (mc-sofa keylist red in lane boxes); F7 open (take r2: 88 open, 10 regr; judges r2)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | co-75, -76, -79 fixed t136 (P2-R13 recheck); mc rv9ka-1 MAJOR; ti-55 open |
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
| train141 | PM, e6a17d59 on 62a04be0 | gate |
| p2-r13-co.md (rc 2; co-75/76/79; 0.8 %) | sonnet medium, haiku personas, 62a04be0 | running s..NyQZ7DssHvkH8nt82 |
| mc-page-end-r1.md (rc 1, F7; rv9ka-1 MAJOR; 1.2 %) | opus high, mc-page-end-r1 42418379 | starting |
| xp-tail-r1.md (F7; rv9kb-2; 0.6 %) | sonnet medium, xp-tail-r1 62a04be0 | starting |
| tour-rows-r1.md (F7, rc 11; 1 %) | sonnet medium, tour-rows-r1 62a04be0 | starting |
| tiling-doors-r4.md (rc 2; ti-55 design; 1.5 %) | opus high, tiling-doors-r4 e39448ad | running s..Pm5veLHCLdY1xzMcC |
| act-wait-r1.md (rc 2; ti-56; 0.6 %) | sonnet medium, act-wait-r1 62a04be0 | running s..LCw4rBW8Em2dqQzgX |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) t140 green 62a04be0. TRAIN141 running. mc-minor r3 MERGE-OK 42418379 to t142; rmm3-1, -2 MINOR to backlog.
2. (rc 2) construct: co-45 fixed (co-polish r1, t141); co-44 open (trace lane after P2-R13-co); P2-R13-co running (co-75/76/79).
3. (rc 2) P2-R10-tiling: not met (ti-55 T7 5th red: persona ties 'xp look' to tag 2). TILING-DOORS r4 (opus high: product door or goal order; 1.5 %).
4. (F7, rc 1, 11) take r2: 1 same, 88 open, 10 regr. Judges r2a, r2b done: all caused and intended but rv9ka-1 MAJOR mc (MC-PAGE-END r1, opus); MINOR rv9kb-2 xp (XP-TAIL r1); rows and tour tools (TOUR-ROWS r1). rv9ka-2, rv9kb-3, -4 backlog.
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku); CI-PR dry run (sonnet). ACT-WAIT r1 (sonnet) running: act.py 9.7 s on an unbound key > 8 s chip.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
