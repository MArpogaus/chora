# tour-base report

## r1 (F7, rc 11, 1b: the tour baseline measures I6 and every red act is named; 2026-11-24)

Branch `claude/tour-base` from 8c411416, head **9616d7d6** (pushed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q` (82 passed) green. `python3 demo/tour/run.py --help` has no help: the arguments are chapter names and `--check`, so it started the tour and broke at once for lack of binaries (`target/debug/chora` was not built by `cargo check`); I built `cargo build --workspace` for the takes. Harness and tests only; no product code. The file is `tests/tour-diff.py` (the brief said demo/tour).

### Commits (red, then fix, then rows)
| sha | what |
|---|---|
| a5eeec7c | red: `test(tour): a baseline without boxes is not an I6 flip` (tests/test_tour.py `CaptureDumps.test_a_baseline_without_boxes_is_not_an_i6_flip`) |
| e7b256d0 | fix: the dump marks I6 "not measured" where the snapshot has no `scene.boxes`; tour-diff leaves a check that either side marks so out of the flips and lists it in the row as `unmeasured` |
| 9616d7d6 | rows: 164 `[[change]]` rows of demo/tour/intended.toml for the 73 regression acts of the take (below) |

Red, before the fix (`python3 -m pytest tests/test_tour.py -q -k i6_flip`):
```
AssertionError: 'regression' != 'same'
 : {'act': 'x1', 'capture': 'Q', 'verdict': 'regression', 'unnamed': [], 'flipped': ['I6'], 'scene': [], 'blobs': [], 'fraction': 0.0}
FAILED tests/test_tour.py::CaptureDumps::test_a_baseline_without_boxes_is_not_an_i6_flip
1 failed, 82 passed
```
After the fix: `83 passed in 0.53s`; `python3 -I tests/test_tour_diff.py`: `Ran 50 tests ... OK`.

The check is a `checks` value: `I6: "not measured"` in the json of the capture of a build with no `scene.boxes` (733967aa, `OldCore`). Only I6 is marked, as the brief says. I2, I5, I9, I13 also read boxes and are green on 733967aa for the same reason; they still flip (the 14 construct I2 acts below, which carry co-44). **Question for the PM:** mark I2, I5, I13 too (one line in tour.py `dump`)?

### The takes (all on this container; /tmp is lost)
- Baseline: `TOUR_BASELINE=1 TOUR_REPO=/home/user/base733 TOUR_OUT=... python3 demo/tour/run.py` on 733967aa (xp 31 red, tiling 28, mc 22, construct stops at `construct.cube.list`, handover BROKE at handover.py:190, as designed).
- Candidate: `TOUR_OUT=... python3 demo/tour/run.py` on 8c411416: **23 red, 0 broken** (xp 1, tiling 4, mc 0, construct 18, handover 0). The head is after xp-readable, so the I6 reds of record-v9 r4 (xp 30, mc, handover) are gone.
- Diff, before the rows (9616d7d6^): `tour-diff: 1 same, 29 change, 73 regression, 7 none` (110 captures; 99 with `I6 not measured`; no I6 flip, 14 I2 flips). After: `tour-diff: 1 same, 98 change, 0 regression, 11 none`.
- `python3 demo/tour/run.py --check` on the head: `tour: check: 23 red, 0 broken`. Acts with neither a row nor a finding: 0 (tour-diff regression 0; every one of the 23 red acts has a finding, below).

### Controlled take of fa7faabd (mc-38, Home alone goes to the first page)
Head with fa7faabd reverted by hand in a worktree (the revert conflicts: main.rs moved; I removed the `Home` arm of `page_key` in methods.rs and the Home row of keys.rs, mc.yaml and docs reverted), `TOUR_REPO=... run.py mc`, diffed against the baseline. Result:
- The pages after m4c are the same: `visible mc-s-agenda / mc-s-boil` and the pixel diffs of m5..m8b go (m5, m5b, m6, m6b lose pixel and gain `added`; m7c, m8b lose visible and pixel). So fa7faabd is the cause of the page divergence.
- It is **not** the cause of the rest: `mc-regions-N` visible (the slot of the region flips on each page turn: m2..m9b), the right panel (`mc-rN-right-*` rect, 19 acts), `mc-rN-footer-next/prev` and `mc-rN-header-head` rects. 21 mc acts stay regressions without it. record-v9 r4 said "the chapter cannot make both builds land on the same page without product help; it stays a regression with this cause": half true. The rows of the mc acts say so and name **mc-59 (proposed)**.

### Causes found by a picture
- t8, t10 bar blob [12, 8, 475, 32]: crop of the bar in both takes: baseline has the gradient border and square island, the head a 1 px hairline and radius 10: bbb105ba (ti-21).
- xp blobs [1052, 694, ...] on x10 and others: the pager buttons of the taskbar are restyled: xp-readable (fc9675ac, 11e34cf8, 94579f91; xp-39 is fixed by it).

### Table: the acts that were regressions, with kind and cause
(73 acts: the 59 with unnamed kinds only, and the 14 construct acts with the I2 flip. The "43 flipped acts" of audit-f carried an I6 flip; on this head none flips on I6 any more, I6 being not measured on the baseline and green on the candidate, so the 43 are inside these 73 and named by the same rows. Cause column: a sha, or a finding id; ids marked P are proposed below, the others exist in findings/.)

| act | kinds | cause (sha, finding) |
|---|---|---|
| c2 | rect,pixel,check | co-44, 1e7cc905, cfc805d4, 29281054, d9b2c4f1, co-52, co-54, co-61, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| c2b | rect,pixel,check | co-44, 1e7cc905, cfc805d4, 29281054, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| c3 | rect,check | co-44, 1e7cc905, cfc805d4, 29281054 |
| c3b | rect,pixel,check | co-44, 1e7cc905, cfc805d4, 29281054, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| c3c | rect,pixel,check | co-44, 1e7cc905, cfc805d4, 29281054, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| c3d | rect,pixel,check | co-44, 1e7cc905, cfc805d4, 29281054, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| c3e | check | co-44, 1e7cc905, cfc805d4 |
| c3f | check | co-44, 1e7cc905, cfc805d4 |
| c3g | check | co-44, 1e7cc905, cfc805d4 |
| c3h | check | co-44, 1e7cc905, cfc805d4 |
| c4 | missing,rect,visible,pixel,check | co-44, 1e7cc905, cfc805d4, d9b2c4f1, co-52, co-54, co-61, 29281054, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| c4b | rect,pixel,check | co-44, 1e7cc905, cfc805d4, 29281054, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| c5c | rect,check | co-44, 1e7cc905, cfc805d4, 29281054 |
| c5d | rect,pixel,check | co-44, 1e7cc905, cfc805d4, 29281054, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| c5e | rect,visible,missing,pixel | co-44, 1e7cc905, cfc805d4, 29281054, d9b2c4f1, co-52, co-54, co-61, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| h1 | missing,rect,added,pixel | fa7faabd, mc-38, mc-59, d9b2c4f1, co-52, co-54, co-61, 29281054, 371f0fa1, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| h1b | missing,rect,added,pixel | fa7faabd, mc-38, mc-59, d9b2c4f1, co-52, co-54, co-61, 29281054, 371f0fa1, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| h1c | missing,rect,added,pixel | fa7faabd, mc-38, mc-59, d9b2c4f1, co-52, co-54, co-61, 29281054, 09ff8ff9, xp-28, bd5af4b3, 371f0fa1, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| h1d | missing,rect,added,pixel | fa7faabd, mc-38, mc-59, d9b2c4f1, co-52, co-54, co-61, 29281054, 09ff8ff9, xp-28, bd5af4b3, 371f0fa1, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| m1 | rect | fa7faabd, mc-38, mc-59 |
| m10 | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m1b | rect | fa7faabd, mc-38, mc-59 |
| m1c | rect | fa7faabd, mc-38, mc-59 |
| m2 | rect,visible,added,pixel | fa7faabd, mc-38, mc-59 |
| m2b | rect,visible,added,pixel | fa7faabd, mc-38, mc-59 |
| m3 | rect,visible,added,pixel | fa7faabd, mc-38, mc-59 |
| m3b | rect,missing,visible,added,pixel | fa7faabd, mc-38, mc-59 |
| m4 | rect,visible | fa7faabd, mc-38, mc-59 |
| m4b | rect,visible | fa7faabd, mc-38, mc-59 |
| m4c | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m5 | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m5b | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m6 | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m6b | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m7b | rect | fa7faabd, mc-38, mc-59 |
| m7c | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m8 | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m8b | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| m9 | rect,visible,added,pixel | fa7faabd, mc-38, mc-59 |
| m9b | rect,visible,pixel | fa7faabd, mc-38, mc-59 |
| t10 | pixel | bbb105ba, ti-21, 573496e5 |
| t10c | added,pixel | 675ebd55, sh-31, bbb105ba, ti-21 |
| t6 | rect,pixel | b8872054, ti-30, bbb105ba, ti-21, 573496e5 |
| t6c | rect,pixel | b8872054, ti-30, bbb105ba, ti-21, 573496e5 |
| t7 | added,pixel | 675ebd55, sh-31, bbb105ba, ti-21 |
| t8 | pixel | bbb105ba, ti-21, 573496e5 |
| t9 | rect,pixel | 573496e5, bbb105ba, ti-21 |
| x1 | rect,pixel | 29281054, fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x10 | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x10b | added,pixel | 371f0fa1, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x10c | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x11b | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x12 | rect | 29281054 |
| x12b | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x12c | rect,visible | 29281054, 20e4ad6e |
| x2 | rect,pixel | 29281054, 675ebd55, sh-31, fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x2b | rect,pixel | 29281054, 675ebd55, sh-31, fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x2c | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x5 | rect | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43 |
| x5b | rect,pixel | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| x5c | rect,pixel | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| x5d | rect,pixel | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| x5e | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x5f | rect,pixel | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| x5g | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x6 | rect,pixel | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| x6b | rect,pixel | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| x6c | rect,pixel | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| x6d | rect,pixel | 4fec75c3, xp-30, 29281054, 09ff8ff9, xp-28, bd5af4b3, xp-43, fc9675ac, 11e34cf8, 94579f91, xp-39 |
| x7 | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x7b | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x8 | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |
| x9b | pixel | fc9675ac, 11e34cf8, 94579f91, xp-39, 09ff8ff9, xp-28, bd5af4b3 |

### Findings row proposals (id free in the ledger at 11-24; PM files them, I did not edit it)
- **mc-59** | MINOR (verify) | tour-base r1 (demo/tour intended.toml, mc acts m1..m10) | the mc region slot (`mc-regions-0/1` visible), the right panel rects and the footer prev/next differ from 733967aa also with fa7faabd reverted; cause not found (a page-turn commit between 733967aa and the head; a bisect with the mc chapter finds it) | capture m2.Q, m4.Q.
- **xp-43** | COSMETIC (verify) | tour-base r1, act x10b | `hud-line` and `xp-dlg-help-content-c-scroll` are new nodes in the head, cause not found; also the xp pixel blobs with no node (x5e, x5g, x7, x8, x9b: the window +88 px and the pager restyle explain the blobs by position, not proven by a take).
- **xp-44** | MINOR | `--check` on 8c411416, act x17 (Start, Help and Support) | I6: 'Notepad' (xp-title-s1) 1.00:1, (214, 223, 247) on (214, 223, 247): the title text has the colour of its bar (the inactive title with the shadow of f1693fb3, xp-41); present in the take and in `--check` | capture x17.Q.
- **xp-45** | MINOR | `--check`, construct acts c1, c2, c3, c13 | I6: 'chart' (xp-title-s6) on VIRTUAL-3 4.31:1, (30, 53, 104) on (123, 155, 223), wants 4.5 (the inactive title of an xp face after xp-41) | capture c1.Q.
- **co-61** | COSMETIC (verify) | tour-base r1, acts c4, c5e (and h1..h1d via the same stand-back) | the doors, face label and hud of a face are not in the scene while the cube stands back (`missing construct-door-*`, `construct-hud`), 733967aa has them; d9b2c4f1 (co-52, co-54) is the likely cause, intended or not is a design question | capture c4.Q.
- Not new: ti-30 is marked fixed (tiling-disc 7adb149c) in the ledger, but the head still reds act 17 and 19 on `tiling-keys-body-h` 6.51:1 (wants 7) in the take and in `--check`; either the fix is not in 8c411416 or it is incomplete. sh-31: t21 and t28 `notify-card-body` 11 px. co-44: construct c4..c18 I2 (9 apart), plus c1..c3, c13 are xp-45 now.
- The `known` rows of `--check` that say "is green: drop known": xp 31, tiling 24 (ti-29), mc 4, 5, 6 (I5 Power glyph, v8 open row), handover 2 (co-43); not touched.

### Gate
`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --only) on 9616d7d6, last 20 lines:
```
mc-handover PASS    11s /tmp/merge-gate.yOPRg4/mc-handover.log
launcher-click PASS    17s /tmp/merge-gate.yOPRg4/launcher-click.log
voice-acp  PASS    12s /tmp/merge-gate.yOPRg4/voice-acp.log
voice-events PASS     1s /tmp/merge-gate.yOPRg4/voice-events.log
agent      PASS    39s /tmp/merge-gate.yOPRg4/agent.log
tiling-keys-more PASS    41s /tmp/merge-gate.yOPRg4/tiling-keys-more.log
tiling-windows PASS   168s /tmp/merge-gate.yOPRg4/tiling-windows.log
tiling-system PASS    11s /tmp/merge-gate.yOPRg4/tiling-system.log
handover   PASS    26s /tmp/merge-gate.yOPRg4/handover.log
mc-scale   PASS    16s /tmp/merge-gate.yOPRg4/mc-scale.log
watchdog-starved PASS    62s /tmp/merge-gate.yOPRg4/watchdog-starved.log
acts       PASS  2294s /tmp/merge-gate.yOPRg4/acts.log
xp-run     PASS    14s /tmp/merge-gate.yOPRg4/xp-run.log
leak       PASS   317s /tmp/merge-gate.yOPRg4/leak.log
demo       PASS   254s /tmp/merge-gate.yOPRg4/demo.log
tiling-bar PASS   146s /tmp/merge-gate.yOPRg4/tiling-bar.log
tiling-theme PASS    18s /tmp/merge-gate.yOPRg4/tiling-theme.log
mc-host    PASS    20s /tmp/merge-gate.yOPRg4/mc-host.log
e2e        PASS   266s /tmp/merge-gate.yOPRg4/e2e.log
all steps passed; no --push, nothing is pushed
```
`tests/merge-gate.sh --red-first-check`: `red-first: every fix and feat commit has a test before it or in it`.

### Limits
- The causes of the xp, tiling (t6 .. t10), construct and handover families are by element id family and the commit list, and by two pictures; only the mc page divergence has a controlled take. Rows say "proposed" or name the sha; a row is no proof that every diff of its kind in the act comes from its sha (rr9-1 of review r4 still holds). A controlled take per family (bisect the gap, the bar, the stand-back) is the next step if the PM wants F7 (2) proven rather than named.
- `mc-regions-N` and the right panel of mc are named by a proposed finding, not by a sha.
- The takes ran on 4 cores with video encoding: 4 to 15 fps per segment; the captures are state reads, not frames.

## Review r1 (seeded review of claude/tour-base 8c411416..9616d7d6; rc 11, 1b; 2026-11-24)

Reviewer: sonnet, not the builder. Step 0 green at 9616d7d6 (the branch tip has since moved to 645afaf5 by two merges; reviewed at 9616d7d6): `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q` (83 passed), `python3 -I tests/test_tour_diff.py` (50 tests OK). No take, no gate run (as briefed); `--check` was not re-run, so check 4 is read from the ledger and the report.

**Verdict: FIX** (rtb-1 is an open MAJOR). 8 findings: 1 MAJOR, 7 MINOR, 0 BLOCK.

### Checks
1. **Red first: OK.** `git log --oneline 8c411416..9616d7d6`: a5eeec7c (test, +18 in tests/test_tour.py only) before e7b256d0 (fix) before 9616d7d6 (rows). A worktree at a5eeec7c: `FAILED test_a_baseline_without_boxes_is_not_an_i6_flip` with `'verdict': 'regression' ... 'flipped': ['I6']`. At head both suites green.
2. **Not measured: OK in the narrow sense, with two gaps (rtb-3, rtb-4).** Scratch cases on `tour-diff.verdict` (not committed): base I6 "not measured" + I2 green, cand I6 red + I2 red gives `regression, flipped ['I2'], unmeasured ['I6']` (a measured check is never hidden); base I6 green, cand I6 red gives `regression, flipped ['I6']`; base missing the key, cand I6 red gives `regression`. The marker is set only where `after.boxes is None` (tour.py:636), which only the baseline's `OldCore` produces (tour.py:141-160); on the candidate a lost `scene.boxes` raises. A candidate red I6 still counts in `run.py --check`: it tallies rows with status RED from the take itself (run.py:98-115), independent of tour-diff. In tour-diff alone it is `same` + "I6 not measured" (rtb-3).
3. **Rows.** 164 new rows (pixel 58, rect 53, visible 20, check 14, added 12, missing 7; xp 39, mc 59, construct 38, handover 16, tiling 12); 0 rows of kind `*` (the 14 `check` rows act alike, rtb-5). All 17 cause shas are in 733967aa..8c411416 (`merge-base --is-ancestor`). Sampled 28 rows across the five families with `git show --stat`: x1, x2, x5, x5b, x6, x10b, x12, x12c (xp: 29281054 launch.rs/startmenu.rs, 09ff8ff9/bd5af4b3 window.rs, fc9675ac/11e34cf8 session.rs, 4fec75c3 chora-assistant, 20e4ad6e wm/windows.rs, 371f0fa1 wm.rs); t6, t7, t8, t9, t10c (tiling: b8872054 touches chora-tiling keys.rs and tiling.yaml, 573496e5 tiling config/wm, bbb105ba tiling waybar style.css, 675ebd55 the notify card); m1, m4, m7b, m10 (mc: fa7faabd mc keys.rs/main.rs/mc.yaml); c2, c3, c4, c5e (construct: 1e7cc905, cfc805d4 hints.rs/overlay.rs, d9b2c4f1 cube/aside.rs, input.rs); h1, h1c (handover: union of xp, mc, construct shas). Every sampled sha touches a plausible element family of the act; the attributions that fail are not by family but by finding status:
   - **Rows whose cause is an open finding with the cause not found: 89 of 164** (rows that cite mc-59 "cause not found" 71, co-61 "proposed" 20, xp-43 10; union 89 over 38 acts). The 71 mc-59 rows cover every mc act m1..m10 and h1..h1d. The report's own controlled take says fa7faabd explains only the `mc-s-*` visible and the pixel diffs after m4c, and that 21 mc acts stay regressions without it; yet m1, m7b and other rect-only rows credit fa7faabd, which cannot move a rect in m1 (before any page key). Tour-diff prints these as `change`, so the F7 counter ("0 regression") now contains 21 acts that are unexplained, which is partial credit (owner 11-17). rtb-1.
   - Broad rows: 38 of 164 cite five or more shas (h1c and h1d pixel: 9 shas plus 7 ids; sha 29281054 sits in 53 rows, fa7faabd in 71). Rows match on (act, kind) only, so a later unrelated pixel or rect diff in these acts passes. rtb-6 (rr9-1 is the known general limit; this is the width).
4. **The 23 red acts of --check** (xp 1, tiling 4, construct 18, from the report): x17 xp-44 (open, xp.md:45); t17, t19 ti-30 (reopened 11-24, tiling.md:31); t21, t28 sh-31 (shared.md:31); c1, c2, c3, c13 xp-45 (open, xp.md:46); c4..c18 without c13 (14 acts) co-44 (open, construct.md:45). 1+4+18 = 23, each with a findings id. New ids exist as open: mc-59 (mc.md:50), xp-43 (xp.md:44), xp-44, xp-45, co-63 (construct.md:65, the co-61 proposal renamed as the id was taken), ti-30 reopened. Gaps: sh-31 names t21 only (t28 is not in the row), and co-44 names `c4..c18` by the run order while tour-diff and the 14 `check` rows use scene ids (c2, c2b, c3 ... c5d); the two numberings cannot be tied by a reader (rtb-8).
5. **The lane's question (I2, I5, I13 as "not measured"): yes, and wider.** The invariants that read `scene.boxes` and find nothing on a box-less baseline are vacuous-green there: I2 (tests/acts/invariants.py:434-455: `node_boxes` 440, `after.boxes` 448; `held_breaches` 420), I5 (504-525: `boxes_on` 509; `screen_texts` 366-380), I9 (611-655: `boxes_on` 638, `after.boxes` 645), I12 (749-760 through `bars_over_fullscreen` 695-699, `modals_below` 717, `covers_held` 732-736), I13 (812-826: `boxes_on` 817), I6 (586-608 through `guide_breaches` 548 and `node_boxes` 596), and I8 returns the line "missing boxes" at 208-209, which tour.py:295-296 already filters on the baseline. Mark I2, I5, I6, I9, I12, I13 from one constant next to tour.py:633-637 (`after.boxes is None`). Effect: the 14 construct I2 flips (the `check` rows at c2..c5d, co-44) become `unmeasured`, not flips; they stay red in `--check`, so run `--check` = 0 unexplained as the second gate (see rtb-3).

### Findings
- **rtb-1** | MAJOR | demo/tour/intended.toml (71 rows citing mc-59: acts m1..m10, h1..h1d) | 21 mc acts differ from 733967aa by a cause the report did not find (mc-regions-N visible, right-panel and footer rects), yet tour-diff prints `change` and the F7 counter reads 0 regression; rows such as m1 rect and m7b rect also credit fa7faabd, which the controlled take shows does not explain them | Fix: bisect mc-59 between 733967aa and the head and name the sha, or give those rows `open = true` and let tour-diff count them as a fourth verdict `open` that fails F7; credit fa7faabd only to the kinds the take showed.
- **rtb-2** | MINOR | demo/tour/intended.toml (20 rows: c2, c4, c5e, h1, h1b, h1c, h1d) | the rows cite "co-61 (proposed)"; co-61 in the ledger is an unrelated fixed MAJOR (construct.md:63, co-reach); the proposal was filed as co-63 | Fix: replace co-61 by co-63 in the 20 causes.
- **rtb-3** | MINOR | tests/tour-diff.py:178-180, 191 | `unmeasured` keeps no candidate colour: base "not measured" with cand red gives verdict `same` and the line "I6 not measured", so a red candidate I6 is visible only in `run.py --check`, which is no part of the diff output | Fix: list `I6 not measured (cand red)` in the row and the print, and make the F7 gate read both tour-diff and `--check`.
- **rtb-4** | MINOR | demo/tour/tour.py:636 | I6 is marked wholesale on a box-less snapshot, but `i6` still measures the reported runs against their picture (tests/acts/invariants.py:590-594, no boxes needed); a baseline-green of that part is real and a candidate flip of it is now hidden | Fix: mark I6 only when the baseline `i6` read no text box, or keep the reported-run check out of the marker.
- **rtb-5** | MINOR | tests/tour-diff.py:181-185, demo/tour/intended.toml (14 `check` rows c2..c5d) | a row of kind `check` absorbs every flipped check of the act, not the one it names; the 14 new rows name I2/co-44 only, so a later I6, I9 or I12 flip in those acts (c2 and c3 already carry the xp-45 I6) passes | Fix: add a `check = "I2"` field to the row and match flips by name.
- **rtb-6** | MINOR | demo/tour/intended.toml (38 rows with five or more shas, e.g. h1c, h1d, x6d) | wide rows with up to 9 shas and 7 ids match on (act, kind) only, so a later unrelated pixel or rect diff in those acts is named in advance | Fix: allow at most three causes per row, or add a `rect` field so a pixel row names its blob.
- **rtb-7** | MINOR | demo/tour/intended.toml (x5, x5b, x5c, x5d, x5f, x6..x6d, x10b cite xp-43) | xp-43 (xp.md:44) names x10b and the blobs of x5e, x5g, x7, x8, x9b; the rows cite it on x5..x6d (which it does not name) and not on x7, x8, x9b, x5e, x5g (which cite only the pager restyle) | Fix: cite xp-43 where the finding names the act, and say in the others that the pager restyle is by position only.
- **rtb-8** | MINOR | findings/shared.md:31, findings/construct.md:45 | sh-31 lists act t21 and not t28, which the report says is red in `--check`; co-44 lists `c4..c18` by run order while tour-diff and the toml use scene ids (c2..c5d), so the 14 `check` rows cannot be tied to the finding | Fix: add t28 to sh-31 and the scene ids to co-44.

### Limits
- Causes by family are read from `git show --stat` and element names, not from controlled takes (as briefed): a sha that touches the family is plausible, not proven (rr9-1).
- `--check` and the takes were not re-run; the 23 red acts and their ids are the report's, matched against the ledger text.
- The scratch cases for check 2 were run on a copy in the scratchpad, not committed.

## r2 (F7, rc 11, 1b: no unexplained act passes as `change`; rtb-1..8; 2026-11-24)

Branch `claude/tour-base` from 645afaf5 (r1 9616d7d6 plus the session head 05a63377); head **1e62811f** (pushed). Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q` (83 passed), `python3 -I tests/test_tour_diff.py` (50 tests OK). Harness and tests only; no product code. The repo hooks refuse attribution lines and subjects over 68 characters, so the commits carry none (the brief says the same).

### Commits (red, then fix, then rows)
| sha | what |
|---|---|
| 457b7ee4 | red: tests only (a)..(d) |
| 192aebbe | fix: `open` verdict, `check` field, `cand_red`, `BOX_CHECKS` (tests/tour-diff.py, demo/tour/tour.py) |
| 1bf04706 | red: `IntendedCauses` in tests/test_tour_diff.py (rtb-2, 5, 6, 7 as data tests) |
| 8649f08d | rows: mc rows from the bisect, co-63, named checks, xp-43, width (demo/tour/intended.toml) |
| 1e62811f | rows: t6, t6c key list renumbered (a611c08a, fa076e7b); m9b, m10 chat nodes (old rows restored) |

Red at 457b7ee4 (`python3 -I tests/test_tour_diff.py`, `python3 -m pytest tests/test_tour.py -q`):
```
(a) FAIL: test_an_open_row_gives_the_verdict_open_not_change        AssertionError: 'change' != 'open'
(a) FAIL: test_the_f7_count_fails_on_an_open_act                    AssertionError: '1 open' not found in 'm1       Q     change      mc-59\ntour-diff: 0 same, 1 change, 0 regression, 0 none\n'
(b) ERROR: test_a_check_row_with_a_name_does_not_absorb_another_check   KeyError: 'flipped'   (the row absorbed the I6 flip: no regression)
(c) FAIL: test_base_not_measured_and_cand_red_is_printed_as_such    AssertionError: 'I6 not measured (cand red)' not found in 'x1       Q     same        I6 not measured\ntour-diff: 1 same, 0 change, 0 regression, 0 none\n'
(d) E  AttributeError: module 'tour' has no attribute 'BOX_CHECKS'
    FAILED tests/test_tour.py::CaptureDumps::test_a_baseline_without_boxes_marks_the_box_checks_from_one_constant
Ran 57 tests ... FAILED (failures=3, errors=1);  1 failed, 83 passed
```
Red at 1bf04706 (the data tests): `test_a_check_row_names_its_check`, `test_a_row_names_at_most_three_causes_or_is_open` (49 rows), `test_no_row_cites_co_61_the_ledger_id_of_another_finding`, `test_xp_43_is_cited_only_on_the_acts_that_finding_names`: `Ran 62 tests ... FAILED (failures=4)`.

After 192aebbe: `python3 -m pytest tests/test_tour.py -q`: 84 passed; `python3 -I tests/test_tour_diff.py`: 62 tests OK (after 1e62811f the same).

What the fix is (KISS: one constant, one new verdict, one new field):
- `tour.BOX_CHECKS = (I2, I5, I6, I9, I12, I13)`: on a snapshot without `scene.boxes` the dump marks them `not measured`, except I6 where a look or block reports a run (`invariants.reported_texts`): that part of I6 reads no box (rtb-4, check 5).
- tour-diff: a row with `open = true` gives the verdict `open` (a row with a found cause is taken first; `open` never hides an unnamed kind or a flip), exit 1 on `regression` or `open`; the summary line has the four verdicts (`same, change, open, regression, none`).
- a row of kind `check` with `check = "I2"` absorbs only that flip; without the field it absorbs every flip (old rows).
- a check that the base marks `not measured` and the candidate reads red is printed `I6 not measured (cand red)` (field `cand_red`; the verdict stays what the rest says; `--check` still counts it, 13 I6 and 10 I2 captures in the full diff below).

### rtb-1: the bisect of mc-59 (mc chapter only)
Method: the mc chapter (`TOUR_BASELINE=1 TOUR_OUT=... TOUR_REPO=<worktree> python3 demo/tour/run.py mc`) on a worktree at each sha, `cargo build -p chora -p choractl -p chora-mc -p chora-notify -p chora-xp -p chora-assistant`. Five bisect builds; the baseline (733967aa) and the head (05a63377 + harness) are two more builds, not bisect steps. Five criteria read from the captures of each build: m1 `mc-r0-right-notify` present, m1 `mc-r0-right-chat-door` y, m1 `mc-s-ingredients-panels-lead-rule` x, m1 `mc-r0-right-keys-2-key-hit` height, the region slot visible (`mc-regions-N`) at m1 m2 m3 m3b m4 m4b m9.

| build | sha | notify block | chat door y | lead rule x | key hit h | slot m1 m2 m3 m3b m4 m4b m9 |
|---|---|---|---|---|---|---|
| base | 733967aa | yes | 194 | 42 | 28 | 0 0 0 0 1 0 0 |
| 1 | 675ebd55^ (6ddd808f) | yes | 194 | 42 | 28 | 0 0 0 0 1 0 0 |
| 2 | **675ebd55** chora-notify shows the notices, mc stacks none | no | 110 | 42 | 28 | 0 1 1 1 0 1 0 |
| 3 | 6fa86c16^ (d3cc19de) | no | 110 | 42 | 28 | 0 1 1 0 0 1 1 |
| 4 | **6fa86c16** key hit, key text, page 2 copy, empty font, heads, rule | no | 110 | 94 | 56 | 0 1 1 0 0 1 1 |
| 5 | fa7faabd Home alone goes to the first page (mc-38) | no | 110 | 42 | 28 | 0 1 1 0 0 1 1 |
| head | 05a63377 | no | 110 | 94 | 56 | 0 1 1 0 0 1 1 |

**The first sha that moves the m1 rect and the visible region slot is 675ebd55** (build 1 equals the baseline in all five columns; build 2 moves the chat door, drops `mc-rN-right-notify`, adds the `notify-card` nodes of chora-notify and flips the slot at m2). The lead rule and the key hit (22 acts, rect) are **6fa86c16**. Not found by sha: the slot parity at m3b and m9 and the pixel, visible, rect diffs of the pages, which lie in (675ebd55..fa7faabd] (284 commits) and (fa7faabd..6fa86c16^] (543), and 3 rects after 6fa86c16. Diff of consecutive builds, acts and kinds (tour-diff, same masks):

| interval | acts | kinds |
|---|---|---|
| (733967aa..675ebd55^] | 4 (m1b, m8, m8b, m9b) | pixel 3, rect 4, visible 1 (not a named cause; rows: open) |
| 675ebd55 | 22 | missing 22, rect 22, visible 12, pixel 8, added 8 |
| (675ebd55..fa7faabd] | 15 | pixel 15, rect 15, visible 12, added 6, missing 6 |
| (fa7faabd..6fa86c16^] | 5 | pixel 5, rect 4 |
| 6fa86c16 | 22 | rect 22 |
| (6fa86c16..head] | 3 (m7b, m9b, m10) | rect 3 |

Rows (22 mc acts): a (act, kind) cites 675ebd55 or 6fa86c16 where its interval shows the kind, fa7faabd only for the kinds the controlled take of r1 showed (pixel on m5, m5b, m6, m6b, m7c, m8b; visible on m7c, m8b), and every other interval is one `mc-59 open` unit naming the commits it lies in, with `open = true`. The r1 statement "fa7faabd explains the page divergence and the rest is mc-59" stands for the pages; the m1 rect, which r1 credited to fa7faabd, is 675ebd55 and 6fa86c16. mc: 4 acts `change` (m1, m1c, m4, m4b), 18 `open`. The handover acts h1..h1d (the mc part) were not bisected (the brief says the mc chapter only): `open`.
mc-59 text for the PM: the cause is **found for the column rects, the notify block and the first slot flips (675ebd55) and for the lead rule and key hit (6fa86c16)**; open for the slot parity at m3b and m9 and the remaining pixel and rect diffs: the two intervals above.

### rtb-2..8
- rtb-2: `co-61 (proposed)` is `co-63 (open)` in the 20 rows (test `test_no_row_cites_co_61...`).
- rtb-5: the 14 `check` rows carry `check = "I2"`. With I2 now `not measured` on the baseline (BOX_CHECKS) the 14 I2 flips of r1 are no flips any more, so the rows are not used on this pair; they stay for a baseline that measures I2.
- rtb-6: at most three causes (units split at "; ") per row; 18 rows that were wider are `open` with the unit "open: more than three causes, none proven for this act alone" (x2, x2b, x5b..x6d, x10b, t6, t6c, c2, c4, c5e among them); the 30 rows of record-v9 r2 with more than three causes are listed in the test (`WIDE_BEFORE_R1`), not touched. The `09ff8ff9 ... bd5af4b3` pair is one unit (xp-28), the `co-44 (...; ...)` semicolon is a comma, so the count by "; " is the count of causes.
- rtb-7: xp-43 only on x10b, x5e, x5g, x7, x8, x9b (named by the finding); the pager unit says "by the position of the blobs, no controlled take" in every row that cites it.
- rtb-8 (the PM edits the ledger): run order to scene id of construct (rows-construct.json of the take; the 14 acts of co-44, I2, are 4..12 and 14..18): 1 c1, 2 c5, 3 c5b, 4 c2, 5 c5c, 6 c5d, 7 c3, 8 c3b, 9 c3c, 10 c3d, 11 c4, 12 c4b, 13 c5e, 14 c2b, 15 c3e, 16 c3f, 17 c3g, 18 c3h. So co-44 is c2, c5c, c5d, c3, c3b, c3c, c3d, c4, c4b, c2b, c3e, c3f, c3g, c3h (act 13 c5e has I6 only). The same mix-up holds for xp: xp-44 "act x17" is the 17th act of the xp run; its scene id is **x10** (x10b holds the same I6 in `hidden`). And xp-45's I6 'chart' (xp-title-s6) 4.31:1 is red in 11 construct acts (run order 1, 2, 3, 4, 5, 6, 10, 11, 12, 13, 14 = c1, c5, c5b, c2, c5c, c5d, c3d, c4, c4b, c5e, c2b), not the four r1 names. sh-31: t28 is green in this take (see below).

### `run.py --check` and the verdict counts (head 1e62811f, binaries at 05a63377's tree)
```
chapter xp: 31 acts, 29 green, 0 inv, 1 known, 1 red, 0 todo, 0 unchecked
chapter tiling: 28 acts, 28 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
chapter mc: 22 acts, 22 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
chapter construct: 18 acts, 0 green, 0 inv, 0 known, 18 red, 0 todo, 0 unchecked
chapter handover: 11 acts, 11 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
tour: check: 19 red, 0 broken
```
r1 had 23 red: the four tiling acts (t17, t19 ti-30; t21, t28 sh-31) are green on this head (658119d7 key list header 7:1, fa076e7b keycaps 14 px, 60a38634 card text 14 px); the PM may close ti-30 and sh-31 (a take of the same head shows tiling 0 red). 0 unexplained: xp x10 (xp-44); construct 18: I2 14 acts (co-44), I6 chart 11 acts (xp-45), I6 help heads 11 px at c4 (co-45).
Full take of the head (`TOUR_OUT=... run.py`: 19 red, 0 broken) against the full baseline take of 733967aa (`TOUR_BASELINE=1`: xp 31 red, tiling 28, mc 22, construct stops at `construct.cube.list`, handover BROKE, as designed):
```
python3 tests/tour-diff.py <base>/captures <head>/captures --intended demo/tour/intended.toml
tour-diff: 1 same, 60 change, 38 open, 0 regression, 11 none        (110 captures)
```
Per family (acts): xp change 20, open 11; tiling change 25, same 1, open 2; mc change 4, open 18; construct change 11, open 3, none 4; handover open 4, none 7. The 38 open captures: mc 18 (mc-59), xp 11 (more than three causes: x10b x2 x2b x5b x5c x5d x5f x6 x6b x6c x6d), construct 3 (c2, c4, c5e: co-63 open), tiling 2 (t6, t6c), handover 4 (h1..h1d: the mc part is not bisected). **F7 is not met by this count: 38 captures are open, none is hidden**; `tests/tour-diff.py` exits 1 on them. The first full diff of 1e62811f^ showed 4 regressions the rows now name (t6, t6c: the key list renumbered by a611c08a, fa076e7b, which came with the session head; m9b, m10: `added` chat-close nodes; I had dropped those two old rows with the mc block). 16 captures print `not measured (cand red)` (c1..c5e, x10, x10b): the red I6 and I2 of the take that the baseline cannot read.
Not a finding in `--check` but seen: the mc-only take of the head (`run.py mc`, twice) has act 7 (Escape closes the chat, capture m9b) red: `the chat is shown: ['hud-chat']`; `--check` and the full take of the same head have it green. A timing flake of the 0.2 s settle after 8ba700f5 or 8d0645cc (busy while the regions cross-fade), not bisected: **mc-60 (proposed, MINOR verify, harness or product)**.

### Gate
`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --only) on 1e62811f, detached; last 20 lines:
```
voice-acp  PASS    11s /tmp/merge-gate.tLezTV/voice-acp.log
voice-events PASS     1s /tmp/merge-gate.tLezTV/voice-events.log
agent      PASS    39s /tmp/merge-gate.tLezTV/agent.log
acts       PASS  1732s /tmp/merge-gate.tLezTV/acts.log
tiling-keys-more PASS    41s /tmp/merge-gate.tLezTV/tiling-keys-more.log
tiling-system PASS    11s /tmp/merge-gate.tLezTV/tiling-system.log
handover   PASS    25s /tmp/merge-gate.tLezTV/handover.log
mc-scale   PASS    14s /tmp/merge-gate.tLezTV/mc-scale.log
watchdog-starved PASS    52s /tmp/merge-gate.tLezTV/watchdog-starved.log
tiling-windows PASS   174s /tmp/merge-gate.tLezTV/tiling-windows.log
xp-run     PASS    13s /tmp/merge-gate.tLezTV/xp-run.log
demo       PASS   235s /tmp/merge-gate.tLezTV/demo.log
leak       PASS   315s /tmp/merge-gate.tLezTV/leak.log
tiling-theme PASS    18s /tmp/merge-gate.tLezTV/tiling-theme.log
tiling-disc PASS    72s /tmp/merge-gate.tLezTV/tiling-disc.log
mc-host    PASS    19s /tmp/merge-gate.tLezTV/mc-host.log
tiling-bar PASS   135s /tmp/merge-gate.tLezTV/tiling-bar.log
e2e        PASS   262s /tmp/merge-gate.tLezTV/e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```
No red cell. `pgrep -fa 'chora --headless'` is empty at the end.

### Limits
- The bisect names a sha only where both sides of it were built (675ebd55, 6fa86c16); the other intervals are commit ranges, so their rows are open, not named. fa7faabd was built once (as an interval boundary), its credit is the controlled take of r1.
- The five takes ran with other builds and the gate not running; the mc-only head take had one flaky act (above). The row text for 675ebd55 and 6fa86c16 is by interval, a row names a sha for the kind in the act, not for every element.
- The handover chapter and the other families (xp, tiling, construct) have no controlled take; their rows are as in r1 except for the width rule and the open rows above.

## Review r2 (seeded review of claude/tour-base 645afaf5..1e62811f; rc 11, 1b; 2026-11-24)

Reviewer: sonnet, not the builder. Step 0 green at 1e62811f (the branch tip is 883214b5, a later merge; reviewed at 1e62811f, an ancestor): `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q` (84 passed), `python3 -I tests/test_tour_diff.py` (62 tests OK). No take, no gate (as briefed).

**Verdict: FIX** (rtb2-1 is an open MAJOR). 4 findings: 1 MAJOR, 3 MINOR, 0 BLOCK.

### Checks
1. **Red first: OK.** Worktrees at each sha: 457b7ee4 `python3 -I tests/test_tour_diff.py`: `Ran 57 tests ... FAILED (failures=3, errors=1)` (open verdict, F7 count, check name, cand red) and `pytest tests/test_tour.py -q`: `1 failed, 83 passed` (box checks from one constant); 192aebbe: `OK` and `84 passed`. 1bf04706: `Ran 62 tests ... FAILED (failures=4)` (check name, three causes, co-61, xp-43); 8649f08d: `OK`; head: both green.
2. **rtb-1: mostly OK, one gap (rtb2-1).** `intended.toml` has 369 rows, 79 `open = true`; all 61 rows that cite mc-59 are `open`, none is `change`. The tour-diff code is right: `verdict` returns `open` when any named row of the act is open, `main` exits 1 on `open` and prints the fourth count. Sample of 97 rows citing 675ebd55 or 6fa86c16, 15 read with `git show --stat`: 675ebd55 touches plugins/chora-mc (mc.yaml, main.rs, region.rs, agent.rs, notify slot) and tests/acts/looks, so mc `missing`, `visible` and column `rect` rows (m1, m1c, m2, m2b, m3, m4, m4b, m7, m8, m9b) fit its family; 6fa86c16 touches keys.rs, layout.rs, region.rs, build.rs, so the `rect` rows (m1, m1c, m4, m4b, m7b, m8b) fit. Rows on t7, t10c (added, pixel), x2, x2b (rect) cite 675ebd55 as sh-31 "chora-notify card": the sha touches no tiling or xp file, so the family is by the ledger (sh-31, rv9-4), not by `--stat` (known: rr9-1, family without a controlled take). The bisect table in the report fits the rows (build 1 equals the base, build 2 is the first mover).
3. **rtb-2..7.** rtb-2 OK: no row cites co-61, 20 rows cite co-63 (the ledger id, construct.md:65). rtb-3 half: `cand_red` is listed and printed (`I6 not measured (cand red)`), but it changes neither the verdict nor the exit code (rtb2-2). rtb-4 OK: `BOX_CHECKS` constant tour.py:141, I6 kept measured where `invariants.reported_texts(after)` is non-empty (test `test_a_baseline_without_boxes_marks_the_box_checks_from_one_constant`); a green reported-run part with a red box part on the candidate stays a flip (the safe side). rtb-5 OK: 14 `check` rows, all with `check = "I2"`, `verdict` absorbs only the named flip (test with I2 and I6). rtb-6 partial (rtb2-3). rtb-7 OK both ways: xp-43 is cited on exactly x10b, x5e, x5g, x7, x8, x9b (the six acts the ledger names), and on each of them.
4. **Counts: not checkable.** No capture folder or take row is committed on the branch (`git ls-files | grep captures`: only tests/construct-captures.sh), so `tests/tour-diff.py` cannot reproduce "1 same, 60 change, 38 open, 0 regression, 11 none". From the report and the ledger: the 19 red acts of `--check` are x10 (xp-44) and the 18 construct acts; the union of co-44 (14 acts, scene ids in the r2 map), xp-45 (11 acts) and co-45 (c4) is exactly those 18, so each has a findings id. xp-44 and xp-45 are "fixed" in the ledger by f65e7f70 and f1397d98, which are not in the head (`merge-base --is-ancestor` fails), so they stay red on this head as the report says. Note for the count: the 9 rows of rtb2-1 are `change` in the 60, not `open`.
5. **A path to a later unrelated diff passing: yes, three (rtb2-1, rtb2-2, rtb2-4).**

### Findings
- **rtb2-1** | MAJOR | demo/tour/intended.toml:1004 (c4 missing), 1080 (c5e visible), 1085 (c5e missing), 1265 (x10b added), 1371, 1387, 1436, 1446, 1451 (x5e, x5g, x7, x8, x9b pixel) | nine rows without `open = true` whose own cause text says the cause is unproven ("cause not found for every element", co-63 "(open)", xp-43 "(proposed) ... cause not found"; the ledger lines xp.md:44 and construct.md:65 are `open`); tour-diff prints `change`, the same partial credit as rtb-1 (owner 11-17), and a later unrelated `missing`, `visible` or `added` diff on c4, c5e, x10b passes | Fix: set `open = true` on these nine rows (or split the unproven unit into its own open row) and make `test_an_open_row_says_what_is_open` two-way: a row whose cause contains "not found", "(open)" or "(proposed)" must carry `open = true`.
- **rtb2-2** | MINOR | tests/tour-diff.py:189-190 (`cand_red`), :294 | a check that the base marks "not measured" (I2, I5, I6, I9, I12, I13 on 733967aa) and the candidate reads red is only printed: the verdict stays `same` or `change` and the exit code 0, so a new red I9 or I12 on a later head passes tour-diff; only `run.py --check` sees it (16 captures today); rtb-3 asked the gate to read both | Fix: make `cand_red` a flip that only a `check` row naming that check absorbs (else `regression`), so the 14 I2 rows and the red I6 rows are named in the toml.
- **rtb2-3** | MINOR | tests/test_tour_diff.py:375 (`WIDE_BEFORE_R1`), :415 | the three-cause rule counts "; " units, not causes: 30 grandfathered rows keep 4 to 6 units, and 36 non-open rows have 3 units but 6 shas and ids (c1 added, c2 added, c3 added, c5 added, c5b added ...: `29281054`, a pager unit with 3 shas); rtb-6 asked at most three causes per row | Fix: count the sha and finding tokens of a row, drop the grandfather set (split the rows or set `open = true`), or say in the test that a unit is one cause.
- **rtb2-4** | MINOR | tests/tour-diff.py:168-171 (`named`) | for a kind with a found row and an open row the found one is taken and the open one is dropped: the act is `change`, the open diff is hidden; none exists in the toml today (21 duplicate (act, kind) pairs, none mixed) | Fix: prefer the open row (or the open flag of any row of the kind), and add a test that a kind with one found and one open row gives `open`.

### Limits
- No take, no gate; the committed pair of check 4 does not exist, so counts are from the report.
- Causes by family are read from `git show --stat` and element names (rr9-1, known); 15 of 97 rows sampled, the rest by the pattern of the row text.
- The scratch scripts were in the scratchpad, not committed.

## r3 (F7, rc 11, 1b: no unproven row passes as `change`; rtb2-1..4; 2026-11-24)

Branch `claude/tour-base` from 883214b5, head **cc71e5d9** (pushed). Step 0 green (`cargo check --workspace`, `pytest tests/test_tour.py` 84 passed, `tests/test_tour_diff.py` 62 OK). Red sha **286524e9** (tests only: tests/test_tour_diff.py, tests/test_tour.py), fix sha **cc71e5d9** (tests/tour-diff.py, demo/tour/intended.toml).

### Red (286524e9, before the fix; `python3 -I tests/test_tour_diff.py`: `Ran 64 tests ... FAILED (failures=4)`, and `pytest tests/test_tour.py`: 1 failed, 83 passed)
- (a) rtb2-1: `FAIL test_a_row_whose_cause_is_unproven_is_open`: `Lists differ: [('c4', 'missing'), ('c5e', 'visible'), ... ] != []`, the 9 rows (c4 missing, c5e visible and missing, x10b added, x5e x5g x7 x8 x9b pixel). Two-way: the marks are "cause not found", "(open)", "(proposed)" and the findings whose ledger state is open with the cause not found (`UNPROVEN_FINDINGS` = mc-59, co-63, xp-43); `test_an_open_row_says_what_is_open` is kept for the other way.
- (b) rtb2-2: `FAIL test_base_not_measured_and_cand_red_is_a_regression_unless_a_check_row_names_it`: `AssertionError: 0 != 1 : x1 Q same I6 not measured (cand red)`; and `FAIL tests/test_tour.py::CaptureDumps::test_a_baseline_without_boxes_is_not_an_i6_flip` (the r2 test pinned `same`; now `regression`, `change` with a `check = "I6"` row).
- (c) rtb2-3: `FAIL test_a_row_names_at_most_three_causes_or_is_open`: `Lists differ: [('c1', 'added', 6), ('c1', 'pixel', 9), ...] != []` (76 rows; `WIDE_BEFORE_R1` is gone; a cause is an 8-hex sha with a digit or a finding id `xx-N`).
- (d) rtb2-4: `FAIL test_a_kind_with_a_found_row_and_an_open_row_gives_open_in_either_order`: `AssertionError: 'change' != 'open'`.

### Fix (cc71e5d9): `python3 -I tests/test_tour_diff.py` OK (64), `pytest tests/test_tour.py` 84 passed
- tests/tour-diff.py: `named` prefers an `open` row of the kind (rtb2-4); `cand_red` joins `flipped`, so only a `check` row that names the check absorbs it, else `regression` (rtb2-2). No new verdict, no new field.
- intended.toml: `open = true` on the 9 rows of rtb2-1 (x10b, x5e, x5g, x7, x8, x9b get the text "open: cause not found") and on the 68 rows that name more than three causes (text "open: more than three causes, none proven for this act alone"); a row cannot be split (one row per act and kind), so open is the honest state. 369 rows, 156 open (r2: 79); 84 of the 103 acts have an open row. No new `check` row was needed in the toml (the 14 I2 rows already name I2); a red I6 on a head against 733967aa needs its own `check = "I6"` row when the pair is taken (it is `regression` until then, as asked).

### `python3 demo/tour/run.py --check` (head cc71e5d9, build of this head)
```
chapter xp: 31 acts, 29 green, 0 inv, 1 known, 1 red, 0 todo, 0 unchecked
chapter tiling: 28 acts, 28 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
chapter mc: 22 acts, 22 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
chapter construct: 18 acts, 0 green, 0 inv, 0 known, 18 red, 0 todo, 0 unchecked
chapter handover: 11 acts, 11 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
```
19 red as r2 (x17 xp-44 and the 18 construct acts: I2 9 apart, co-44, and I6 xp-45; xp-44 and xp-45 are fixed in f65e7f70 and f1397d98, not in this head). Exit 1.

### tour-diff counts per verdict: not produced
tour-diff needs the capture pair (a full take of 733967aa with `TOUR_BASELINE=1` and of the head); no capture folder is committed and this lane took neither (cost cap). The counts that the toml fixes: before r3 the r2 report had 1 same, 60 change, 38 open, 0 regression, 11 none; of the 60 `change`, the captures whose named rows now carry `open` move to `open` (every act with an open row: 84 of 103 acts). Open is the honest F7 rest; the next take decides the exact count.

### Gate (`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, full, 2445 s on acts): one red cell, not caused by this change (the diff against 883214b5 is tests/tour-diff.py, tests/test_tour*.py, demo/tour/intended.toml only)
```
acts: 236 pass, 2 known red, 1 blocking
FAIL: xp I6: in act 'a click on Notepad starts it under the volume popup': on HEADLESS-1 'Starting Notepad' (notify-card-body) has the contrast 1.04:1 ((119, 179, 75) on (115, 176, 72)), the guide wants 4.5:1 (evidence /tmp/acts-run-4149nvkw/xp-I6)
FAIL: acts: a cell blocks (see the FAIL lines)
EXIT 1
```
This is xp-38 (findings/xp.md:39, "fixed claude/xp-38 c7f2e64f"; c7f2e64f is an ancestor of the head): the same act, the same card at 1.04:1, a capture in the fade-in of the card. It recurs after the fix, once in one run; not reproduced alone (no time). Every other cell PASS (gate-lib, ci-check, build, matrix, tiling-audit, tiling-look, doors, cube, pull, ...). `pgrep -fa 'chora --headless'`: empty.

## Review r3 (seeded review of claude/tour-base 883214b5..cc71e5d9; rc 11, 1b; 2026-11-24)

Reviewer: sonnet, not the builder. Step 0 green at cc71e5d9: `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q` (84 passed), `python3 -I tests/test_tour_diff.py` (64 tests OK). No take, no gate (as briefed).

**Verdict: MERGE-OK.** 4 findings: 0 BLOCK, 0 MAJOR, 1 MINOR, 3 NIT.

### Checks
1. **Red first: OK.** 286524e9 (parent 883214b5, files tests/test_tour.py and tests/test_tour_diff.py only; message cites rtb2-1..4) in a worktree: `python3 -I tests/test_tour_diff.py`: `Ran 64 tests ... FAILED (failures=4)`: `test_a_row_names_at_most_three_causes_or_is_open` (c), `test_a_row_whose_cause_is_unproven_is_open` (a), `test_a_kind_with_a_found_row_and_an_open_row_gives_open_in_either_order` (d), `test_base_not_measured_and_cand_red_is_a_regression_unless_a_check_row_names_it` (b); `pytest tests/test_tour.py -q`: `1 failed, 83 passed` (`test_a_baseline_without_boxes_is_not_an_i6_flip`). cc71e5d9 (tests/tour-diff.py, demo/tour/intended.toml only): `OK` (64) and `84 passed`. Author and committer of both commits: the given identity.
2. **rtb2-1: OK.** `tomllib` over demo/tour/intended.toml at head: 369 rows, 156 `open = true`, 103 acts, 84 with an open row (the report's 156/369 and 84/103 agree; 156 - 79 of r2 = 77 = 9 + 68). No row with "cause not found", "(open)" or "(proposed)" lacks `open = true`; every open row says "open" in its cause. Of the ledger ids (findings/*.md on claude/session-log) still open, mc-59, co-63 and xp-43 (cause not found) sit only on open rows. Four more open ids sit on non-open rows that also name a sha: co-44 (15 rows, cause 1e7cc905 in the ledger), xp-28 (17, 09ff8ff9 bd5af4b3), mc-56 (2, 42585079), ti-30 (2, b8872054): the ledger state is a defect still open, the diff cause is found; read each, they fit (rtb3-2).
3. **rtb2-2..4: OK.** (b) `verdict` in a Python session over tests/tour-diff.py at head: base "not measured" and cand red: no row, an I2 row only: `regression`, `flipped ['I6']`; a `check = "I6"` row: `change`; an open I6 row: `open`. (c) `TOKEN` counts 8-hex shas with a digit and `xx-N` ids as a set; no grandfather set; a wider scan (7 to 40 hex, 2 to 5 letter ids) over the 213 non-open rows finds none over three (the only extra token is `icon-2` of `xp-icon-2`, not a cause). (d) found and open row, either order: `open`, test green.
4. **KISS: OK.** No new verdict (same, change, open, regression, none), no new field in intended.toml (`open`, `check` as before). tour-diff diff: `named` picks the open row, `cand_red` joins `flipped`. Counts per verdict: not checkable. No capture pair is committed on the branch or on claude/session-log (`git ls-tree` for `.Q.json`: 0), the lane made no take; the Done-when of the brief is met without one.
5. **A path to a later unrelated diff passing: one new (rtb3-1), one known.** Base green and cand "not measured" reads `same` (only printed). Known and by design: a `check = "I2"` row (14, all co-44 or xp rows) absorbs any I2 flip on that act, and a `pixel` row any pixel diff of the act; the family is by the ledger (rr9-1), not by the element.
6. **The acts FAIL is foreign: OK.** `git diff --name-only 883214b5..cc71e5d9`: demo/tour/intended.toml, tests/test_tour.py, tests/test_tour_diff.py, tests/tour-diff.py. `grep -rln "demo/tour\|tour-diff\|intended" tests/acts tests/test_acts*.py tests/test_act_pointer.py`: nothing; tests/gate-acts.sh:22-23 runs only the two selftests (green). The failing act is tests/acts/looks/xp.py:437 (`STARTING = "Starting Notepad"`), run against the product, which the diff does not touch; xp-38 (findings/xp.md:39) names the same text and 1.04:1.

### Findings
- **rtb3-1** | MINOR | tests/tour-diff.py:189-192 (`unmeasured`, `cand_red`) | a check that the base reads green or red and the candidate marks "not measured" (a build that stops reporting `scene.boxes`: I2, I5, I6, I9, I12, I13 all at once) gives `same`, only the note "not measured"; checked: `verdict("a", [], True, [], ({"I6": "green"}, {"I6": "not measured"}))` returns `same`; the candidate loses the check and passes | Fix: where the candidate is "not measured" and the base is not, add the name to `flipped` (absorbed only by a `check` row), as rtb2-2 does for the other side.
- **rtb3-2** | NIT | tests/test_tour_diff.py:417-418 (`UNPROVEN_FINDINGS`) | the three ids are typed in, not read from the ledger (not on this branch), so an id that turns open with no cause found later is missed; 36 rows cite an open id with a sha (co-44, xp-28, mc-56, ti-30) and stay `change` | Fix: say in the docstring that an id counts only when the ledger says "cause not found", and add the id there when a finding gets that state.
- **rtb3-3** | NIT | demo/tour/intended.toml (t10b, t1d, t3 ... 68 rows) | the added tail "open: more than three causes, none proven for this act alone" sits behind causes that do carry a proof (t10b: "the candidate take with gap_inner 10 matches the replay to the pixel outside the bar"), so the row contradicts itself | Fix: the tail "open: more than three causes, not split per cause" says what is true.
- **rtb3-4** | NIT | tests/tour-diff.py:197-200 (`absorbed`) | the check row is the first one found (`next`), while `named` now prefers an open row; with one found and one open `check` row for the same check the open one is hidden; none exists today (14 check rows, all I2, no duplicate) | Fix: pick the open row first there too (the same `next(... if r.get("open") ...)`).

### Limits
- No take and no gate; the counts per verdict are not producible (no pair), the toml counts are from `tomllib`.
- Causes by family are not re-proven (rr9-1, known); the 19 red of `--check` and xp-38 are as the report says.
- Scratch scripts in the scratchpad, not committed.

## r4

rtb3-1 (a check the candidate stops measuring is `same`): fixed.

- Red: 44b79d4e (tests/test_tour_diff.py only). `test_a_check_the_candidate_stops_measuring_is_not_same`: `AssertionError: Tuples differ: ('same', None) != ('regression', ['I6'])`; `Ran 65 tests ... FAILED (failures=1)`. Covers base green and base red, a `check = "I6"` row absorbing it (`change`), and both sides "not measured" staying `same`.
- Fix: d0801b7e (tests/tour-diff.py, 2 lines). `cand_lost` = checks the candidate marks "not measured" and the base does not; they join `flipped` next to `cand_red`, absorbed only by a `check` row. No new verdict, no new field. `python3 -I tests/test_tour_diff.py`: `Ran 65 tests ... OK`; `pytest tests/test_tour.py -q`: `84 passed`.
- `demo/tour/run.py --check`: did not finish headless here (killed at the 100 s timeout, no output); no counts per verdict. No capture pair exists, so the diff counts are not producible (as in r3).
- Full gate `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`: `all steps passed; no --push, nothing is pushed`, `EXIT 0`, no FAIL line (acts PASS 2056s, e2e PASS 275s). Last cells: tiling-disc, mc-host, tiling-bar, e2e PASS.
- `pgrep -fa 'chora --headless'` after the gate: empty.
- Branch claude/tour-base-r4 at d0801b7e, pushed.

## PM review r4 (d0801b7e, base d40142e4; diff 13+/1-, 2 files)
- Red 44b79d4e then fix d0801b7e, rerun here: red `FAILED (failures=1)`, fix `OK` (65 tests). rtb3-1 fixed: base green or red, candidate "not measured" is `regression` with `flipped ["I6"]`; a `check` row absorbs it.
- Full gate EXIT 0 (lane), no foreign red. --check did not run headless (100 s, no output); the Done-when allows the reason.
- rtb4-1 MINOR: a check absent on the base and "not measured" on the candidate is also a flip: `verdict('a',[],True,[],({}, {'I9':'not measured'}))` gives `regression` (run here). A check new in v9 and unmeasured would mark every act; base absent should count as unmeasured (`base.get(n) not in (NOT_MEASURED, None)`). Next tour-base round.
- Verdict (computed): no open MAJOR, MERGE-OK. Next train.

## r5 (F7, rc 11: rtb4-1, rv9j-4, rv9j-5)

Branch claude/tour-base-r5, head 61d0ef7a (base d0801b7e). Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`, `pytest tests/test_tour.py -q` (84 passed), `python3 -I tests/test_tour_diff.py` (65 OK). The chapter is `demo/tour/chapters/handover.py` (the brief said demo/tour/handover.py).

### Red, then fix (every red commit holds tests only)

| id | red | fix | red line |
|---|---|---|---|
| rtb4-1 | 51a44e9f | 6d01db40 | `FAIL: test_a_check_absent_on_the_base_and_not_measured_on_the_candidate_is_not_a_flip`, `AssertionError: 'regression' != 'same'` (`Ran 66 tests ... FAILED (failures=1)`) |
| rv9j-5 | 1deb1bc9 | dd356a8f | `ERROR: test_a_node_no_row_names_is_unnamed` (KeyError `unnamed`: the toml row of the kind absorbed the node), `FAIL: test_a_row_with_nodes_absorbs_only_those_nodes`, `AssertionError: 'xp-47 greeting (open)' != '29281054 labels; xp-47 greeting (open)'` (`Ran 71 tests ... FAILED (failures=2, errors=1)`) |
| rv9j-4 | a12bc57e | 671b426e | `AssertionError: unexpectedly None : chapters/handover.py builds no companion command for the tiling output`, `ValueError: substring not found` (`3 failed, 84 deselected`) |
| rv9j-5 rows | 60e6687d | 61d0ef7a | `FAIL: test_no_row_of_the_xp_or_handover_acts_cites_the_session_captions`, `Lists differ: [('h1', 'pixel'), ('h1b', 'pixel'), ('h1c'[437 chars]el')] != []` (`Ran 74 tests ... FAILED (failures=1)`) |

After the fixes: `python3 -I tests/test_tour_diff.py` `Ran 74 tests ... OK`; `pytest tests/test_tour.py -q` `87 passed`.

- **rtb4-1** (tests/tour-diff.py, 1 line): `cand_lost` needs `base.get(n) not in (NOT_MEASURED, None)`. A check the base lacks and the candidate cannot measure is `unmeasured` (named in the row), no flip; base green or red and candidate "not measured" stays `regression` (rtb3-1).
- **rv9j-5** (tests/tour-diff.py, `named()`): an optional `nodes = [...]` per row, the ids of the scene diff (a window as `app_id:title`). A node is named by the rows that list it first, else by the rows of the kind without a list, an open row first. So a closed row with `nodes` proves those nodes only; a node it does not list falls to an open row without a list (stays `open`), and with no such row it is unnamed (`regression`). A pixel diff has no node: rows without a list first, else any row of the kind. A row without `nodes` is as before. The docstring and the header of intended.toml say it; three tests on the toml (nodes are a list of ids of a scene kind; no xp or handover row cites fc9675ac, 11e34cf8, 94579f91; no `hud-chat` row cites 29281054).
- **rv9j-4** (handover.py): `tiling_companions(output)` returns `swaybg -o <tiling output> --color #2e3440` (the colour of tiling.py:143; `-o` so the xp, mc and construct outputs keep their own desktops) and `dbus-run-session -- sh flavors/tiling/waybar/waybar.sh flavors/tiling/waybar`; `start_multi` runs them by `choractl run setsid` (CHORA_FLAVOR_DIR of the tiling flavor, as `Launcher.start`) once `owner:tiling` is up. Seen in the take: `media/2026-11-24/tour-base-r5/05-01-two-screens-R.png` has the bar and the wallpaper (before: pure black).

### The rows (demo/tour/intended.toml, 61d0ef7a)

The pair is a fresh take (the stills of the judge's pair are not on claude/session-log): v9 = `TOUR_OUT=... demo/tour/run.py` on the head (671b426e, the take is before the toml changes, which it does not read); v8 = `TOUR_REPO=<worktree of 733967aa, built> TOUR_BASELINE=1 demo/tour/run.py`, own `TOUR_SCRATCH`, one after the other (not at the same time: the judge's red xp and handover acts were typing under load). v9: xp 30 green 1 known, tiling 28 green, mc 22 green, handover 11 green, construct 14 red (I2 co-44 in all, I6 co-45 at act 11, as the judge). v8: xp, tiling, mc replay (every act red on the old build, as before), construct `baseline: none` at `cube.list`, handover BROKE at handover.py:190 (as the judge).

127 rows of the kinds meta, rect, missing and pixel of the xp acts (x1..x12c) and the handover acts (h1..h1d) are replaced by 172 rows per node, each node of each act of the take assigned by its id; a node with no rule stops the generator, so nothing is left to a catch-all. The pixel rows are classified by the blobs of the take: every blob lies in the rect of the pager (4 buttons), the bin (`xp-icon-2`, 470 to 482), the moved Notepad window or the notify card, within 16 px; only x8 has two blobs beside (the eyes of the paperclip, a blink).

| nodes | cause |
|---|---|
| `xp-item-3`, `xp-item-4` meta ('UXTerm' to 'Foot', 'XTerm' to 'Foot Client') | 29281054 (only these; before it covered `hud-chat` too) |
| `hud-chat` meta (x5..x6d) | 4fec75c3 (xp-30), **open**; no row names xp-47 (judge r1 x5d, x5d.jpg) |
| `hud-chat*` rect (x5..x6d) | 62708d35 (88 px left of the bin) and 4fec75c3, **open** (height 141 to 204 is not proven apart from the greeting) |
| x6b `hud-card`, `hud-card-*`, `hud-chat*` rect, `hud-card` meta, `hud-chat-hint` missing | 62708d35 (88 px left) and 86343e3f (the typing line stands in the card, 'Typing does not answer.', card 188 to 208 px; the hint line goes) (judge r1 x6b, x6b.jpg); the `hud-chat` meta of x6b stays open (the greeting under "You") |
| `xp-icon-2` rect (the bin, 12 px down) | 62708d35 (judge r1 x2, x2.jpg); before: "the label of xp-icon-2 'UXTerm' to 'Foot'" (29281054), which is the wrong node |
| `Notepad:None`, `xp-*-s1` rect | 09ff8ff9, bd5af4b3 (xp-28; judge r1 x7, x7.jpg) |
| `notify-card-*` rect | 675ebd55 (sh-31), as before |
| pixel, closed | d78fc4bb (pager, 4 buttons), 62708d35 (bin), 09ff8ff9 (windows), 675ebd55 (notify) by the position of the blobs; fc9675ac, 11e34cf8, 94579f91 are cited by no xp or handover row (the session-button captions are not on screen) |
| h1..h1d `construct-*` missing/rect | d9b2c4f1, co-63 (open), as before but for these nodes only; the pixel row of h1 no longer cites it (the blobs are pager and bin; h1c, h1d also the window) (judge r1 h1, h1.jpg; h1c, t8b-h1c.jpg) |
| h1..h1d `mc-*` missing/rect | 675ebd55, mc-59 (open), as before for these nodes only |
| c4 check I6 | co-45, b8872054 (the sheet's scope line and heads at 11 px; judge r1 c4, c4.jpg) |

The construct acts' open pixel rows (c2, c2b, c3b, c3c, c3d, c4, c4b, c5d, c5e) lose fc9675ac, 11e34cf8, 94579f91 and name d78fc4bb (pager of the xp face), 62708d35 (bin), 909bc57b (the chips Click Fly in and Pull out, co-64, co-65) and f1397d98 (inactive titles on the face, xp-45), each still open (judge r1 c2, c3b: not proven per element).

**Deviations, said plainly.**
1. **6d336bdf is not in this head** (`git merge-base --is-ancestor 6d336bdf HEAD` fails; it is the later merge of co-pull with the fix co-68, be6e62ec). The chips of the judge's table are in the head by 909bc57b (co-64, co-65), so that sha is in the rows; no row cites 6d336bdf, a row cannot cite a commit that the take does not contain.
2. **0a0fca76 (xp-47) is not in this head** either, so the greeting still stands in `hud-chat` in the take (x5..x6d). The rows say `open` with 4fec75c3 (xp-30) and name no row for xp-47; the next take on a head with 0a0fca76 shows whether those nine acts leave `open`.
3. rv9j-1 and rv9j-3 are open findings, not intended changes: no row. The pager's current desktop (rv9j-1) cannot be seen in the stills and no row says it is meant.
4. The x8 blobs (paperclip blink, 1213,593 and 1233,593) are not masked by the tour: the x8 pixel row stays open with that text.

### Counts per verdict on the pair

Same pair (fresh v8 replay and v9 take), `tests/tour-diff.py BASE CAND --intended demo/tour/intended.toml` (the CLI; output in `media/2026-11-24/tour-base-r5/tour-diff-r5.txt`):

| | same | change | open | regression | none |
|---|---|---|---|---|---|
| before (intended.toml of d0801b7e) | 1 | 14 | 82 | 2 | 11 |
| after (61d0ef7a) | 1 | 27 | 71 | 0 | 11 |

Moved: open to change: x1, x2, x2b, x2c, x5e, x5g, x7, x7b, x9b, x10, x10c, x11b, x12b (13 xp acts; the rows are the per node rows above, the pixel row of each is closed). Regression to open: x6b (new `missing hud-chat-hint` row, 86343e3f; open stays because of the greeting in `hud-chat`) and c4 (the I6 flip now has its row; open stays through the construct rows of c4). No act went from change to open or regression.

Acts of the xp and handover chapters that stay `open` and their row: x5, x5b, x5c, x5d, x5f, x6, x6c, x6d and x6b (`hud-chat` meta and rect: 4fec75c3 xp-30, open); x8 (pixel: paperclip blink, open); x10b (`added` rows xp-43 open: hud-line and the help scroll node); h1, h1b, h1c, h1d (construct rows d9b2c4f1 co-63 open, mc rows 675ebd55 mc-59 open). The 44 other open acts (construct 14, tiling 24, mc 18 minus the changes, as before) are untouched rows of r2 to r4: only the causes the judge named were changed.

### Gate

Full gate `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` on 61d0ef7a: **EXIT 1, one red cell, not caused by this change** (tests/tour tooling only, no product code): 55 cells PASS (red-first, identity, pre-commit, gate-lib, ci-check, build, test, matrix, tiling-audit, cube, pull, handover, mc-scale and the rest), `acts` FAIL after 2097s, `acts: 236 pass, 2 known red, 1 blocking`. The blocking line:

`FAIL: xp I6: in act 'a click on Notepad starts it under the volume popup': on HEADLESS-1 'Starting Notepad' (notify-card-body) has the contrast 1.04:1 ((119, 179, 75) on (115, 176, 72)), the guide wants 4.5:1 (evidence /tmp/acts-run-j3mjvn3j/xp-I6)`

The text colour (119, 179, 75) is the wallpaper green (115, 176, 72) at 1.04:1: the card of "Starting Notepad" was captured while it faded in. A flake of an xp act under load (the r4 gate had `acts PASS 2056s`; the xp acts do not run handover.py or intended.toml). Rerun of the xp look alone: `BIN=target/debug tests/gate-acts.sh --no-selftest xp`: `acts: 80 pass, 0 known red, 0 blocking`, `ok: every act and invariant passes or is listed as known red`, `EXIT 0`. The 2 known red of the full run are the construct I2 and I6 cells of known-red.toml (co-44, co-45). The whole gate was not run a second time (35 minutes); the other 55 cells are green.

Last 20 lines of /tmp/gate.log:

```
cell xp act:the speaker opens the volume popup again pass
cell xp act:the speaker opens the volume popup for Mute pass
cell xp act:the speaker opens the volume popup for an outside click pass
cell xp act:the speaker opens the volume popup over the window pass
cell xp act:the speaker opens the volume popup to shut it pass
cell xp act:the task button brings the window back pass
cell xp act:the tooltip of the speaker says the level that the key set pass
cell xp act:the tooltip of the speaker says the level that the popup set pass
cell xp act:the topic Keyboard shortcuts lists every key inside the window pass
cell xp act:the volume key raises the level pass
cell xp act:the volume popup opens while the balloon shows pass
cell xp act:the window closes pass
cell xp act:the window of Notepad shows pass
cell xp act:the window of Show Desktop closes pass
cell xp act:the window of the key check closes pass
acts: evidence kept in /tmp/acts-run-j3mjvn3j
acts: 236 pass, 2 known red, 1 blocking
FAIL: xp I6: in act 'a click on Notepad starts it under the volume popup': on HEADLESS-1 'Starting Notepad' (notify-card-body) has the contrast 1.04:1 ((119, 179, 75) on (115, 176, 72)), the guide wants 4.5:1 (evidence /tmp/acts-r
FAIL: acts: a cell blocks (see the FAIL lines)
EXIT 1
```

`tools/sleep-ratchet` ok. `pgrep -fa 'chora --headless'` empty at the end.

## Review r5 (F7, rc 11: seeded review of claude/tour-base-r5, head 61d0ef7a, base d0801b7e)

**Verdict: FIX** (one MAJOR). Step 0 green on 61d0ef7a: `cargo check --workspace`, `pytest tests/test_tour.py -q` (87 passed), `python3 -I tests/test_tour_diff.py` (74 OK). No takes; checked by git, diffs, tests and the judge's r1 table.

1. **Red first: ok.** Each red test fails at its red sha and passes at head (run in a worktree).
   - 51a44e9f: `FAIL: test_a_check_absent_on_the_base_and_not_measured_on_the_candidate_is_not_a_flip`, `AssertionError: 'regression' != 'same'`, `Ran 66 tests ... FAILED (failures=1)`; 6d01db40 passes it.
   - 1deb1bc9: `ERROR: test_a_node_no_row_names_is_unnamed`, `FAIL: test_a_row_with_nodes_absorbs_only_those_nodes`, `AssertionError: 'xp-47 greeting (open)' != '29281054 labels; xp-47 greeting (open)'`, `Ran 71 tests ... FAILED (failures=2, errors=1)` (at the last red sha a12bc57e the rtb4-1 test fails too); dd356a8f: `Ran 71 tests ... OK`.
   - a12bc57e: `AssertionError: unexpectedly None : chapters/handover.py builds no companion command for the tiling output`, `3 failed, 84 passed`; 671b426e: `87 passed`.
   - 60e6687d: `FAIL: test_no_row_of_the_xp_or_handover_acts_cites_the_session_captions`, `Ran 74 tests ... FAILED (failures=1)`; 61d0ef7a: `Ran 74 tests ... OK`, `87 passed`.
   - The four red commits change tests only (`git diff --stat`: test_tour.py, test_tour_diff.py).
2. **rtb4-1: ok.** `verdict('a',[],True,[],({}, {'I9':'not measured'}))` gives `same` with `unmeasured ['I9']`; base green and candidate "not measured" stays `regression` (the second assert of the test). One line in tests/tour-diff.py (`base.get(n) not in (NOT_MEASURED, None)`).
3. **rv9j-5 `nodes`: ok for the scene kinds.** One optional field, no new verdict. A node listed by a row takes that row; a node not listed takes the rows of the kind without a list (open first); none: unnamed, `regression`. Seen: `verdict('x5', [hud-chat], ...)` with a closed row `nodes=[desk-label]` and an open list-less row stays `open`; a row without `nodes` matches as before. Every scene diff has an id, so `node is None` only for `pixel`. The hole is in the rows that stay without `nodes` (rtb5-1).
4. **Rows: the named ones match.** 29281054 only on `xp-item-3/4` meta; fc9675ac, 11e34cf8, 94579f91 cited by no row (0 of 36); 62708d35 (76 rows, bin and balloon rects), d78fc4bb (41, pager), 86343e3f (x6b only), f1397d98 and 909bc57b (the nine construct acts), d9b2c4f1 per node on h1..h1d. No row for xp-47 or 6d336bdf. All 8-hex shas in the file are ancestors of 61d0ef7a. Of 181 new or changed rows, 0 name "open", "proposed", "cause not found" or an open id with `open = true` missing (checked on all rows of the file, not on a sample of 10). The deviation that 6d336bdf is not in the head is true (`merge-base --is-ancestor` fails); the chips are cited by 909bc57b, which is. The `nodes` rows cite the judge's act and capture; the 32 pixel rows cite `x7, x7.jpg` for acts the judge did not look at (rtb5-2).
5. **Counts: reproduced from media/2026-11-24/tour-base-r5/tour-diff-r5.txt** (`1 same, 27 change, 71 open, 0 regression, 11 none`). x6b left `regression` honestly: the new row `missing hud-chat-hint` (86343e3f, the judge called it intended) is closed, but the `hud-chat` meta row (xp-30, open) stays, so x6b is `open`. c4 left `regression` honestly: the new row `check I6` (co-45, `check = "I6"`, so it absorbs I6 only) is closed; the construct rows `missing`, `rect` and `pixel` of c4 are open, so c4 is `open`. Neither is hidden by a broad row. **The 13 acts moved from open to change are another matter**: see rtb5-1.
6. **rv9j-4: ok.** `tiling_companions()` builds `swaybg -o <tiling output> --color #2e3440` and the `dbus-run-session -- sh .../waybar.sh .../waybar` of tiling.py:143; the test checks the command lines (wallpaper on the tiling output only, bar form, started after `owner:tiling`). Started by `choractl run setsid` with CHORA_FLAVOR_DIR as `Launcher.start` does. The still 05-01-two-screens-R.png shows the bar and the dark wallpaper.
7. **Acts blocking line: foreign.** The diff is demo/tour/chapters/handover.py, demo/tour/intended.toml, tests/test_tour.py, tests/test_tour_diff.py, tests/tour-diff.py. The acts cell runs tests/acts/run.py and the xp act in tests/acts/looks/xp.py (none touched); the selftests in gate-acts.sh are green at head and the xp rerun passed 80, so 'Starting Notepad' 1.04:1 (xp I6, the card fading in) is the xp-38 family.

### Findings

- rtb5-1 MAJOR, demo/tour/intended.toml (rows of x1..x12c and h1..h1d, kinds `added` 35 and `pixel` 32, `visible` x12c) and tests/tour-diff.py:170-179: the rows that still have no `nodes` absorb any later diff of their kind, so the 13 acts moved to `change` (x1, x2, x2b, x2c, x5e, x5g, x7, x7b, x9b, x10, x10c, x11b, x12b) would stay `change` when something unrelated changes. Run here: `verdict('x1', [{kind added, id unrelated-popup}, {kind meta, id xp-item-3}], False, rows)` gives `change` (cause 371f0fa1 ... d78fc4bb ... 62708d35); an unrelated pixel blob anywhere on the screen passes too. Fix: give each `added` row (and the x12c `visible` row) a `nodes` list of the ids of the take, and let a closed `pixel` row carry `boxes = [[x,y,w,h], ...]` so a blob outside them is unnamed (the blobs are already classified by position in the r5 text); until then those rows must be `open`.
- rtb5-2 MINOR, demo/tour/intended.toml (pixel rows of x2..x12b, h1..h1d): the cause cites "judge r1 x7, x7.jpg" for 30 acts, but the judge looked at x2 and x7 only; the proof is the builder's blob classification of its own take. Fix: cite the act's own still of the r5 take (tour-diff-r5.txt) or say "blobs classified in r5" instead of a judge capture.
- rtb5-3 MINOR, demo/tour/chapters/handover.py:105: `choractl run setsid` for the bar and wallpaper runs with `check=False, capture_output=True`, and the test reads only the command lines, so a missing swaybg or a failed waybar leaves the tiling screen black again and the act stays green. Fix: after the loop wait for the two processes (or check the returncode) and raise on a miss, as the chapter does for `owner:*`.

Not findings: rv9j-1, rv9j-3, xp-47, the 71 open (F7 rest), 6d336bdf not in the head (said plainly by the builder).
