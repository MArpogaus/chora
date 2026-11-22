# State

## Now (2026-11-22 21:40 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 21:38 PM 4.9 % (Bangkok 11-22) + 1.3 % (11-23), week about 14 % (lanes est.) vs 28 % pace; under pace: 6 lanes
session branch: 6b01c976 (t108 green 13:01); 63 greens. Reds: t109g FLK-CUBE-SETTLE; t109h K1 pose (FLAKE-K1); t110 FLK-CHAT-IDLE; t110b 21:20 FLK-PIX-CAPSULE (1/4 under load, 3/3 alone). t110c 21:38 = 9ee1cef1 + core-fix + mc-helper (40a2ba92).
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
| mc-round-r1.md (rc 3, 9, 1: FLK-MC-SCALE, MH-2, MC-HEAD, MH-1; 1 %) | sonnet medium, claude/mc-round 42585079 | session_016iS3kPDDmVwVQ6Mocgk14G 21:40 |
| tiling-fix-r1.md (rc 3, 1: ti-37, 27, 25c; 1 %) | sonnet medium, claude/tiling-fix 011db749 | done 21:28, PM review MERGE-OK (tf-1, tf-2 MINOR); t111 |
| xp-idle-r1.md (F5, rc 3: FLK-CHAT-IDLE, xh-1; 0.5 %) | sonnet medium, claude/xp-idle bd5af4b3 | session_016iS3kM1upUA2pS1qWuZ3JY 21:38 (xp-helper r2 MERGE-OK, rides in it) |

## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (F7, rc 11) record-v9 r2 after ti-38.
5. (rc 1, 3, 8, 9) t110c running: t110b list + CORE-FIX, MC-HELPER (40a2ba92); t111: XP-IDLE (carries XP-MINOR, XP-HELPER), TILING-FIX, CUBE-SETTLE, FLAKE-HINT as they pass. Then: eprintln ratchet; haiku series 5 (F5).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
