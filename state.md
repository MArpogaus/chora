# State

## Now (2026-11-19 06:16 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); tokens.py 06:16: PM 0.7 % today. use the week (owner 06:04); trains (1 per 3 h) cap merges.
models: haiku tests; sonnet builds, port reviews; opus hard causes, core reviews; Fable flakes.
session branch: dc4bc46d (train 56: mv3); 24 greens; train 57 wm1b running; MV1, MV2 x3, CUT1, NET1, NET2, KIT2, WM1, KIT4, KIT1, DOCS-F8, MV3 in.
burn-down: F1 done; NET1-2, KIT1-2, KIT4, WM1 in; F2-F5, F7, F8 open (F2: every look on kit, copies 0; F3: WM1b-d); F6 partly; 2.1 days.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); open findings to known issues |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33-40 green (9); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 29e2cd77 13:13 (train 41, PR #106, all 10 checks) |
| 5 | Packaging smoke | arch, rpm, podman green on 29e2cd77 13:13 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; decision records D78-D95 train 37; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | train 14; flakes r5 train 17 |
| 11 | Tour, UX loop | v7 MERGE-OK; no tour work (owner 11-17 14:20); new tour with checks after refactor and user test (next week) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| wm1b-r1.md (F2, F3) | sonnet high, claude/wm1b | 0ffc53a8 MERGE-OK; train 57 06:16 |
| kit2b-r2.md (F2) | sonnet medium, claude/kit2b | r1 FIX; r2 session_016iS3kLM8gQdZfpG4r53S3j
| net3-r1.md (F7) | opus high, claude/net3 | session_016iS3kLZ9zKCda38a6cRs6a |
| wm2-r1.md (F3) | opus high, claude/wm2 | session_016iS3kMAkdWBVDyLsYxLUjZ |
| wm1c-r1.md (F2, F3) | sonnet high, claude/wm1c | session_016iS3kPjrW3RH4BYJ8KZtJc |
| kit6-r1.md (F2) | sonnet high, claude/kit6 | session_016iS3kQVP9HrenFDKnEYDaa |
| cut2-r1.md (F6) | sonnet high, claude/cut2 | session_016iS3kNdkNBFAYBgQRp7tcg |

## Queue (% of week; 5 % reserve)
4. Use the week (owner 06:04), critics 06:16: F2 ports first; <= 4 unmerged; reviews seeded, no gate run (sonnet for ports, opus for NET3, WM2, WM7a); slack on flakes and tests.
5. Next lanes: Fable F21/F29 flake causes (F5, F17; trains); KIT2c, WM1d, KIT2d, KIT3, KIT5 (F2); MV4 region.rs, chat.rs (F4); WM7a (F3); docs F8b (F8). Sat: opus check lane measures F1-F8 on the head. Cut: WM4, WM6, NET4. F7 open (tour next week).
6. (process) PM decisions 14:55 (owner agreed 15:35; P1 runs into next week): D87 revised (pidfd, no waitid) and D92 kept, no Fluent (KISS); cube form after WM7a; records 1-14 filed with docs F8.

## Needs owner
- F27: setup re-saved by owner 11-19 06:01; verify fuzzel >= 1.11 at the next lane start. Delete claude/release-rc4 in GitHub (session push 403). rc.4 tag: kept local (owner).
