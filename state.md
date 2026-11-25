# State

## Now (2026-11-25 08:33 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 07:43 PM 2.4 % (11-25), week about 34 % vs 55 % pace: under-spending, fill 6 lanes
session branch: 6f37120b (t136 07:41: co-close r2, co-reach, co-close-waits, stray-super-minor, tour-base r8); 84 greens. TRAIN137 gate-ratchets (+PM fix e25848d6: own sed line counted as a sleep)
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | construct co-75, -76, -79 fixed t136 (recheck next persona round); xp, mc: none; tiling ti-55, ti-56 reopened r9 |
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
| train137 (harness) | PM, 6b89b56f on 6f37120b | gate running |
| co-minor-r1.md (rc 2; co-77, -78, rcr1-1; 1 %) | sonnet high, claude/co-minor-r1 6f37120b | session_016iS3kPtWQpHRWp48U6q5Xm |
| xp-minor-r2.md (rc 1; xp-10, -13, -30, rxm1-1; 1 %) | sonnet high, claude/xp-minor-r2 d2a048d5 | session_016iS3kQRG6ST7x32pFZswCM |
| mc-minor-r2.md (rc 1; rmm1-1..4, rpx1-10, -15; 1 %) | sonnet high, claude/mc-minor-r2 5cb288e2 | session_016iS3kNyhRbv8jD3qsSnufr |
| tiling-doors-r3.md (rc 2; rtd2-1 MAJOR, rtd2-4; 0.5 %) | sonnet medium, claude/tiling-doors-r3 fa676de8 | starting |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) t136 green 6f37120b (co-75, -76, -79 fixed). TRAIN137 gate-ratchets r1 e25848d6 (harness, alone). TRAIN138: xp-minor r1 d2a048d5, mc-minor r1 5cb288e2 (both MERGE-OK).
2. (rc 2) construct: CO-MINOR r1 (co-77, co-78, rcr1-1; 1 %); persona recheck of co-75/76/79 in P2-R10.
3. (rc 2) TILING-DOORS r2 fa676de8 review r2 FIX (rtd2-1 MAJOR: launch-name fix had no red first); r3 then PM review, train, P2-R10 (0.8 %); TILING-MINOR r1 31af60c5 MERGE-OK (rtm1-1..5 minor), merge waits for ti-55, ti-56 (rules.md 6); joins train after DOORS r2.
4. (F7, rc 11) TOUR-BASE r8 MERGE-OK 27fb2599 (PM), train136; then v9 take; rv9j-1..5 MINOR after.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
