# State

## Now (2026-11-22 17:02 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 16:15 PM day 4.9 %, week about 13 % (lanes est.) vs 24 % pace; under pace: 6 lanes full
session branch: 6b01c976 (t108 green 13:01: GATE-ALIVE); 63 greens. t109b cd7beeac red 14:35: matrix flake FLK-GIVE-POINT (rerun green). t109c red 15:37: FLK-K1 (2nd). t109d red 16:46: mc-scale FLK-MC-SCALE (new, load). t109e from 16:47.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open again (critic 11-22: one series, hint act red twice; closes on flake-hint + haiku series 5 at 2 jobs on t109 head); F7 open.

## RC (plans/week-3.md 1; rc.4 waiver gone)
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 r2 MERGE-OK 15:15, next train) |
| 2 | Persona round 4 | xp, mc meet; co T1 T2, tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; open: flake-hint, flake-wait |
| 4, 5 | CI green on a PR; packaging | pre-commit green on 6b01c976; CI PR at release |
| 6 | Review per area | r1 MERGE-OK 13:35 (0 MAJOR, rd-5 MEDIUM: MC-FOCUS); Thu round |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 MERGE-OK (next train); r2 pr-1..5 |
| 9 | Flakes, window loss | open: core-diag (t110), mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| flake-hint-r1.md (F5, rc 3: hint act; 0.8 %) | sonnet medium, claude/flake-hint 3b14c825 | session_016iS3kLq6SXjYGB7qdMUEni 12:46 |
| flake-wait-r1.md (F5, rc 3, 9; 1 %) | sonnet medium, claude/flake-wait 6b01c976 | session_016iS3kLfksC26x7cKnd3hBR 14:42 |
| xp-38-r1.md (rc 1, 3; 0.8 %) | sonnet medium, claude/xp-38 6b01c976 | session_016iS3kLMbUMw4vZ3TDqcxR6 15:03 |
| co-46-r1.md (rc 1: hidden keys; 1 %) | sonnet high, claude/co-46 6b01c976 | session_016iS3kPkq3PWcVkMRxyrwRU 15:16 |
| flake-k1-r1.md (F5, rc 3: FLK-K1; 0.6 %) | sonnet medium, claude/flake-k1 6b01c976 | session_016iS3kLrYViY6AbWZ3NAnes 15:39 |

## Queue (% week; 5 % reserve)
3. DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r1: diff void (clock); r2 after ti-38.
5. (rc 3; 0 %) t109e from 16:47 (cd7beeac): LAUNCH-TIMER 629faf7b, TEST-GAPS 89dc32c4, LOG-DOWN-CO b974134e, SIM-LONG ce451499, SPLIT-C 96e5c804, SPLIT-B 2af7801c, LOG-DOWN-MC b843bf5f, LOOKS1-MC b0453e19.
6. (rc 1, 8, 9) next train: MC-FOCUS e22f824d, TI-38 a1f3c8d7, DOCS-RC 54c30d16, TI-MINOR 136bbd82 (PM MERGE-OK); D119 at merge. After t109c: eprintln ratchet down; haiku series 5 at 2 jobs after the flake lanes (F5); FLK-MC-SCALE lane (sonnet, 0.8 %) at a free slot.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
