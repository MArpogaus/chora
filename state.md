# State

## Now (2026-11-21 02:26 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left; at 21:00 7 h left = up to 5 %/h shared; under-spending; tokens.py 00:26 PM 2.3 % (Bangkok day 11-21); disk 18 G free: one build (train) at a time; 4 lanes
session branch: 618ae513; 52 greens; t95 red 01:15 (acts: persona-begin needs a begin, co-pane's C2 selftest has none); t95b running at --jobs 1: give-raise, co-hud-load.
burn-down: audit r4 14:23: F1-F4, F6, F8 met; F5 open (bar 5/5 at --jobs 2); F7 plan accepted (tour-v9, 9.5 %), build started. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; co r3 fails, co-pane merged (t94b), P2-R4-CO after t95; tiling r3 22:03: 0 breaches; open: T7 window under xp (PM MAJOR: GIVE-RAISE), harness begin r2 (t95 red); xp Start (sh-23) not reproduced in full replay r3: known issue |
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

## Queue (% week; 5 % reserve)
4. (F6, rc 2-3) t95b running: give-raise 4b561e79, co-hud-load 06ea05e4; PERSONA-BEGIN r2 MERGE-OK dad65a95: train 96 alone (harness); flakes-q train 97; then the PM fixes dcb77733 and the check.py merge to a seeded review.
5. (F5, F7, rc 2) FLAKE-SERIES r3: 2/5 green at --jobs 2, kit 91.76 %: F5 not met. Running: GUIDE-CHECKS r4 0.5 % (xp tray exemption, co restart), FLAKES-Q fb85b156, FLAKES-P bd3ecad4 MERGE-OK: train 97; open FLK-XP-FACES, FLK-MATRIX-S1. After t95: P2-R4-CO, tour-v9 4-5.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
