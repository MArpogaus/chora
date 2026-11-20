# State

## Now (2026-11-20 09:16 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 09:13: 61 % (half other agent, session ~30 %) at 59 % time: on pace; 39 % left, 2.1 %/h shared; tokens.py 07:58 PM 7.6 %; 5-h window resets 12:00, binds; 2-4 lanes
session branch: 9973d0eb (train 85c green 08:38, e2e incl. tiling-apps-click PASS with fuzzel 1.11.1: lane reds = lane fuzzel or flake); 44 greens. Train 86 (harness-l) running 08:40.
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
| xp-36-r1.md (RC bar honest failure: Run... silent on launcher fail; xp MAJOR) | sonnet medium | starting |

## Queue (% week; 5 % reserve)
4. Trains: t86 running: harness-l 75dfa4b9; t87 next: mc-fix cb2b80f1 (r2 merge, PM review MERGE-OK 09:15), co-r4 42c9304b (contains co-majors-r3; merge-gate.sh comment conflict, PM joins both), xp-24 2d1e06c6, core-l b1d1491d, flakes-k e2508ec1; fuzzel 1.11.1 PATH.
5. F work: running: FLAKE-SERIES r2 haiku 1 % (F5), XP-36 0.5 % (xp MAJOR from persona r3 09:15: T6; T1, T4:clock reds are harness, P2R3-HARNESS); next: MC persona r3 1 % (after t87, branch claude/p2-r3-mc), LINT-DOWN r3 0.1 %, co-26 pane; FLAKES-L r2 1 % (F5), AUDIT-F r4 Sat 0.6 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
