# State

## Now (2026-11-21 00:07 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left; at 21:00 7 h left = up to 5 %/h shared; under-spending; tokens.py 00:05 PM 2.2 % (Bangkok day 11-21); disk 11 G free (t93b): one build (train) at a time; 5 lanes
session branch: dcb77733; 51 greens; t94 red 23:16 again only construct-hud:138 at --jobs 2 (co-35, twice since co-pane: lane co-hud-load); t94b running 23:18 at --jobs 1 (procedure).
burn-down: audit r4 14:23: F1-F4, F6, F8 met; F5 open (bar 5/5 at --jobs 2); F7 plan accepted (tour-v9, 9.5 %), build started. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; co r3 fails, co-pane MERGE-OK (t93b), then P2-R4-CO; tiling r3 22:03: 0 breaches; open: T7 window under xp (PM MAJOR: GIVE-RAISE), harness begin MERGE-OK 4f07fe6c (t95), xp Start after Super+Return |
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
| xp-super-start-r2.md (RC 2, sh-23 MAJOR: r1 FIX xs-1, replay the ricer steps) | sonnet medium, claude/xp-super-start 31736130 | session_016iS3kMmzSCh2upau5SUh2P 00:07 |
| guide-checks-r3.md (F7: review r2 FIX, rg-9 MAJOR flaky rows, rg-10, rg-11) | sonnet medium, claude/guide-checks 4efbf430 | session_016iS3kN5ar7g9J3RE64oRKz 23:42 |
| flake-series-r3.md (F5: 5 gates in a row at --jobs 2) | haiku high, claude/flake-series-r3 dcb77733 | session_016iS3kM9VU8jgbUii5hViVY 21:29 |

## Queue (% week; 5 % reserve)
4. (F6, F5, rc 3) t94b running: co-pane, tour-diff, tour-one, xp-tidy; then the PM fixes dcb77733 and the check.py merge to a seeded review; next train 95: give-raise 4b561e79, persona-begin 4f07fe6c, co-hud-load 06ea05e4 (co-35) (PM checks MERGE-OK).
5. (F5, F7, rc 2) Running: GUIDE-CHECKS review r2 FIX 23:41, r3 0.6 %, FLAKE-SERIES r3 haiku 1 %; after t93b: P2-R4-CO, tour-v9 4-5 wait for xp-give.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
