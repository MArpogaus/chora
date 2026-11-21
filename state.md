# State

## Now (2026-11-21 02:29 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left; at 21:00 7 h left = up to 5 %/h shared; under-spending; tokens.py 02:29 PM 2.7 % (Bangkok day 11-21); disk 18 G free: one build (train) at a time; 4 lanes
session branch: f4f49783; 53 greens; t95b green 02:33 (give-raise T7 MAJOR, co-hud-load co-35); t96 running: persona-begin alone (harness).
burn-down: audit r4 14:23: F1-F4, F6, F8 met; F5 open (bar 5/5 at --jobs 2); F7 plan accepted (tour-v9, 9.5 %), build started. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; co r3 fails, co-pane (t94b), co-hud-load (t95b) merged, P2-R4-CO after t96; tiling r3 22:03: 0 breaches; T7 window under xp fixed (give-raise, t95b); harness begin r2 in t96; xp Start (sh-23) not reproduced in full replay r3: known issue |
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
| guide-checks-r4.md (F7: rg-9 xp tray exemption, construct restart red) | sonnet medium, claude/guide-checks 9881ccda | session_016iS3kMwg6gNXRJSvVuuage 01:26 |
| chapters-a.md (F7: xp, tiling, handover rows checked) | sonnet high, claude/chapters-a f4f49783 | session_016iS3kLLge6driZZFcqW4i5 02:29 |
| chapters-b.md (F7: mc, construct rows checked) | sonnet high, claude/chapters-b f4f49783 | session_016iS3kQFBrWrwCWGMfrVhpP 02:29 |

## Queue (% week; 5 % reserve)
4. (F6, rc 2-3) t96 running: persona-begin dad65a95 alone (harness); then tell lanes to merge; train 97: flakes-q fb85b156, flakes-p bd3ecad4; then the PM fixes dcb77733 and the check.py merge to a seeded review.
5. (F5, F7, rc 2) FLAKE-SERIES r3: 2/5 green at --jobs 2, kit 91.76 %: F5 not met. Running: GUIDE-CHECKS r4 0.5 %; FLAKES-Q, FLAKES-P MERGE-OK: train 97; open FLK-XP-FACES, FLK-MATRIX-S1. CHAPTERS-A 1.2 %, CHAPTERS-B 1 % (tour-v9 lanes 4-5). After t96: P2-R4-CO 1 %.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
