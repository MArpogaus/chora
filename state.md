# State

## Now (2026-11-24 12:57 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 11:58 PM 4.9 % (11-22) + 5.2 % (11-23) + 3.6 % (11-24), week about 28 % vs 47 % pace
session branch: d40142e4 (t127b 12:28); 76 greens, series 2/3; t128 red (sleep-ratchet); t129 12:40 co-pull.
burn-down: audit r6 01:40: F1-F4, F6, F8 met (F4 no margin); F5 7 rows, no series; F7 open (r4 take stale, 24 acts)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: sh-38 (co-68 MERGE-OK; ti-47, ti-48, co-67 fixed d40142e4) |
| 2 | Persona r4 | xp, mc meet; co r9 not met: co-64, co-65 fixed r3; tiling r7 not met (T7 path, ti-27, ti-50): ti-47/48 fixed; TILING-CALM r2 (t128 red) |
| 3 | 3 greens, no flake | r6 0/5; r7 2/3 (sh-38); FLAKE-SERIES r8 |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-35 leak-cell train119; sh-33/sl-9 closed soak r3; sh-36 closed soak r4; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| flake-series-r8.md (F5; 1.2 %) | sonnet high, claude/flake-series-r7 6f7bcb26 | session_016iS3kPwWVUYse7EqYxr313 |
| tiling-flake-r3.md (sh-39, sh-40; 1 %) | sonnet high, claude/tiling-flake-r3 4e257c06 | session_016iS3kPxH8AyTt14UBrf3hi |
| xp-38-r2.md (rc 3) | sonnet high, claude/xp-38-r2 2235551f | session_016iS3kLvpV2kdtwhHXTnNRh |
| tour-base-r4.md (F7; rtb3-1; 0.5 %) | sonnet medium, claude/tour-base-r4 d40142e4 | session_016iS3kNJtZ94HWpr1njZs7F |
| tiling-calm-r2.md (rc 2; t128 red sleep-ratchet 782/779; 0.6 %) | sonnet medium, claude/tiling-calm-r2 e9387fb2 | session_016iS3kNyLMFdcTUGYBG8Fui |
| tour-v9-take-r1.md (F7, rc 1b; 1 %) | sonnet medium, claude/tour-v9-take d40142e4 | session_016iS3kQ7AREXGscKyEdNsv9 |
## Queue (% week; 5 % reserve)
0. (rc 8) VERIFY-MINOR r2: 12 of 14 closed (xp-14, xp-15, xp-17 fixed d40142e4); xp-34 known; xp-18 verify.
1. (F4, F5, rc 1) t128 red 12:36: sleep-ratchet 782/779 -> TILING-CALM r2; TRAIN129 co-pull be6e62ec 12:40; then tiling persona r8.
2. (rc 1, 2) CO-PULL be6e62ec MERGE-OK (co-68) in train129; then persona r11 on construct (C2 reach).
3. (rc 3, 9; F5) FLAKE-SERIES r8 (rfs7-1, rfs7-2; sh-38 leak slope); TILING-FLAKE r3 (sh-39, sh-40).
4. (F7, rc 11) TOUR-BASE r3 merged d40142e4 (rtb3-1 MINOR open); TOUR-V9 take lane.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
