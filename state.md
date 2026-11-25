# State

## Now (2026-11-25 07:30 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 05:33 PM 2.1 % (11-25), week about 34 % vs 55 % pace: under-spending, fill 6 lanes
session branch: 8e63e0ed (t135 05:44, sh42-fix r2); 83 greens. Lane gates miss ratchets: GATE-RATCHETS r1
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: construct co-75 BLOCK, co-76, co-79 MAJOR (CO-CLOSE r2 + CO-REACH r1 7f519bc8 MERGE-OK; CO-CLOSE-WAITS r1 MERGE-OK, train136); xp, mc: none; tiling ti-55, ti-56 reopened r9 |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r9 not met (ti-55, ti-56; TILING-DOORS r2, then P2-R10) |
| 3 | 3 greens, no flake | met: t130-t132 green; FLAKE-SERIES r9 5/5 c3c37c2e |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-33, sh-35, sh-36, sl-9 closed; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train136 (rc 1, F7) | PM, 6f37120b on 8e63e0ed | gate running |
| xp-minor-r2.md (rc 1; xp-10, -13, -30, rxm1-1; 1 %) | sonnet high, claude/xp-minor-r2 d2a048d5 | session_016iS3kQRG6ST7x32pFZswCM |
| mc-minor-r1.md (rc 1; mc-56, rpx1-9..14; 1 %) | sonnet high, claude/mc-minor-r1 721be592 | session_016iS3kMCPqruUf56LphN2U8 |
| tiling-doors-r2.md (rc 2; 1.0 %) | sonnet high, claude/tiling-doors-r2 721be592 | session_016iS3kPKrsTv7tRKpcW3uAv |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) t135 green (sh-42 closed); TRAIN136 running: stray-super-minor r1 8c3ac745, tour-base r8 27fb2599, co-close-waits r1 22bf5fb7 (+co-reach, co-close r2). TRAIN137: gate-ratchets r1 0094bffd MERGE-OK (PM, harness, alone). TRAIN138: xp-minor r1 d2a048d5 MERGE-OK (PM; rxm1-1 MINOR).
2. (rc 1, 2) construct: CO-REACH r1 7f519bc8 MERGE-OK (holds co-close r2); CO-CLOSE-WAITS r1 22bf5fb7 MERGE-OK, train136; co-77, co-78, rcr1-1 MINOR after.
3. (rc 2) TILING-DOORS r2 (ti-55, ti-56, rp9-1), review, train, P2-R10 (0.8 %); TILING-MINOR r1 31af60c5 MERGE-OK (rtm1-1..5 minor), merge waits for ti-55, ti-56 (rules.md 6); joins train after DOORS r2.
4. (F7, rc 11) TOUR-BASE r8 MERGE-OK 27fb2599 (PM), train136; then v9 take; rv9j-1..5 MINOR after.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
