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
