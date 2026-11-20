# State

## Now (2026-11-20 14:39 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left in 13.3 h = 2.6 %/h shared; 09:13-14:39 only 0.75 %/h: under-spending; tokens.py 13:47 PM 9.5 %; 6 lanes
session branch: d81c8293; 46 greens. t87e green 12:50 (--jobs 1; mc-fix, xp-24, core-l, flakes-k, xp-36 in). t87-t87d reds: flakes FLK-LAY1, FLK-HUD2, ink@2 under load (flakes-l r2); K5 race fixed by co-r5.
burn-down: audit-f r4 14:23 on d81c8293: F1-F4, F6, F8 met; F5 open (12 flake rows after t89/t88b; bar 5/5 at --jobs 2); F7 next wk. 0.6 d.

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
| mc-fix-r5.md (RC 2: review r4 rv-r4-1 MAJOR, I5 cover rule) | sonnet high, claude/mc-fix a8cd0de0 | session_016iS3kN4aqRF2eXQgSQEZof 14:31 |
| flakes-m-r1.md (F5: cube-queue, FJ-R2, xp helper word) | sonnet high, claude/flakes-m d81c8293 | session_016iS3kQkhdam4gPFKKZCUse 12:56 |
| flakes-n-r1.md (F5: rw-3 panel test, MX-CAP) | sonnet high, claude/flakes-n f5ac3500 | session_016iS3kQcJMyvpDghzxqqp5X 14:24 |
| flakes-l-r2.md (F5: 6 gate flakes) | sonnet high, claude/flakes-l | session_016iS3kN8oKpH32yd68zAAp2 11:58 |
| gate-silent-r1.md (F5/F6: SKIP not PASS, fuzzel guard, MC-47-CELL) | sonnet high, claude/gate-silent f5ac3500 | starting |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) Trains: t88b co-r5 c2b0310f (head fee9c695, --jobs 1) from 13:45 (t88 red: leaked probes, sh-30); next t89 probe-leak f5ac3500 alone (harness), t90 co-majors-r4 e0c3a266 + lint-down 78ec62e1 (PM MERGE-OK), mc-fix after r4 and its review.
5. (F5, F4, shared bar) PROBE-LEAK, LINT-DOWN r3 MERGE-OK; running FLAKES-L r2 1.2 %, FLAKES-M r1 1.2 %, FLAKES-N r1 0.5 %, MC-FIX r5 0.5 % (review r4 FIX: rv-r4-1 MAJOR), then a PM review, then mc persona r4 1 %; AUDIT-F r4: F5 open 12 rows; GATE-SILENT r1 0.5 %; later FLAKE-SERIES r3 haiku 1 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
