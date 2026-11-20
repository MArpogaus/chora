# State

## Now (2026-11-20 13:22 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 09:13: 61 % (half other agent) at 59 % time; 39 % left in 15 h = 2.6 %/h shared: under-spending (critic 13:05); tokens.py 12:51 PM 9.1 %; fill to 4-5 lanes
session branch: d81c8293; 46 greens. t87e green 12:50 (--jobs 1; mc-fix, xp-24, core-l, flakes-k, xp-36 in). t87-t87d reds: flakes FLK-LAY1, FLK-HUD2, ink@2 under load (flakes-l r2); K5 race fixed by co-r5.
burn-down: audit-f r3 on 79c8d2b5: F1-F4, F6, F8 met; F5 open (flakes only); F7 by design (next wk); F8 holds with docs-f8e (t85). 0.6 d. F4/F6 recheck: audit-f r4 after t88.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r3 09:15: 0 breaches, xp-35 fixed; xp-36 fixed 3290f1d4 (merged t87e), 2 harness reds; mc r3 12:22: mc-42 reopened, mc-fix r3, r4 after |
| 3 | Three greens in a row | 30b, 33-40 green (9) |
| 4 | CI green | green on 29e2cd77 13:13 |
| 5 | Packaging smoke | arch, rpm, podman green 29e2cd77 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | train 14, 17 |
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| review-mc-fix-r3.md (RC 2: seeded review of mc-fix b0997b71) | opus high, claude/mc-fix b0997b71 | starting |
| co-majors-r4.md (shared bar no stray keys: co-25, co-26) | sonnet high, claude/co-majors-r4 d81c8293 | session_016iS3kMcC5iEnUXoQQKkZ4G 12:56 |
| flakes-m-r1.md (F5: cube-queue, FJ-R2, xp helper word) | sonnet high, claude/flakes-m d81c8293 | session_016iS3kQkhdam4gPFKKZCUse 12:56 |
| flakes-l-r2.md (F5: 6 gate flakes) | sonnet high, claude/flakes-l | session_016iS3kN8oKpH32yd68zAAp2 11:58 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) Trains: t88 co-r5 c2b0310f on d81c8293 (head fee9c695, --jobs 1) in the gate from 12:51; next: mc-fix b0997b71 after review r3 (opus 0.6 %); fuzzel 1.11.1 PATH.
5. (F5, F4, shared bar) running: FLAKES-L r2 1.2 %, FLAKES-M r1 1.2 %, CO-MAJORS r4 1.5 %, REVIEW MC-FIX r3 0.6 % then mc r4 1 %; next: AUDIT-F r4 opus 0.6 % after t88 (F4, F6 recheck), LINT-DOWN r3 (F4) 0.3 %. Greens at --jobs 1 do not count for F5.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
