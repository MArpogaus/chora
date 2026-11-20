# State

## Now (2026-11-20 20:04 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 14:39: 65 % at 71 % time; 35 % left in 13.3 h = 2.6 %/h shared; under-spending; tokens.py 19:48 PM 0.9 % since compaction, 11.4 % before; disk 12 G free: one build (train) at a time; 6 lanes
session branch: b0aa1c75; 50 greens; t92 red 20:02 only on lint-ratchet: t92b of 8 running; xp-tidy r3 lint fix; co-fix rides in co-pane.
burn-down: audit-f r4 14:23 on d81c8293: F1-F4, F6, F8 met; F5 open (bar 5/5 at --jobs 2); F7 plan accepted (tour-v9, 9.5 %), build started. 0.6 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet the stop rule; co-25, co-33 fixed (co-fix MERGE-OK); co r3 17:12 fails; co-pane review r2 FIX (rw-1 MAJOR: xp Super chords lost in use mode): co-pane r3; tiling r2 17:04 fails (I3, T7): handover-ti review r2 MERGE-OK (t92), then P2-R3-TILING |
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
| xp-tidy-r3.md (RC 2: t92 lint-ratchet red) | sonnet medium, claude/xp-tidy 74b423bb | starting |
| review-co-pane-r3.md (RC 2: co-pane r3 done 18417f06, rw-1 core input fix) | opus high, claude/co-pane 18417f06 | session_016iS3kQqZ5jjuzqGjnDWbbW 19:59 |
| review-tour-one-r1.md (F7, RC 11: tour-one r1 done 8b996cee, one set, 0 copies) | sonnet high, claude/tour-one 8b996cee | session_016iS3kLr7ww1z9iavGDE8qJ 19:48 |
| tour-diff-r1.md (F7: tour-v9 lane 2, v8 diff and video check) | sonnet medium, claude/tour-diff 8b996cee | session_016iS3kPo9C5pNQkcuQwr9iS 19:49 |
| guide-checks-r1.md (F7: tour-v9 lane 3) | sonnet medium, claude/guide-checks 8b996cee | session_016iS3kPbqYusAxcH6mxYMgD 19:49 |

## Queue (% week; 5 % reserve)
4. (F6, rc 3) t92b running: lint-down, mc-minor, flakes-o, flakes-n, persona-check, flakes-l, kit-minor, handover-ti; then --jobs 2; xp-tidy after r3; co-fix with co-pane (needs the lint fix too).
5. (F5, F7, shared bar) Running: REVIEW-CO-PANE r3 opus 0.6 %, REVIEW-TOUR-ONE 0.5 %, TOUR-DIFF 1.3 %, GUIDE-CHECKS 1 % (tour-v9 4-5 wait for xp-give); after t92: P2-R3-TILING (I5), FLAKE-SERIES r3 haiku 1 % (+FLK-HUD-TAB); then P2-R4-CO.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
