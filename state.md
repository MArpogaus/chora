# State

## Now (2026-11-20 19:03 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left in 13.3 h = 2.6 %/h shared; under-spending; tokens.py 18:39 PM 0.7 % since compaction, 11.4 % before; disk 12 G free: one build (train) at a time; 6 lanes
session branch: 9dc384b3; 49 greens (t90 gate-silent green 17:44); t91 gate-orphan red 18:44 on construct-hud (FLK-HUD-TAB, cell alone 3/3 green): rerun t91b running.
burn-down: audit-f r4 14:23 on d81c8293: F1-F4, F6, F8 met; F5 open (bar 5/5 at --jobs 2); F7 plan accepted (tour-v9, 9.5 %), build started. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet the stop rule; co-25, co-33 fixed (co-fix MERGE-OK); co r3 17:12 fails; co-pane r2 done, review r2; tiling r2 17:04 fails (I3, T7): handover-ti review r2 MERGE-OK (t92), then P2-R3-TILING |
| 3 | Three greens in a row | 30b, 33-40 green (9); latest 88b, 89b |
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
| review-co-pane-r2.md (RC 2: r2 done 628bb930, 15 cells PASS) | sonnet high, claude/co-pane 628bb930 | starting |
| tour-one-r1.md (F7, RC 11: design/tour-v9.md lane 1, one tour set) | sonnet medium, claude/tour-one 9dc384b3 | session_016iS3kNTVyCQRmPu3jQAyF1 18:06 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) t91b gate-orphan 437173e1 running; t92 lint-down 78ec62e1, mc-minor f7735e43, flakes-o 16da7bc7, flakes-n d7c69aac, persona-check e3763f69, co-fix bbaad12b, flakes-l cc1282db, kit-minor 939acf6a, xp-tidy 74b423bb (r2 MERGE-OK), handover-ti d1ba7e2c (r2 MERGE-OK); then --jobs 2.
5. (F5, F4, shared bar) MERGE-OK: the t91, t92 shas. Running: REVIEW-CO-PANE r2 0.5 %, handover-ti MERGE-OK into t92, TOUR-ONE r1 1.5 % (tour-v9 accepted, 9.5 % plan); then P2-R3-TILING (I5 open), P2-R4-CO. Later FLAKE-SERIES r3 haiku 1 % (adds FLK-HUD-TAB).

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
