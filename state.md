# State

## Now (2026-11-20 15:58 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left in 13.3 h = 2.6 %/h shared; 09:13-14:39 only 0.75 %/h: under-spending; tokens.py 15:46 PM 10.3 %; 6 lanes
session branch: fee9c695; 47 greens. t89 red xp-question 1x Yes 32 px; t89b running.
burn-down: audit-f r4 14:23 on d81c8293: F1-F4, F6, F8 met; F5 open (12 flake rows after t89/t88b; bar 5/5 at --jobs 2); F7 next wk. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4 15:35 meets the stop rule (0 breaches; T4:clock checker); mc r4 15:21: mc-42 FAIL first arrival (4th hit), stop rule not met; mc-fix r6 opus, r5 after; co r2 15:34 not met (co-25 pane, C2), co-fix r1 |
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
| flakes-l-r2.md (F5: 6 gate flakes) | sonnet high, claude/flakes-l | session_016iS3kN8oKpH32yd68zAAp2 11:58 |
| mc-fix-r6.md (RC 2: mc-42 first arrival, I3 chat over window) | opus high, claude/mc-fix bb7cbeb8 | session_016iS3kLWs5G6iqU2TsBAiTy 15:22 |
| gate-orphan-r1.md (F6: red step leaves no orphan) | sonnet medium, claude/gate-orphan fee9c695 | session_016iS3kNWpf3LijcZ7WSxG1g 15:54 |
| xp-minor-r1.md (RC 2 known issues: 5 xp MINOR) | sonnet medium, claude/xp-minor fee9c695 | session_016iS3kLKnKNMM5NJqp8UV7Y 15:58 |
| co-fix-r1.md (RC 2: co-25 persona path, C2 pull, C1 cube row) | opus high, claude/co-fix 59efbc17 | session_016iS3kQV9F7BQukecZNFTL1 15:35 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) Trains: t89b probe-leak f5ac3500 alone (running); t90 gate-silent baac5dcf alone (harness); t91 co-majors-r4 e0c3a266 + lint-down 78ec62e1 + mc-fix bb7cbeb8 + flakes-m ecfa2029 + flakes-n d7c69aac + persona-check e3763f69 (MERGE-OK 15:56, 15:59). No PM selftests during a train.
5. (F5, F4, shared bar) MERGE-OK: FLAKES-M r1, MC-FIX r5, FLAKES-N r2, PERSONA-CHECK r1. Running: FLAKES-L r2 1.2 %, FLAKES-O r1 1 %, MC-FIX r6 opus 2 %, CO-FIX r1 opus 2 %, GATE-ORPHAN r1 0.8 %, XP-MINOR r1 1.5 %. AUDIT-F r4: F5 open 12 rows; later FLAKE-SERIES r3 haiku 1 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
