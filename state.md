# State

## Now (2026-11-20 17:52 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left in 13.3 h = 2.6 %/h shared; under-spending; tokens.py 17:45 PM 11.4 % today; disk 19 G free: one build (train) at a time; 6 lanes
session branch: 9dc384b3; 49 greens (t90 gate-silent green 17:44); t91 gate-orphan running (merge-gate.sh conflict with t90 resolved, selftest ok).
burn-down: audit-f r4 14:23 on d81c8293: F1-F4, F6, F8 met; F5 open (bar 5/5 at --jobs 2); F7 next wk. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4 15:35 meets the stop rule; mc r5 16:33 meets the stop rule (mc-42, I3 PASS; 0 breaches); co-25, co-33 fixed (co-fix MERGE-OK); co r3 17:12 fails (pane drag I3, C2 checker): co-pane; tiling r2 17:04 fails (I3, T7 hand-over high): handover-ti |
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
| flakes-o-r1.md (F5: FLK-CUBE-STILL, FLK-XP-HW) | sonnet high, claude/flakes-o ecfa2029 | session_016iS3kLHNi8VhaXCo2eoCGT 15:21 |
| co-pane-r1.md (RC 2: co r3 pane drag I3, key sheet, checker) | sonnet high, claude/co-pane bbaad12b | session_016iS3kNKpN5aPruPuWBdAXM 17:12 |
| xp-tidy-r1.md (shared bar: open xp MINORs) | sonnet medium, claude/xp-tidy 22fead94 | session_016iS3kLbXcJARDTBAnvHK6j 17:23 |
| review-handover-ti-r1.md (RC 2: seeded review of 371f0fa1) | opus high, claude/handover-ti 371f0fa1 | session_016iS3kNi8Hg7DJ6BW4qVg5T 17:52 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) t91 gate-orphan 437173e1 alone (running); t92 lint-down 78ec62e1, mc-minor f7735e43, flakes-m ecfa2029, flakes-n d7c69aac, persona-check e3763f69, co-fix bbaad12b, xp-minor fca4f8b5, flakes-l cc1282db, kit-minor 939acf6a, xp-give 22fead94; then --jobs 2.
5. (F5, F4, shared bar) MERGE-OK: the t91, t92 shas. Running: FLAKES-O r1 1 %, CO-PANE r1 1.5 %, REVIEW-HANDOVER-TI opus 0.5 %, XP-TIDY r1 1.5 %; then P2-R3-TILING (I5 open), P2-R4-CO. Later FLAKE-SERIES r3 haiku 1 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
