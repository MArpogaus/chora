# State

## Now (2026-11-21 14:41 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); pace 0.6 %/h = 14 %/day, no day cap; tokens.py 14:01 week 5.4 % (0.54 %/h since reset; 6 lanes); disk: one train at a time on the PM machine (not a lane limit); 6 lanes
session branch: 1c599c74; 56 greens; t98 green 08:46 (gate-path, xp-fields, chapters-a, xp-37, flakes-r). F5 series in the cloud (flake-series r4).
burn-down: F1-F4, F6, F8 met (audit r4); F5, F7 open past the 11-21 03:59 deadline, carried into this week (owner 11-17 15:35).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; P2-R4-CO 03:32: fails on T2 (co-41 MAJOR, no key list by mouse or F1); tiling r3 22:03: 0 breaches |
| 3 | Three greens in a row | 30b, 33-40 green (9); latest 88b, 89b |
| 4 | CI green | green on 29e2cd77 13:13 |
| 5 | Packaging smoke | arch, rpm, podman green 29e2cd77 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | t14, t17 |
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| co-hint-r1.md (rc 2, F7: co-42 face hint gone after 3 s) | sonnet medium, claude/co-hint c69354ae | session_016iS3kL6rhVtqF6gejqFJ7c 14:10 |
| flk-doors-chat-r1.md (rc 2-3, F5: t99c, t99d red at doors) | sonnet medium, claude/flk-doors-chat 32a505db | session_016iS3kPXboVoSG2KpYtSdoQ 12:55 |
| flake-series-r4.md (F5: 5 full gates at --jobs 2) | haiku high, claude/flake-series-r4 1c599c74 | session_016iS3kL8UvFUknonKbwRzEb 12:55 |
| ci-head-r1.md (rc 4, 5: CI pre-commit red) | sonnet low, claude/ci-head 1c599c74 | session_016iS3kQdWnL898jZB77Bubx 13:12 |
| deps-r1.md (rc 7: minor Dependabot bumps) | sonnet low, claude/deps 1c599c74 | starting |
| gate-waits-r1.md (F5, rc 3: sleeps to waits in six flaky cells) | sonnet medium, claude/gate-waits 1c599c74 | session_016iS3kMaugq8mKyEhmsaZMe 14:01 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) t99d red at doors (tiling launcher-chat missing, second sighting): FLK-DOORS-CHAT 0.8 % on 32a505db; its fix plus guide-checks, ti-29, mc-minor, handover-mo 73180551 form t100 with CO-KEYS r3 c69354ae and CORE-RV 1db1a50f (xp-38 may block acts). Then RECORD-V9 1.2 % on its head.
5. (plans/week-3.md D1-D2) running: CI-HEAD 0.7 % (own train), GATE-WAITS 1.6 %; FLAKE-SERIES r4 1 % then FLAKES-S r1; DEPS 0.9 %; next as lanes free: KEYS-ROUTE, LOOP-SLEEPS (after co-hint), GATE-DRIFT; HARNESS-CELLS dropped (cells-for.sh, full gate); REVIEW 1 % after RECORD.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
