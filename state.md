# State

## Now (2026-11-23 03:52 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 03:01 PM 4.9 % (Bangkok 11-22) + 2.2 % (11-23), week about 16 % vs 32 % pace; under: 6 lanes
session branch: 86e619f3 (t111 green 03:00); 65 greens. t112b red 03:51 (record-v9 RV9-5); t112c 03:53 without it (2c1748a4).
burn-down: F1-F4, F6, F8 met (audit r4); F5 open (closes on the flake lanes + haiku series 5 at 2 jobs); F7 open.

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r6 not met (co-48..51), tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; open: FLAKE-WAIT, FLAKE-K1, FLK-K1b, PIX-CAPSULE, TILING-LAYOUT |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 MERGE-OK (next train); r2 pr-1..5 |
| 9 | Flakes, window loss | open: cube-settle, mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| flake-hint-r1.md (F5, rc 3) | sonnet medium, claude/flake-hint d2634fe0 | MERGE-OK; t112c |
| cube-settle-r2.md (F5, rc 3) | sonnet medium, claude/cube-settle e905195e | MERGE-OK; t112c |
| mc-round-r2.md (rc 3, 9, 1; 0.4 %) | sonnet low, claude/mc-round 9ed1fbc8 | session_016iS3kQ7xHyiK3NbSiTGxak 03:12 |
| tiling-fix-r1.md (rc 3, 1; 1 %) | sonnet medium, claude/tiling-fix 011db749 | MERGE-OK; t112c |
| tiling-round-r1.md (rc 2, 1; 1 %) | sonnet medium, claude/tiling-round 011db749 | MERGE-OK 8721b4d6; t112c |
| co-focus-r1.md (rc 1, 2; 2 %) | opus high, claude/co-focus 86e619f3 | session_016iS3kNKsq8ZXRvDqAj6z3B 03:45 (co-48..51; p2-r6-co r6 not met) |
| tiling-flake-r1.md (F5, rc 3, 1; 0.8 %) | sonnet medium, claude/tiling-flake 8721b4d6 | session_016iS3kPS7AZ63LG2obX116Q 02:24 |
| pix-capsule-r1.md (F5, rc 3; 0.6 %) | sonnet medium, claude/pix-capsule e905195e | MERGE-OK 05828b32; t112c |
| flk-hang-r1.md (F5, rc 3; 0.3 %) | sonnet low, claude/flk-hang 6b01c976 | MERGE-OK 022fa50c; t112c |
| record-v9-r3.md (F7, rc 11; 1.2 %) | sonnet medium, claude/record-v9 ec45efbb | starting (RV9-1..5) |
| xp-idle-r1.md (F5, rc 3; 0.5 %) | sonnet medium, claude/xp-idle db7a7acd | MERGE-OK; t112c |

## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r3 running; then fix lanes per look.
5. (rc 1, 3, 8, 9) t112c running: XP-IDLE, FLK-HANG, TILING-FIX, CUBE-SETTLE, FLAKE-HINT, PIX-CAPSULE, TILING-ROUND; MC-ROUND r2. Then: haiku series 5 (F5); eprintln ratchet.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
