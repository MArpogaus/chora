# State

## Now (2026-11-21 13:12 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); pace 0.6 %/h = 14 %/day, no day cap; tokens.py 12:54 week 5.2 % (6 lanes from 13:10); disk: one train at a time on the PM machine (not a lane limit); 6 lanes
session branch: 1c599c74; 56 greens; t98 green 08:46 (gate-path, xp-fields, chapters-a, xp-37, flakes-r). F5 series in the cloud (flake-series r4).
burn-down: F1-F4, F6, F8 met (audit r4); F5, F7 open past the 11-21 03:59 deadline, carried into this week (owner 11-17 15:35).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; P2-R4-CO 03:32: fails on T2 (co-41 MAJOR, no key list by mouse or F1); tiling r3 22:03: 0 breaches; T7 window under xp fixed (give-raise, t95b); harness begin r2 in t96 |
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
| co-keys-r3.md (rc 2, F7: t99 red, three construct acts) | sonnet medium, claude/co-keys f70d564f | session_016iS3kMdG5AZVg4Vkvm6R5t 09:59 |
| flk-doors-chat-r1.md (rc 2-3, F5: t99c, t99d red at doors) | sonnet medium, claude/flk-doors-chat 32a505db | session_016iS3kPXboVoSG2KpYtSdoQ 12:55 |
| flake-series-r4.md (F5: 5 full gates at --jobs 2) | haiku high, claude/flake-series-r4 1c599c74 | session_016iS3kL8UvFUknonKbwRzEb 12:55 |
| ci-head-r1.md (rc 4, 5: GitHub pre-commit red on 1c599c74) | sonnet low, claude/ci-head 1c599c74 | starting |
| core-rv-r1.md (rc 9: CORE-L-RV rv-1, rv-2, rv-5, CM-R1) | sonnet high, claude/core-rv 1c599c74 | starting |
| verify-maj-r3.md (rc 1, 9: co-36, ti-20, mc-47, A13) | sonnet medium, claude/verify-maj3 1c599c74 | starting |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) t99d red at doors (tiling launcher-chat missing, second sighting): FLK-DOORS-CHAT 0.8 % on 32a505db; its fix plus guide-checks, ti-29, mc-minor, handover-mo 73180551 form t100 with CO-KEYS r3 0.6 %. Then RECORD-V9 1.2 % on its head.
5. (plans/week-3.md D1-D2) running: CI-HEAD 0.7 % (harness: own train first), CORE-RV 1.4 %, VERIFY-MAJ r3 1 %; FLAKE-SERIES r4 1 % then FLAKES-S r1; next as lanes free: HARNESS-CELLS 1.1 %, GATE-WAITS, KEYS-ROUTE, LOOP-SLEEPS, DEPS, GATE-DRIFT; REVIEW opus 1 % after RECORD; WM4 only if F5 met.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
