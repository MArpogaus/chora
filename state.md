# State

## Now (2026-11-20 14:19 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 09:13: 61 % (half other agent) at 59 % time; 39 % left in 15 h = 2.6 %/h shared: under-spending (critic 13:05); tokens.py 13:47 PM 9.5 %; 6 lanes; fill to 4-5 lanes
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
| review-mc-fix-r4.md (RC 2: seeded review of mc-fix a8cd0de0) | sonnet high, claude/mc-fix a8cd0de0 | session_016iS3kPuqmxv3n5rzT1YmdJ 14:19 |
| flakes-m-r1.md (F5: cube-queue, FJ-R2, xp helper word) | sonnet high, claude/flakes-m d81c8293 | session_016iS3kQkhdam4gPFKKZCUse 12:56 |
| audit-f-r4.md (F4, F5, F6 recheck, last slices) | opus high, claude/audit-f-r4 d81c8293 | session_016iS3kQew7v6rwzyo8XyHL7 14:15 |
| flakes-l-r2.md (F5: 6 gate flakes) | sonnet high, claude/flakes-l | session_016iS3kN8oKpH32yd68zAAp2 11:58 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) Trains: t88b co-r5 c2b0310f (head fee9c695, --jobs 1) from 13:45 (t88 red: leaked probes, sh-30); next t89 probe-leak f5ac3500 alone (harness), t90 co-majors-r4 e0c3a266 + lint-down 78ec62e1 (PM MERGE-OK), mc-fix after r4 and its review.
5. (F5, F4, shared bar) PROBE-LEAK r1 MERGE-OK; running: FLAKES-L r2 1.2 %, FLAKES-M r1 1.2 %, MC-FIX r4 done a8cd0de0, REVIEW r4 0.4 %, then mc persona r4 1 %; running AUDIT-F r4 opus 0.6 % (F4, F5, F6 recheck); LINT-DOWN r3 MERGE-OK (F4 lint counts gated). Greens at --jobs 1 do not count for F5.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
