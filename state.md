# State

## Now (2026-11-20 08:42 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 01:45: 40 %; tokens.py 07:58 PM 7.6 %; 20 h left; 5-h limit hit 06:39-06:50, lanes idle to 07:57 (resumed); the 5-h window, not the week, binds
session branch: 9973d0eb (train 85c green 08:38, e2e incl. tiling-apps-click PASS with fuzzel 1.11.1: lane reds = lane fuzzel or flake); 44 greens. Train 86 (harness-l) running 08:40.
burn-down: audit-f r3 on 79c8d2b5: F1-F4, F6, F8 met; F5 open (flakes only); F7 by design (next wk); F8 holds with docs-f8e (t85). 1.9 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r2 red (xp-35 I5); mc r2 red (mc-47..49; M2 ruled out) |
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
| flake-series-r2.md (F5: 5 gates on t85b head 9973d0eb, kit cov) | haiku high | session_016iS3kMGkHjEB73ojsbgotQ |
| mc-fix-r2.md (RC 2: merge session head, resolve check.py/session.sh conflict; then PM review, t88) | sonnet high | session_016iS3kNHBMbi3VStszbq1N5 |
| p2-r3-xp.md (RC 2: xp persona r3 on c61ad6c8 = 85c + xp-24) | sonnet medium | session_016iS3kNeZESVqcoF5B5qrYP |

## Queue (% week; 5 % reserve)
4. Trains: t86 running: harness-l 75dfa4b9; t87 next: co-r4 42c9304b (contains co-majors-r3; merge-gate.sh comment conflict, PM joins both), xp-24 2d1e06c6, core-l b1d1491d, flakes-k e2508ec1; fuzzel 1.11.1 PATH.
4b. t88: mc-fix (b4104cae MERGE-OK 08:43, conflicts with 9973d0eb in check.py, session.sh): r2 merges, PM reviews the resolution.
5. F work: running: FLAKE-SERIES r2 haiku 1 % (F5), XP persona r3 1 % (RC 2), MC-FIX r2 0.4 %; next: MC persona r3 1 % (after mc-fix r2, branch claude/p2-r3-mc at 9973d0eb), LINT-DOWN r3 0.1 %, co-26 pane; FLAKES-L r2 1 % (F5), AUDIT-F r4 Sat 0.6 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
