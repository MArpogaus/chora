# State

## Now (2026-11-23 00:13 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 00:09 PM 4.9 % (Bangkok 11-22) + 1.7 % (11-23), week about 14 % (lanes est.) vs 28 % pace; under pace: 6 lanes
session branch: 6b01c976 (t108 green 13:01); 63 greens. Reds t109g-t110b (fixed in lanes). t110c red: load of leaked compositors (HARNESS-LEAK), clean 3/3 at 40a2ba92; t110d lost to a container restart 23:16; t110e red 00:12 FLK-HANG (test race, hang.rs:346); t110f 00:14 same head.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open (closes on the flake lanes + haiku series 5 at 2 jobs); F7 open.

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co T1 T2, tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; open: FLAKE-WAIT, FLAKE-K1, FLK-K1b, MC-SCALE, PIX-CAPSULE, HANG |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 MERGE-OK (next train); r2 pr-1..5 |
| 9 | Flakes, window loss | open: cube-settle, mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| flake-hint-r1.md (F5, rc 3) | sonnet medium, claude/flake-hint d2634fe0 | MERGE-OK; t111 |
| cube-settle-r2.md (F5, rc 3) | sonnet medium, claude/cube-settle e905195e | MERGE-OK; t111 |
| mc-round-r1.md (rc 3, 9, 1: FLK-MC-SCALE, MH-2, MC-HEAD, MH-1; 1 %) | sonnet medium, claude/mc-round 42585079 | session_016iS3kPDDmVwVQ6Mocgk14G 21:40 |
| tiling-fix-r1.md (rc 3, 1: ti-37, 27, 25c; 1 %) | sonnet medium, claude/tiling-fix 011db749 | MERGE-OK; t111 |
| harness-leak-r1.md (F5, rc 3, 9: HARNESS-LEAK; 0.5 %) | sonnet medium, claude/harness-leak 6b01c976 | session_016iS3kLAyzZ7Uff7AJRj4wH 22:57 |
| tiling-round-r1.md (rc 2, 1: T1, T3, I2, TF-1; 1 %) | sonnet medium, claude/tiling-round 011db749 | 00:05 INCOMPLETE (gate cut); I2, TF-1 ok; T3 closed; T1: D55 holds, T1b new; gate rerun |
| pix-capsule-r1.md (F5, rc 3; 0.6 %) | sonnet medium, claude/pix-capsule e905195e | session_016iS3kQCc5SxBL77HLj5CXY 00:10 |
| flk-hang-r1.md (F5, rc 3; 0.3 %) | sonnet low, claude/flk-hang 6b01c976 | starting |
| xp-idle-r1.md (F5, rc 3: FLK-CHAT-IDLE, xh-1; 0.5 %) | sonnet medium, claude/xp-idle db7a7acd | MERGE-OK; t111 |

## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r2 after ti-38.
5. (rc 1, 3, 8, 9) t110f running: t110b list + CORE-FIX, MC-HELPER (40a2ba92); t111: XP-IDLE (carries XP-MINOR, XP-HELPER), TILING-FIX, CUBE-SETTLE, FLAKE-HINT as they pass. Then: eprintln ratchet; haiku series 5 (F5).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
