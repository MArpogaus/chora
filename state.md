# State

## Now (2026-11-22 18:30 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 17:46 PM 4.9 % (Bangkok 11-22) + 0.3 % (11-23), week about 13 % (lanes est.) vs 24 % pace; under pace: 6 lanes
session branch: 6b01c976 (t108 green 13:01: GATE-ALIVE); 63 greens. t109b-e red on flakes. t109f red 17:49: FLK-TX-STALL (core-diag unit, wall time). t109g 356e22ef from 17:50 without core-diag.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open again (critic 11-22: one series, hint act red twice; closes on flake-hint + haiku series 5 at 2 jobs on t109 head); F7 open.

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co T1 T2, tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; open: flake-hint, FLAKE-WAIT, FLAKE-K1 (t110), FLK-K1b, mc-scale |
| 4, 5 | CI green on a PR; packaging | pre-commit green on 6b01c976; CI PR at release |
| 6 | Review per area | r1 MERGE-OK 13:35 (0 MAJOR, rd-5 MEDIUM: MC-FOCUS) |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 MERGE-OK (next train); r2 pr-1..5 |
| 9 | Flakes, window loss | open: core-diag r3, mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| flake-hint-r1.md (F5, rc 3: hint act; 0.8 %) | sonnet medium, claude/flake-hint 3b14c825 | session_016iS3kLq6SXjYGB7qdMUEni 12:46 |
| flake-k1-r2.md (F5, rc 3: FLK-K1b; 0.6 %) | sonnet medium, claude/flake-k1 308f0915 | session_016iS3kNr7GYRx24ttW9gkp2 17:13 |
| mc-scale-r1.md (F5, rc 3: FLK-MC-SCALE; 0.8 %) | sonnet medium, claude/mc-scale 6b01c976 | session_016iS3kP1gjagETR7McCFgR3 17:03 |
| core-diag-r3.md (F5, rc 3: FLK-TX-STALL; 0.6 %) | sonnet medium, claude/core-diag 788605f4 | session_016iS3kLKNX1cM4LDH1m9k8n 17:51 |
| tiling-theme-r1.md (rc 3, 1: ti-39, ti-11; 1 %) | sonnet high, claude/tiling-theme 6b01c976 | session_016iS3kNVHNyLaK85Wp7GsnZ 17:59 |

## Queue (% week; 5 % reserve)
3. DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r2 after ti-38.
5. (rc 3; 0 %) t109g from 17:50 (356e22ef): LAUNCH-TIMER 629faf7b, TEST-GAPS 89dc32c4, LOG-DOWN-CO b974134e, SIM-LONG ce451499, SPLIT-C 96e5c804, SPLIT-B 2af7801c, LOG-DOWN-MC b843bf5f, LOOKS1-MC b0453e19.
6. (rc 1, 8, 9) next train: MC-FOCUS e22f824d, TI-38 a1f3c8d7, DOCS-RC 54c30d16, TI-MINOR 136bbd82, FLAKE-WAIT 176938d1, XP-38 c7f2e64f, FLAKE-K1 308f0915, CO-46 83033ed6, XP-MINOR 3bec0cbf (PM MERGE-OK). After t109c: eprintln ratchet down; haiku series 5 at 2 jobs after flake lanes (F5).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
