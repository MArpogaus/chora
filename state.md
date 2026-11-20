# State

## Now (2026-11-20 11:58 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 09:13: 61 % (half other agent, session ~30 %) at 59 % time: on pace; 39 % left, 2.1 %/h shared; tokens.py 11:05 PM 8.8 %; 5-h window resets 12:00, binds; 2-4 lanes
session branch: adf71956; 45 greens. t87 layout red = flake FLK-LAY1; t87b hud red = flake FLK-HUD2 (voice-state-rate timeout, load 5); t87c K5 wobble race (co-r4 out, co-r5); t87d tiling-look ink@2 red = known flake (4th, load 5-6); t87e same head d81c8293 --jobs 1 from 11:34.
burn-down: audit-f r3 on 79c8d2b5: F1-F4, F6, F8 met; F5 open (flakes only); F7 by design (next wk); F8 holds with docs-f8e (t85). 1.9 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r3 09:15: 0 breaches, xp-35 fixed; xp-36 fixed 3290f1d4 (MERGE-OK, next train), 2 harness reds; mc r3 after t87 |
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
| flakes-l-r2.md (F5: 6 gate flakes) | sonnet high, claude/flakes-l | starting 12:00 |

## Queue (% week; 5 % reserve)
4. Trains: t87e (--jobs 1, flakes) in the gate from 11:34 (d81c8293 = adf71956 + mc-fix cb2b80f1, xp-24 2d1e06c6, core-l b1d1491d, flakes-k e2508ec1, xp-36 3290f1d4); next: t88 co-r5 c2b0310f (MERGE-OK 11:58, carries co-r4; 12 h ~20:38); fuzzel 1.11.1 PATH.
5. F work: running: FLAKES-L r2 sonnet 1.2 % (F5; series r2 0/5: launcher-click fuzzel 1.9.2, acts, tiling-audit; kit 91.73 %); next: MC persona r3 1 % (after t87, branch claude/p2-r3-mc), LINT-DOWN r3 0.1 %, co-26 pane, AUDIT-F r4 Sat 0.6 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
