# State

## Now (2026-11-20 07:59 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 01:45: 40 %; tokens.py 07:58 PM 7.6 %; 20 h left; 5-h limit hit 06:39-06:50, lanes idle to 07:57 (resumed); the 5-h window, not the week, binds
session branch: 79c8d2b5 (train 81); 43 greens. Train 85b (9973d0eb) red 07:3x only on matrix time cap (320 s > 315, all cells ok; container restart under it); rerun as 85c 07:56.
burn-down: audit-f r3 on 79c8d2b5: F1-F4, F6, F8 met; F5 open (flakes only); F7 by design (next wk); F8 holds with docs-f8e (t85). 1.9 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r2 red (xp-35 I5); mc r2 red (mc-47..49; M2 ruled out) |
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
| mc-fix-r1.md (RC 2: mc-47, 48, 49 MAJOR cand., P2-CHECK; Queue 5) | sonnet high | session_016iS3kPRxyZyGMAUJ4zQHkq |
| review-core-l-r1.md (F5 WD2, L16: b1d1491d) | opus high | session_016iS3kMrXHnHPAGR4AjqyAD |
| flakes-k-r1.md (F5: FLK-AG1, IN1, TR1, HUD1, doors Esc; Queue 5) | sonnet high | session_016iS3kNaWQ8q1kH49DuWMEp |
| co-r4.md (RC 2: co-30 grab rule, co-28, rv-1..3; Queue 5) | opus high | session_016iS3kPhQfF1B27WLPDCdyc |

## Queue (% week; 5 % reserve)
4. Trains: t85c running (= t85b: t84 + 11 lanes); t86: harness-l 75dfa4b9 (MERGE-OK 06:20, own train); t87: co-majors-r3 68790912, xp-24 2d1e06c6 (MERGE-OK 08:05), core-l if OK; PATH with fuzzel 1.11.1 (GATE-SILENT).
5. F work: running: FLAKE-SERIES r2 haiku 1 % (F5), CO-R4 1.5 %, XP-24 1 %, FLAKES-K 1.2 % (F5), MC-FIX 1.5 %, REVIEW CORE-L 0.5 %; after t85b: XP persona r3 (RC 2, xp-35 fix in t85b), LINT-DOWN r3 0.1 %, co-26 pane; FLAKES-L r2 1 % (F5), AUDIT-F r4 Sat 0.6 %.

## Needs owner
- Delete claude/release-rc4 (403).
- setup.sh refused in lanes (apt): default skip. rc.4 tag: kept local (owner).
