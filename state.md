# State

## Now (2026-11-16 08:58 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 40 % at 08:49 Mon (owner). forecast (tokens.py 08:49): 7.2 %/day, ~74 % at reset; owner 08:52: use the whole week -> target ~12 %/day (cap 12.5 % from next merge). More lanes in parallel.
models: haiku low tests, gates, trains; sonnet medium builds (high only with a reason); opus only first reviews and hard causes.
session branch: 66588880 (train 23, 08:3x UTC: media; all steps incl. doors passed). Train 24 (claude/f25 df101c14) gate running. CI green 66588880 (train 23).
rc target: Fri 11-20 (tag). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | sh-23, co-16 MAJOR open (tour v3): multilook r1; ti-1 not chora, Apps cell in CI e2e, fuzzel>=1.11 pinned (train 22) |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; F16 fixed (train 22); F17, F26 open |
| 4 | CI green | PR #106: green 5aa30b54, 4693fe07; green 8e7effad, 3c0603e8; F25 cause found (f25 r2, df101c1f): damage scissor; review, then train |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; security a (r5) and b merged; area reviews Tue |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Visual acceptance screencast | tour v3 merged (train 23); to the owner after sh-23, co-16 fixed |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| f25 r2 (rc 4) | handed back 08:15, claude/f25 df101c1f | briefs/f25-r2.md | MERGE-OK (reports/review-f25-r2.md, 2 MINOR); train 24 gate running |
| multilook r1 (rc 1) | cloud sonnet medium, claude/multilook (66f13d68) | briefs/multilook-r1.md | 4th start 08:46 (session ...NhsR4); sh-23, co-16 MAJOR; ti-20 verify |

| testwatch r1 (rc 3, 4) | cloud haiku low, session branch | briefs/testwatch-r1.md | starting 09:0x |
| review areas a r1 (rc 6) | cloud sonnet medium, read-only | briefs/review-areas-a-r1.md | kit, xp, tiling, notify |
| review areas b r1 (rc 6) | done 08:58 | briefs/review-areas-b-r1.md | 74 findings, no HIGH; sec MEDIUM rab-60, 61 |
| secfix b r1 (rc 6) | cloud sonnet medium, claude/secfix-b | briefs/secfix-b-r1.md | rab-60, rab-61; starting 09:0x |

## Queue (by day; budget 10 % a UTC day)
1. Mon: train 24 (f25), CI on PR #106; multilook r1 (sh-23, co-16); then ledger r3 (haiku). (rc 1, 4, 11)
2. Tue: persona round 4 (haiku swarm) after multilook, triage.py; area review fixes (sonnet). (rc 2, 6)
3. Wed-Thu: flakes F17, F24, F26, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3)

## Needs owner
- setup.sh fast start (drafts/setup-fast.diff: prebuilt sccache, deny, machete; never exit 1): default apply 12:00 UTC
