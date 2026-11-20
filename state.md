# State

## Now (2026-11-20 22:01 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left; at 21:00 7 h left = up to 5 %/h shared; under-spending; tokens.py 21:00 PM 1.4 % (Bangkok day 11-21); disk 13 G free: one build (train) at a time; 4 lanes
session branch: dcb77733; 51 greens; t92c green 21:24; now --jobs 2; t93 red 21:58 (tiling-look: xp-tidy screenshot notice order vs gate-tiling-power.sh:99); t93b running without xp-tidy (PM check.py merge: review).
burn-down: audit r4 14:23: F1-F4, F6, F8 met; F5 open (bar 5/5 at --jobs 2); F7 plan accepted (tour-v9, 9.5 %), build started. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet the stop rule; co-25, co-33 fixed (co-fix MERGE-OK); co r3 17:12 fails; co-pane review r2 FIX (rw-1 MAJOR: xp Super chords lost in use mode): co-pane r3; tiling r2 17:04 fails (I3, T7): handover-ti review r2 MERGE-OK (t92), then P2-R3-TILING |
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
| review-tour-one-r2.md (F7, RC 11: r2 done c7bac047, rt-1..rt-7 fixed) | sonnet high, claude/tour-one c7bac047 | session_016iS3kM7JmGq5WqN6S6naDo 21:46 |
| guide-checks-r2.md (F7: review r1 FIX, rg-1..rg-3 MAJOR) | sonnet medium, claude/guide-checks a78b1a07 | session_016iS3kNHeeU1tRAuu1PvRx4 21:07 |
| xp-tidy-r4.md (F6: t93 red, gate-tiling-power.sh:99 stale) | sonnet medium, claude/xp-tidy efe85512 | starting 22:05 |
| flake-series-r3.md (F5: 5 gates in a row at --jobs 2) | haiku high, claude/flake-series-r3 dcb77733 | session_016iS3kM9VU8jgbUii5hViVY 21:29 |
| p2-r3-tiling.md (RC 2: tiling round 3 after handover-ti) | sonnet medium, claude/p2-r3-tiling dcb77733 | session_016iS3kLkKwwSV6HmeApSXPK 21:29 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) t93b at --jobs 2: co-pane 79787d9f with co-fix, tour-diff 4f4f1897; XP-TIDY r4 0.3 % (stale gate line) then next train; PM fixes dcb77733 and t93 check.py merge to the next seeded review.
5. (F5, F7, rc 2) Running: REVIEW-TOUR-ONE r2 0.5 %, GUIDE-CHECKS r2 0.7 % (after tour-one), FLAKE-SERIES r3 haiku 1 %, P2-R3-TILING 1 %; after t93: P2-R4-CO; tour-v9 4-5 wait for xp-give.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
