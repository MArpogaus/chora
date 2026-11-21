# State

## Now (2026-11-21 16:18 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 16:01 week 5.6 % (0.47 %/h since reset, under pace; 6 lanes); disk: one train at a time on the PM machine (not a lane limit); 6 lanes
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
| 6 | Code review per area | 08:59 and delta r1 16:11 (to 1c599c74): MERGE-OK, 0 MAJOR; rd-2 MEDIUM in xp-rd |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | t14, t17 |
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| co-hint-r1.md (rc 2, F7: co-42 hint) | sonnet medium, claude/co-hint c69354ae | session_016iS3kL6rhVtqF6gejqFJ7c 14:10 |
| p2-harness-l-r1.md (rc 2: persona L1-L3, ph-1, ph-2, RPM-1) | sonnet medium, claude/p2-harness-l a84effda | session_016iS3kMzndQRcQR2ZTYuC5D 16:16 |
| xp-rd-r1.md (rc 6: review-delta rd-2 MEDIUM slider, rd-3 wheel) | sonnet medium, claude/xp-rd a84effda | session_016iS3kPdsb7N22TV7u8MqWn 16:12 |
| flake-series-r4.md (F5: 5 full gates at --jobs 2) | haiku high, claude/flake-series-r4 1c599c74 | session_016iS3kL8UvFUknonKbwRzEb 12:55 |
| ti-xdg-r1.md (rc 6: one runtime-dir rule, mkdir guard) | sonnet low, claude/ti-xdg a84effda | starting |
| gate-waits-r1.md (F5, rc 3: sleeps to waits in six flaky cells) | sonnet medium, claude/gate-waits 1c599c74 | session_016iS3kMaugq8mKyEhmsaZMe 14:01 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) t101 running on a84effda: guide-checks, ti-29, mc-minor, handover-mo, CO-KEYS r3, CORE-RV, FLK-DOORS-CHAT. Then RECORD-V9 1.2 % on its head; t102: GATE-DRIFT 96a91f1f (harness first), DEPS a606a00f (both PM review MERGE-OK).
5. (plans/week-3.md D1-D2) running: GATE-WAITS 1.6 %; FLAKE-SERIES r4 1 % then FLAKES-S r1; TI-XDG 0.82 %; P2-HARNESS-L 0.95 %; next as lanes free: KEYS-ROUTE, LOOP-SLEEPS (after co-hint); REVIEW 1 % after RECORD; REVIEW-DELTA r1 MERGE-OK 16:11, r2 on the Thu head; XP-RD 0.6 %.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
