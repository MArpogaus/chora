# State

## Now (2026-11-20 05:43 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 01:45: 40 %; tokens.py 05:12 PM 6.7 %; 22.8 h left: 60 %, 2.6 %/h; 6 lanes full
models (owner 06:56): haiku tests; sonnet routine; opus complex, planning; Fable by need
session branch: 79c8d2b5 (train 81); 43 greens. Train 84 (t83 + harness) running 05:37, head 355cdbe5.
burn-down: audit-f r3 on 79c8d2b5: F1-F4, F6, F8 met; F5 open (flakes only); F7 by design (next wk); F8 holds with docs-f8e (t85). 1.9 d.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r2 red (xp-35 I5) |
| 3 | Three greens in a row | 30b, 33-40 green (9); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 29e2cd77 13:13 (train 41, PR #106, all 10 checks) |
| 5 | Packaging smoke | arch, rpm, podman green 29e2cd77 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | train 14, 17 |
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| harness-l-r1.md (F6 gates: R15, R12, R14, H-move; Queue 5) | sonnet medium | session_016iS3kND6YHtNvLZZ3zyyMN |
| p2-r2-mc.md (RC 2 persona round 4, mc; Queue 5) | sonnet medium | session_016iS3kQWQuaukrpP1MwhJDd |
| core-l-r1.md (F5 F22, stability: WD2, L16; Queue 5) | sonnet high | starting |
| flakes-k-r1.md (F5: FLK-AG1, IN1, TR1, HUD1, doors Esc; Queue 5) | sonnet high | session_016iS3kNaWQ8q1kH49DuWMEp |
| review-xp-p2r3-r1.md (RC 2, seeded review of f89f785c; Queue 4) | sonnet high | session_016iS3kQLLfhEJgLJAbKpZ1t |
| xp-24-r1.md (RC 1 rule 6: xp-24 MAJOR, fields grid; Queue 5) | sonnet high | session_016iS3kPeHe5eFteANmxugvt |
| co-majors-r3.md (RC 2, rule 6: co-27, co-29 MAJOR, co-28; Queue 5) | sonnet high | session_016iS3kNjVuFx3uAeQzWEbHq |

## Queue (% week; 5 % reserve)
4. Trains: t84 running (t83 4 branches + debt-f4, harness-k, flakes-j). t85: co-p2-fix ab4cbb4f, stray-grab 591e98f8, black-pane 91e6a114, mc-p2-fix 17d9a798, mc-remote dac835dd, kit-tests 40b61f28, tiling-bar 3a9a426f, kit-minors 7a7e00a2, core-minors d23b03ae, docs-f8e c529242b.
5. F work: running: HARNESS-L 0.8 % (F6), CO-MAJORS r3 1.2 %, XP-24 1 %, REVIEW-XP-P2R3 0.5 %, FLAKES-K 1.2 % (F5), P2-R2-MC 1 %, CORE-L 1 % (F5); after t85: LINT-DOWN r3 0.1 %, co-26 pane; FLAKES-L r2 1 % (F5), FLAKE-SERIES r2 haiku 1 %, AUDIT-F r4 Sat 0.6 %.

## Needs owner
- Delete claude/release-rc4 (403).
- Lane auto mode refused setup.sh (apt) 23:37: allow, or all tools in env setup? Default: skip it. rc.4 tag: kept local (owner).
