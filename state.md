# State

## Now (2026-11-22 19:59 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 19:01 PM 4.9 % (Bangkok 11-22) + 0.6 % (11-23), week about 13 % (lanes est.) vs 24 % pace; under pace: 6 lanes
session branch: 6b01c976 (t108 green 13:01); 63 greens. t109g red: FLK-CUBE-SETTLE; t109h red 19:30 construct-extra K1 fly-out pose 2 px (FLAKE-K1 fixes). t110 from 19:50 (aeaf8138 = t109h + Queue 6; conflicts gate steps, known-red resolved).
burn-down: F1-F4, F6, F8 met (audit r4); F5 open again (critic 11-22: one series, hint act red twice; closes on flake-hint + haiku series 5 at 2 jobs on t109 head); F7 open.

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co T1 T2, tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; open: flake-hint, FLAKE-WAIT, FLAKE-K1, MC-SCALE (t110), FLK-K1b |
| 4, 5 | CI green on a PR; packaging | pre-commit green on 6b01c976; CI PR at release |
| 6 | Review per area | r1 MERGE-OK 13:35 (0 MAJOR, rd-5 MEDIUM: MC-FOCUS) |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 MERGE-OK (next train); r2 pr-1..5 |
| 9 | Flakes, window loss | open: cube-settle, mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| flake-hint-r1.md (F5, rc 3: hint act; 0.8 %) | sonnet medium, claude/flake-hint 3b14c825 | session_016iS3kLq6SXjYGB7qdMUEni 12:46 |
| cube-settle-r1.md (F5, rc 3: FLK-CUBE-SETTLE; 0.8 %) | sonnet medium, claude/cube-settle 6b01c976 | session_016iS3kLPse7BU4Uz4qMNKqV 18:53 ok |
| mc-helper-r1.md (rc 1: mc-57, 56, 55; 1 %) | sonnet medium, claude/mc-helper ac538e82 | session_016iS3kPf9yfyEu48CsZyHxX 19:37 |
| tiling-fix-r1.md (rc 3, 1: ti-37, 27, 25c; 1 %) | sonnet medium, claude/tiling-fix 4e46028b | session_016iS3kLAsJPbPUQ193bX8zu 19:25 |
| core-fix-r1.md (rc 9: mch-1..4, ratchet 362; 0.8 %) | sonnet medium, claude/core-fix 46e98b3e | session_016iS3kLGtC2Po7TiyJVrWmQ 19:35 ok |
| xp-helper-r2.md (rc 1: rx-1 MAJOR greeting X, rx-2..8; 0.8 %) | sonnet medium, claude/xp-helper eda60a6b | starting (review r1 FIX 19:58) |

## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r2 after ti-38.
5. (rc 1, 3, 8, 9) t110 running, adds t109h list: MC-FOCUS e22f824d, TI-38 a1f3c8d7, DOCS-RC 54c30d16, TI-MINOR 136bbd82, FLAKE-WAIT 176938d1, XP-38 c7f2e64f, FLAKE-K1 7d8ad513, CO-46 83033ed6, XP-MINOR 3bec0cbf, TI-THEME 4e46028b, MC-SCALE ac538e82. Then: eprintln ratchet; haiku series 5 (F5).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
