# State

## Now (2026-11-20 10:01 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 09:13: 61 % (half other agent, session ~30 %) at 59 % time: on pace; 39 % left, 2.1 %/h shared; tokens.py 10:01 PM 8.5 %; 5-h window resets 12:00, binds; 2-4 lanes
session branch: adf71956 (train 86 harness-l green 09:27); 45 greens. Train 87 RED 09:58: layout light-1280x720 construct HUD (no 3rd terminal, empty hints, lost window); green on adf71956; bisect co-r4 alone running.
burn-down: audit-f r3 on 79c8d2b5: F1-F4, F6, F8 met; F5 open (flakes only); F7 by design (next wk); F8 holds with docs-f8e (t85). 1.9 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r3 09:15: 0 breaches, xp-35 fixed; open xp-36 MAJOR (Run silent), 2 harness reds; mc r3 after t87 |
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
| xp-36-r1.md (RC bar honest failure: Run... silent on launcher fail; xp MAJOR) | sonnet medium | session_016iS3kLTuyjo2azqRJL1sKy |

## Queue (% week; 5 % reserve)
4. Trains: t87 red (layout), bisect co-r4 first on adf71956: mc-fix cb2b80f1, xp-24 2d1e06c6, core-l b1d1491d, flakes-k e2508ec1, co-r4 42c9304b (merge-gate.sh comment joined by PM); fuzzel 1.11.1 PATH.
5. F work: running: FLAKE-SERIES r2 haiku 1 % (F5), XP-36 0.5 % (xp MAJOR from persona r3 09:15: T6; T1, T4:clock reds are harness, P2R3-HARNESS); next: MC persona r3 1 % (after t87, branch claude/p2-r3-mc), LINT-DOWN r3 0.1 %, co-26 pane; FLAKES-L r2 1 % (F5), AUDIT-F r4 Sat 0.6 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
