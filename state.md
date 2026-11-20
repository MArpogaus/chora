# State

## Now (2026-11-20 04:56 UTC)
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
| verify-maj-r2.md (RC 1, rule 6: ti-20, xp-24 asserts; Queue 5) | sonnet medium | session_016iS3kQa2hAkn1uRKgDKcHT |
| audit-f-r3.md (F1-F8 Friday audit, slices; Queue 5) | opus high | starting |
| flakes-j-r1.md (F5, Queue 5; launcher-click PM review MERGE-OK, t82) | sonnet high | session_016iS3kP9upx2vf4oJ8gNtxW |
| xp-p2r3-r1.md (RC 2, shared bar: face swallow, XPF-R1, co rw-1/2; Queue 5) | sonnet high | session_016iS3kNNbYNpanv6161XNnY |
| co-majors-r3.md (RC 2, rule 6: co-27, co-29 MAJOR, co-28; Queue 5) | sonnet high | session_016iS3kNjVuFx3uAeQzWEbHq |
| core-minors-r1.md (F5, F8 core MINORs; Queue 5) | sonnet high | session_016iS3kNyrbRy98nAwMrpmiC |

## Queue (% week; 5 % reserve)
4. Trains (F6): t83 (launcher-click, p2-harness, voice-v12, xp-fix) --jobs 2. t84 harness: debt-f4 fc404a90, harness-k 0dfdad8d. t85: co-p2-fix ab4cbb4f, stray-grab 591e98f8, black-pane 91e6a114, mc-p2-fix 17d9a798, mc-remote dac835dd, kit-tests 40b61f28, tiling-bar 3a9a426f, kit-minors 7a7e00a2.
5. F work: running: VERIFY-MAJ r2 0.6 % (rule 6), CO-MAJORS r3 1.2 % (rule 6), FLAKES-J 1 %, XP-P2R3 1 %, CORE-MINORS 1 % (F5); after t85: LINT-DOWN r3 0.1 %, co-26 pane; T1: tiling stop; construct reopened (co-25); AUDIT-F r3 2 % running.

## Needs owner
- Delete claude/release-rc4 (403).
- Lane auto mode refused setup.sh (apt) 23:37: allow, or all tools in env setup? Default: skip it. rc.4 tag: kept local (owner).
