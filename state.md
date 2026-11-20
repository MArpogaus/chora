# State

## Now (2026-11-20 03:43 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 01:45: 40 %; tokens.py 03:13 PM 5.4 %; 25 h left: 60 %, 2.4 %/h; 6 lanes full
models (owner 06:56): haiku tests; sonnet routine; opus complex, planning; Fable by need
session branch: 79c8d2b5 (train 81); 43 greens; MV1-4, CUT1-2, NET1-3, KIT1-7, WM1-2, WM1b-d, COPIES-0, DOCS-F8c-d, MK, MC-I5, CONSTRUCT-RED, CUBE-TESTS, MX1-M2,ID-GATE, GATE-PICK in.
burn-down: F1, F8 done; F2-F5, F7 open; F6 partly; 1.9 d. Sat (07:00): ~20 merges, 85-95 %; F1,3,4,6,8 likely, F5 open, F2 partly, F7 next wk.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r2 red (xp-35 I5) |
| 3 | Three greens in a row | 30b, 33-40 green (9); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 29e2cd77 13:13 (train 41, PR #106, all 10 checks) |
| 5 | Packaging smoke | arch, rpm, podman green 29e2cd77 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | train 14, 17 |
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| tiling-bar-r1.md (RC look, ti-21 MAJOR, Queue 5) | sonnet high | session_016iS3kM8MGNJCBw8qRRcTmr |
| voice-v12-r1.md (RC 9, V12 voice scripts red; Queue 5) | sonnet high | session_016iS3kLFi1M3X4nFLrZr54s |
| flakes-j-r1.md (F5, Queue 5; launcher-click PM review MERGE-OK, t82) | sonnet high | session_016iS3kP9upx2vf4oJ8gNtxW |
| lint-down-r2.md (F4 guard; r1 FIX: rv-1 MAJOR list vs queued merges) | sonnet high | session_016iS3kNPsBppDum7tLcMWUp |
| mc-remote-r1.md (RC 2, Queue 5; mc-46 MAJOR remote key out of a window) | sonnet high | session_016iS3kNmyiYJJg8SGuNnYcn |
| co-p2-fix-r2.md (RC bar; r1 FIX: rv-1 MAJOR swallow timer) | sonnet high | session_016iS3kNfmuVEezbtEfBfuqz |

## Queue (% week; 5 % reserve)
4. Trains (F6): t81 green 02:29 (79c8d2b5); t82b (launcher-click 741d6bca) red 03:35 tiling-look ink-2560, rerun alone. t83: p2-harness 5dbd0ccf. t84: xp-fix a767790d (harness; PM MERGE-OK). t85: stray-grab 591e98f8, black-pane 91e6a114, mc-p2-fix 17d9a798, kit-tests 40b61f28 (PM MERGE-OK).
5. F work: running: MC-REMOTE 0.8 % (mc-46 MAJOR); TILING-BAR 1 % (ti-21 MAJOR), FLAKES-J 1 %, CO-P2-FIX r2 0.8 % (rv-1 MAJOR), LINT-DOWN r2 0.4 % (F4: lints at RC; workarounds, hygiene open), VOICE-V12 1 %; then P2 r3 xp after t84; T1: tiling stop; construct reopened (co-25); AUDIT-F r3 Fri.

## Needs owner
- Delete claude/release-rc4 (403).
- Lane auto mode refused setup.sh (apt) 23:37: allow, or all tools in env setup? Default: skip it. rc.4 tag: kept local (owner).
