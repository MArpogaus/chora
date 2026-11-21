# State

## Now (2026-11-21 16:05 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); pace 0.6 %/h = 14 %/day, no day cap; tokens.py 16:01 week 5.6 % (0.47 %/h since reset, under pace; 6 lanes); disk: one train at a time on the PM machine (not a lane limit); 6 lanes
session branch: a84effda; 57 greens; t100 green 16:02 (ci-head); t101 running (7 branches). F5 series in the cloud (flake-series r4).
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
| review-delta-r1.md (rc 6: code review adf71956..1c599c74) | opus high, claude/review-delta 1c599c74 | session_016iS3kPrJT6ddDm8F9Z3L1h 16:02 |
| flake-series-r4.md (F5: 5 full gates at --jobs 2) | haiku high, claude/flake-series-r4 1c599c74 | session_016iS3kL8UvFUknonKbwRzEb 12:55 |
| gate-drift-r1.md (F6, rc 3: tool versions as SKIP) | sonnet medium, claude/gate-drift 1c599c74 | session_016iS3kPnspxzzYMoUj6sGSE 14:44 |
| deps-r1.md (rc 7: minor bumps) | sonnet low, claude/deps 1c599c74 | session_016iS3kLjqSJKWsiFCCvs8M5 14:41 |
| gate-waits-r1.md (F5, rc 3: sleeps to waits in six flaky cells) | sonnet medium, claude/gate-waits 1c599c74 | session_016iS3kMaugq8mKyEhmsaZMe 14:01 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) t101 running on a84effda: guide-checks, ti-29, mc-minor, handover-mo, CO-KEYS r3 (camera.rs conflict resolved by hand), CORE-RV, FLK-DOORS-CHAT. Then RECORD-V9 1.2 % on its head.
5. (plans/week-3.md D1-D2) running: GATE-WAITS 1.6 %; FLAKE-SERIES r4 1 % then FLAKES-S r1; DEPS 0.9 %; GATE-DRIFT 0.95 %; next as lanes free: KEYS-ROUTE, LOOP-SLEEPS (after co-hint); HARNESS-CELLS dropped; REVIEW 1 % after RECORD; REVIEW-DELTA r1 1 % now (under pace), r2 on the Thu head.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
