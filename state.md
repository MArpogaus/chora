# State

## Now (2026-11-22 15:15 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 14:56 PM day 4.3 %, week about 11.5 % (lanes est. +1.7 since 12:45); under pace: 6 lanes full
session branch: 6b01c976 (t108 green 13:01: GATE-ALIVE); 63 greens. t109 red 13:54: pull flake FLK-PULL-DRAG (4 reruns green). t109b cd7beeac (t109 + core-diag, log-down-xp, mesh-churn) red 14:35: matrix flake FLK-GIVE-POINT (rerun green). t109c same head from 14:41.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open again (critic 11-22: one series, hint act red twice; closes on flake-hint + haiku series 5 at 2 jobs on t109 head); F7 open.

## RC (plans/week-3.md 1; rc.4 waiver gone)
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: ti-38 review r2; RECORD-V9 |
| 2 | Persona round 4 | xp, mc meet; co T1 T2, tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; open: flake-hint, flake-wait |
| 4, 5 | CI green on a PR; packaging | pre-commit green on 6b01c976; CI PR at release |
| 6 | Review per area | r1 MERGE-OK 13:35 (0 MAJOR, rd-5 MEDIUM: MC-FOCUS); Thu round |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | open: DOCS-RC |
| 9 | Flakes, window loss | open: core-diag (t110), mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| review-ti-38-r2.md (F7, rc 1, 2: r2 a1f3c8d7 49/49 green; 0.5 %) | opus high, claude/ti-38 a1f3c8d7 | session_016iS3kLARzvqTXxYsJFkqS9 15:07 |
| flake-hint-r1.md (F5, rc 3: ldm-2 hint-click act; 0.8 %) | sonnet medium, claude/flake-hint 3b14c825 | session_016iS3kLq6SXjYGB7qdMUEni 12:46 |
| flake-wait-r1.md (F5, rc 3, 9: FLK-PULL-DRAG, FLK-GIVE-POINT, gate leaks; 1 %) | sonnet medium, claude/flake-wait 6b01c976 | session_016iS3kLfksC26x7cKnd3hBR 14:42 |
| xp-38-r1.md (rc 1, 3: notify card 1.04:1, twice; 0.8 %) | sonnet medium, claude/xp-38 6b01c976 | session_016iS3kLMbUMw4vZ3TDqcxR6 15:03 |
| co-46-r1.md (rc 1: hidden window keeps keys; 1 %) | sonnet high, claude/co-46 6b01c976 | starting |
| docs-rc-r1.md (rc 8: CHANGELOG, known issues, mc keys; 0.8 %) | sonnet low, claude/docs-rc 6b01c976 | session_016iS3kLAuHWp9BSjW4taJuA 13:55 |

## Queue (% week; 5 % reserve)
3. DOCS-RC r1 now (under pace); r2 on the Thu head.
4. (F7, rc 11) record-v9 r1: diff void (clock); r2 after ti-38.
5. (rc 3; 0 %) t109c from 14:41 (cd7beeac): LAUNCH-TIMER 629faf7b, TEST-GAPS 89dc32c4, LOG-DOWN-CO b974134e, SIM-LONG ce451499, SPLIT-C 96e5c804, SPLIT-B 2af7801c, LOG-DOWN-MC b843bf5f, LOOKS1-MC b0453e19 (gates pending t109).
6. (rc 9) next train: MC-FOCUS e22f824d (PM review MERGE-OK). After t109c: eprintln ratchet down; haiku series 5 at 2 jobs on its head (F5).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
