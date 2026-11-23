# State

## Now (2026-11-23 06:28 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 05:56 PM 4.9 % (11-22) + 2.8 % (11-23), week about 17 % vs 33 % pace; under: 6 lanes
session branch: 2c1748a4 (t112d green 05:55: XP-IDLE, FLK-HANG, TILING-FIX, CUBE-SETTLE, FLAKE-HINT, PIX-CAPSULE, TILING-ROUND); 66 greens. t113 running 06:00 (MC-ROUND, eprintln ratchet 65 to 47).
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
| mc-round-r2.md (rc 3, 9, 1; 0.4 %) | sonnet low, claude/mc-round 9ed1fbc8 | MERGE-OK 8ba700f5; t113 running |
| review-co-focus-r1.md (rc 1, 2; 0.5 %) | opus high, claude/co-focus 866382dd | r1 done (co-50, co-51 MAJOR fixed; gate 60 PASS); review starting |
| tiling-flake-r2.md (F5, rc 3; 1 %) | sonnet medium, claude/tiling-flake 8f2ed754 | session_016iS3kPRNFtfhtkZyvMaKdM 05:55 (r1 INCOMPLETE) |
| record-v9-r3.md (F7, rc 11; 1.2 %) | sonnet medium, claude/record-v9 ec45efbb | session_016iS3kQhX8yEhfo3pGdpce4 03:53 |

## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r3 running; then fix lanes per look.
5. (rc 1, 3, 9; F5) t113 running: MC-ROUND r2, eprintln ratchet. Then haiku series 5 (5 gates at 2 jobs on the t113 head; 1 %), then fill lanes to 6.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
