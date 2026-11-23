# State

## Now (2026-11-23 02:23 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 01:36 PM 4.9 % (Bangkok 11-22) + 2.0 % (11-23), week about 15 % vs 31 % pace; under: 6 lanes
session branch: 40a2ba92 (t110f green 01:32: t110b list + CORE-FIX, MC-HELPER); 64 greens. t110e red FLK-HANG (lane flk-hang); t111 01:33: HARNESS-LEAK alone.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open (closes on the flake lanes + haiku series 5 at 2 jobs); F7 open.

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co T1 T2, tiling I2 T1 T3 open |
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
| flake-hint-r1.md (F5, rc 3) | sonnet medium, claude/flake-hint d2634fe0 | MERGE-OK; t112 |
| cube-settle-r2.md (F5, rc 3) | sonnet medium, claude/cube-settle e905195e | MERGE-OK; t112 |
| mc-round-r1.md (rc 3, 9, 1; 1 %) | sonnet medium, claude/mc-round 42585079 | MERGE-OK 9ed1fbc8; t112 |
| tiling-fix-r1.md (rc 3, 1; 1 %) | sonnet medium, claude/tiling-fix 011db749 | MERGE-OK; t112 |
| harness-leak-r1.md (F5, rc 3, 9; 0.5 %) | sonnet medium, claude/harness-leak 6b01c976 | MERGE-OK 7982d549; own train t111 first |
| tiling-round-r1.md (rc 2, 1; 1 %) | sonnet medium, claude/tiling-round 011db749 | MERGE-OK 8721b4d6; t112 |
| tiling-flake-r1.md (F5, rc 3, 1: FLK-TILING-LAYOUT, T1b; 0.8 %) | sonnet medium, claude/tiling-flake 8721b4d6 | starting |
| pix-capsule-r1.md (F5, rc 3; 0.6 %) | sonnet medium, claude/pix-capsule e905195e | MERGE-OK 05828b32; t112 |
| flk-hang-r1.md (F5, rc 3; 0.3 %) | sonnet low, claude/flk-hang 6b01c976 | MERGE-OK 022fa50c; t112 |
| record-v9-r2.md (F7, rc 11: TOUR-CLOCK, TOUR-LAUNCH, retake; 1.2 %) | sonnet medium, claude/record-v9 d7a80ff8 | session_016iS3kP1X4RaM1Q7xBpdxSa 01:38 |
| xp-idle-r1.md (F5, rc 3; 0.5 %) | sonnet medium, claude/xp-idle db7a7acd | MERGE-OK; t112 |

## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r2 running (ti-38 in 40a2ba92).
5. (rc 1, 3, 8, 9) t111 running: HARNESS-LEAK alone; t112: XP-IDLE, MC-ROUND, FLK-HANG, TILING-FIX, CUBE-SETTLE, FLAKE-HINT, PIX-CAPSULE, TILING-ROUND as they pass. Then: haiku series 5 (F5); co persona retest (rc 2, T2 via co-42; haiku); eprintln ratchet.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
