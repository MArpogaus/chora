# State

## Now (2026-11-20 15:22 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left in 13.3 h = 2.6 %/h shared; 09:13-14:39 only 0.75 %/h: under-spending; tokens.py 14:48 PM 9.8 %; 6 lanes
session branch: fee9c695; 47 greens. t89 red xp-question 1x Yes 32 px (flake: 3/3 green alone; PM selftests shared the host); t89b running (--jobs 1). t88b green 14:49.
burn-down: audit-f r4 14:23 on d81c8293: F1-F4, F6, F8 met; F5 open (12 flake rows after t89/t88b; bar 5/5 at --jobs 2); F7 next wk. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r3 09:15: 0 breaches, xp-35 fixed; xp-36 fixed 3290f1d4 (merged t87e), 2 harness reds; mc r4 15:21: mc-42 FAIL first arrival (4th hit), stop rule not met; mc-fix r6 opus, r5 after |
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
| flakes-o-r1.md (F5: FLK-CUBE-STILL, FLK-XP-HW) | sonnet high, claude/flakes-o ecfa2029 | session_016iS3kLHNi8VhaXCo2eoCGT 15:21 |
| flakes-n-r1.md (F5: rw-3 panel test, MX-CAP) | sonnet high, claude/flakes-n f5ac3500 | session_016iS3kQcJMyvpDghzxqqp5X 14:24 |
| flakes-l-r2.md (F5: 6 gate flakes) | sonnet high, claude/flakes-l | session_016iS3kN8oKpH32yd68zAAp2 11:58 |
| mc-fix-r6.md (RC 2: mc-42 first arrival, I3 chat over window) | opus high, claude/mc-fix bb7cbeb8 | starting |
| p2-r4-xp.md (RC 2: xp persona r4 after xp-36) | sonnet medium, claude/p2-r4-xp fee9c695 | session_016iS3kMyJL9W3hiLkAUZZ12 15:19 |
| p2-r2-co.md (RC 2: construct persona r2 after co fixes) | sonnet medium, claude/p2-r2-co 59efbc17 | session_016iS3kLgjQ9oGAwjM4rQrXz 15:08 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) Trains: t89b probe-leak f5ac3500 alone (running); t90 gate-silent baac5dcf alone (harness); t91 co-majors-r4 e0c3a266 + lint-down 78ec62e1 + mc-fix bb7cbeb8 + flakes-m ecfa2029; flakes-n after review. No PM selftests during a train.
5. (F5, F4, shared bar) FLAKES-L r2 1.2 %, FLAKES-M r1, MC-FIX r5 MERGE-OK; FLAKES-N r1 0.5 %, FLAKES-O r1 1 %, MC-FIX r6 opus 2 %, P2-R2-CO 1 %, P2-R4-XP 1 %; AUDIT-F r4: F5 open 12 rows; later FLAKE-SERIES r3 haiku 1 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
