# State

## Now (2026-11-22 20:11 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 19:01 PM 4.9 % (Bangkok 11-22) + 0.6 % (11-23), week about 13 % (lanes est.) vs 24 % pace; under pace: 6 lanes
session branch: 6b01c976 (t108 green 13:01); 63 greens. t109g red: FLK-CUBE-SETTLE; t109h red 19:30 construct-extra K1 fly-out pose 2 px (FLAKE-K1 fixes). t110 red 20:00 xp-chat-idle (FLK-CHAT-IDLE, XP-MINOR cell race, 3/3 alone); t110b 20:12 without XP-MINOR (9ee1cef1).
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
| cube-settle-r2.md (F5, rc 3: float steps under load; 0.6 %) | sonnet medium, claude/cube-settle ffb197f3 | session_016iS3kPndgfMBpzaRMa6JGf 20:12 (r1 PARTIAL) |
| mc-helper-r1.md (rc 1: mc-57, 56, 55; 1 %) | sonnet medium, claude/mc-helper ac538e82 | session_016iS3kPf9yfyEu48CsZyHxX 19:37 |
| tiling-fix-r1.md (rc 3, 1: ti-37, 27, 25c; 1 %) | sonnet medium, claude/tiling-fix 4e46028b | session_016iS3kLAsJPbPUQ193bX8zu 19:25 |
| core-fix-r1.md (rc 9: mch-1..4, ratchet 362; 0.8 %) | sonnet medium, claude/core-fix 46e98b3e | session_016iS3kLGtC2Po7TiyJVrWmQ 19:35 ok |
| xp-helper-r2.md (rc 1: rx-1 MAJOR greeting X, rx-2..8; 0.8 %) | sonnet medium, claude/xp-helper eda60a6b | session_016iS3kQ9ejqKsH4NhfzK8XR 19:59 (review r1 FIX) |

## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r2 after ti-38.
5. (rc 1, 3, 8, 9) t110b running, adds t109h list: 10 of t109h list (9ee1cef1). XP-MINOR 3bec0cbf waits for FLK-CHAT-IDLE fix. Then: eprintln ratchet; haiku series 5 (F5).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
