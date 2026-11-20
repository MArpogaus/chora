# State

## Now (2026-11-20 16:37 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left in 13.3 h = 2.6 %/h shared; under-spending; tokens.py 15:46 PM 10.3 %; 6 lanes
session branch: ce2c436b; 48 greens (t89b green 16:25); t90 gate-silent running.
burn-down: audit-f r4 14:23 on d81c8293: F1-F4, F6, F8 met; F5 open (12 flake rows after t89/t88b; bar 5/5 at --jobs 2); F7 next wk. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4 15:35 meets the stop rule; mc r5 16:33 meets the stop rule (mc-42, I3 PASS; 0 breaches); co r2 not met; co-25 fixed (co-fix r1 MERGE-OK); co-33 hint in co-fix r2, co r3 after t92 |
| 3 | Three greens in a row | 30b, 33-40 green (9); latest 88b, 89b |
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
| kit-minor-r1.md (kit MINOR: Book origin, kit7 r2 helper) | sonnet medium, claude/kit-minor e7f2410a | session_016iS3kN7xygsb7sS4PkKmav 16:34 |
| xp-minor-r2.md (RC 2 bar: Run... err vs Escape, pr1-1) | sonnet medium, claude/xp-minor 22884a79 | session_016iS3kQfMuef1xFRc5ak7to 16:27 |
| co-fix-r2.md (RC 2: co-33 one-screen pull hint) | sonnet medium, claude/co-fix e97b55f0 | session_016iS3kNHGpdqC8kd76BiPhK 16:13 |
| p2-r2-tiling.md (RC 2: tiling persona round 2) | sonnet medium + haiku, claude/p2-r2-tiling ce2c436b | session_016iS3kNo5CzwTTqXM8x3PEa 16:37 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) t90 gate-silent baac5dcf alone (running); t91 gate-orphan 437173e1 alone (harness); t92 co-majors-r4 e0c3a266 + lint-down 78ec62e1 + mc-fix 929de144 + flakes-m ecfa2029 + flakes-n d7c69aac + persona-check e3763f69 + co-fix e97b55f0.
5. (F5, F4, shared bar) MERGE-OK: FLAKES-M r1, MC-FIX r6, FLAKES-N r2, PERSONA-CHECK r1, CO-FIX r1, GATE-ORPHAN r1. Running: FLAKES-L r2 1.2 %, FLAKES-O r1 1 %, KIT-MINOR r1 1 %, XP-MINOR r2 0.5 %, CO-FIX r2 1 %, P2-R2-TILING 1 %. TI-MINOR r1 done (nothing to merge). Later FLAKE-SERIES r3 haiku 1 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
