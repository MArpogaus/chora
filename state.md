# State

## Now (2026-11-20 16:20 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left in 13.3 h = 2.6 %/h shared; under-spending; tokens.py 15:46 PM 10.3 %; 6 lanes
session branch: fee9c695; 47 greens. t89 red xp-question 1x Yes 32 px; t89b running.
burn-down: audit-f r4 14:23 on d81c8293: F1-F4, F6, F8 met; F5 open (12 flake rows after t89/t88b; bar 5/5 at --jobs 2); F7 next wk. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4 15:35 meets the stop rule; mc r4 15:21: mc-42 FAIL first arrival (4th hit), stop rule not met; mc-fix r6 MERGE-OK, mc r5 running; co r2 not met; co-25 fixed (co-fix r1 MERGE-OK); co-33 hint in co-fix r2, co r3 after t92 |
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
| p2-r5-mc.md (RC 2: persona r5 on mc) | sonnet medium, claude/p2-r5-mc e7f2410a | starting |
| xp-minor-r1.md (RC 2 known issues: 5 xp MINOR) | sonnet medium, claude/xp-minor fee9c695 | session_016iS3kLKnKNMM5NJqp8UV7Y 15:58 |
| co-fix-r2.md (RC 2: co-33 one-screen pull hint) | sonnet medium, claude/co-fix e97b55f0 | session_016iS3kNHGpdqC8kd76BiPhK 16:13 |
| ti-minor-r1.md (RC 2 known issues: ti-5, ti-6/19, ti-7) | sonnet medium, claude/ti-minor fee9c695 | session_016iS3kMtfHHYuxz7yDwxtzX 16:18 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) t89b probe-leak f5ac3500 alone (running); t90 gate-silent baac5dcf, t91 gate-orphan 437173e1: each alone (harness); t92 co-majors-r4 e0c3a266 + lint-down 78ec62e1 + mc-fix 929de144 + flakes-m ecfa2029 + flakes-n d7c69aac + persona-check e3763f69 + co-fix e97b55f0.
5. (F5, F4, shared bar) MERGE-OK: FLAKES-M r1, MC-FIX r6, FLAKES-N r2, PERSONA-CHECK r1, CO-FIX r1, GATE-ORPHAN r1. Running: FLAKES-L r2 1.2 %, FLAKES-O r1 1 %, P2-R5-MC 1 %, XP-MINOR r1 1.5 %, CO-FIX r2 1 %, TI-MINOR r1 1.5 %. AUDIT-F r4: F5 open 12 rows; later FLAKE-SERIES r3 haiku 1 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
