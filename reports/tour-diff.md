# tour-diff report

## r1 (lane 2 of design/tour-v9.md section 8, 2026-11-20)

Branch `claude/tour-diff` on 8b996cee (tour-one r1). Commits: red **a58a39f6** (tests plus stubs that flag nothing), green **fa796bb9** (tools, intended.toml, gate hook). Gate on fa796bb9: `pre-commit`, `gate-lib`, `ci-check`, `build`, `test`, `acts` all PASS (`--jobs 1`; 1, 85, 124, 157, 233, 278 s cumulative); `tests/gate-acts.sh` now runs `tests/test_tour_diff.py` (25 tests, 3.7 s).

**Files**
- `tests/tour-diff.py`: BASE and CAND folders with `<act>.<capture>.png` + `.json` (scene.query nodes, masks, checks). Scene diff (visible, rect over 2 px, meta text, owner, top-layer z-order, missing, added; windows by app_id and title), pixel diff (24 per channel, 0.1 %, 8-connected blob over 12x12; clock and text nodes and listed masks ignored; H50 is scene only), verdict same, change, regression, none (`baseline: none`). Reuses `png-pixel.py` (one PNG reader), no new copy. Exit 1 on a regression.
- `demo/tour/intended.toml`: `[[change]]` act, kind, cause; one row today (h2, xp-give Send To). The v8-open-row-gone rows come with CHAPTERS-A/B and RECORD.
- `tests/tour-video-check.py`: ffmpeg blackdetect (over 2 frames), freezedetect (1.5 s, `--cut` ranges excluded), flicker (frame differs from both neighbours that are equal), `--samples` for screen-fixed nodes that move in a camera move.

**Planted cases, red then green** (red = `python3 -I tests/test_tour_diff.py` run on a58a39f6's stubs; the video helper had a lavfi syntax slip, fixed before the red re-run; 14 failures and 1 error, all of the cases below):

| case | test | red | green |
|---|---|---|---|
| moved node | `SceneDiff.test_moved_node_is_flagged` | a58a39f6 | fa796bb9 |
| changed blob | `PixelDiff.test_changed_blob_is_flagged` | a58a39f6 | fa796bb9 |
| flicker frame | `VideoCheck.test_flicker_frame_is_flagged` | a58a39f6 | fa796bb9 |
| freeze (and one inside a cut is not) | `VideoCheck.test_freeze_is_flagged` | a58a39f6 | fa796bb9 |
| black stretch (one frame is not) | `VideoCheck.test_black_stretch_is_flagged` | a58a39f6 | fa796bb9 |
| fixed node moves in a camera move | `FixedInCameraMove.test_fixed_node_that_moves_is_flagged` | a58a39f6 | fa796bb9 |

Clean cases (2 px move, 3x3 change, masked change, below-threshold colour, a step that stays, a moving clip) stay green.

**Run on stored v8 captures** (branch claude/media-v8, `media/2026-11-17/tour-v8/`)
- Stills `stills/02-1-tile_left.png` as BASE and `stills/02-1-tile_bottom.png` as CAND (both 1280x720), act id t3, Q:
  `t3 Q regression pixel` (exit 1); with an intended row (act t3, kind pixel) `t3 Q change`; the same still on both sides: `t4 Q same`. 3 s per pair.
- Videos `01-xp.mp4` and `04-construct.mp4`: `tour-video-check.py` flags 21 and 7 freezes (v8 chapters hold stills with narration, so the long still stretches are real; no black frames, no flicker). v9 must give the video checker `--cut` ranges for slides and title cards, or accept the freezes as rows.

**Open**
- "Empty or wrongly themed frame at a change" (section 6) is not built: it needs the act timeline's change times; RECORD can add it by passing the stills of the checker.
- The snapshot JSON (`nodes`, `masks`, `checks` per capture) is the format `run.py` must write for a capture; tour-one rows carry `capture` but do not write it yet (RECORD lane).
- Pixel diff reads PNG in pure Python: 3 s per 1280x720 pair; fine for about 100 captures.

## Review r1 (F7 tour-diff, fa796bb9 on 8b996cee, 2026-11-20)

Verdict: **FIX** (2 MAJOR, 0 BLOCK, 5 MINOR, 1 NIT). Step 0 green (unshallow, `cargo check --workspace`). `python3 -I tests/test_tour_diff.py` on fa796bb9: 25 OK; on a58a39f6: 12 failures + 6 errors (the report says 14 + 1); the parent has no tool or test. v8 runs reproduce: t3 tile_left vs tile_bottom = regression (exit 1), with a pixel row = change, same vs same = same; 01-xp gives 21 freezes, 04-construct 7, nothing else.

- rd-1, MAJOR, tests/tour-diff.py:177: the loop runs over CAND stills only, so a capture in BASE that CAND lacks gives no row, and an empty or missing CAND folder prints "0 same" and exits 0 (verified; a missing capture turns into a pass). Fix: iterate the union of BASE and CAND, make a capture missing in CAND a regression, and exit 1 on zero captures.
- rd-2, MAJOR, tests/test_tour_diff.py:150: `skipUnless(ffmpeg)` skips all 7 video tests and the gate-acts hook passes (verified with PATH emptied: OK, skipped=7), so a runner without ffmpeg passes the gate with no video check. Fix: fail (not skip) in the hook when ffmpeg is absent, e.g. a `need ffmpeg` in tests/gate-acts.sh.
- rd-3, MINOR, tests/tour-video-check.py:27: ffmpeg or ffprobe missing is a raw FileNotFoundError traceback, not a clear error. Fix: `shutil.which` at the start, exit 2 with "ffmpeg missing: cannot check".
- rd-4, MINOR, demo/tour/intended.toml:9: the only row is `kind = "*"` for h2, which names every diff of h2 (pixel regressions included) as intended. Fix: list the kinds (meta, rect, added) of the Send To row, keep `*` for no row.
- rd-5, MINOR, tests/tour-diff.py:181: a BASE file that is absent reads as `baseline: none`, so a failed replay capture hides as none, not declared. Fix: take the `baseline: none` acts from a list (intended.toml or a file) and flag an undeclared absence.
- rd-6, MINOR, tests/tour-diff.py:154: a check absent or red on CAND with no BASE entry, and every red check of a `none` row, is not a regression (design: judged by checks). Fix: a check red on CAND that is missing from BASE is a regression unless intended.
- rd-7, MINOR, tests/tour-video-check.py:21: FLICKER_STEP 8.0, the 32x18 grey size and blackdetect pix_th 0.1 have no justification, and a 2-frame flash is not caught; the flicker test only uses flat colours and a `cuts` argument that flicker ignores. Fix: comment each threshold with the v8 data it came from and add a flicker test on testsrc2 with a 2-frame flash.
- rd-8, NIT, reports/tour-diff.md r1: the red counts differ from what a58a39f6 gives (12 + 6 against 14 + 1). Fix: restate the counts.

Not covered by a test: the CLI exit code, H50 (scene only), a size mismatch, `baseline: none`. The "empty or wrongly themed frame" check is open as the report says.

## r2 (fix of Review r1, claude/tour-diff, 2026-11-20)

Red **38690cb8** (tests only, on fa796bb9: 7 failures + 10 errors of 44; the errors are the new `compare(..., none)` argument), green **4f4f1897** (44 OK). rd-8: the red counts of r1 are restated as the review measured them, on a58a39f6 12 failures + 6 errors (not 14 + 1).

| finding | fix | test |
|---|---|---|
| rd-1 | compare runs over the union of BASE and CAND; a capture missing in CAND is a regression; no capture exits 1 | `CompareDirs.test_a_capture_missing_in_cand_is_a_regression`, `test_empty_or_missing_cand_exits_1` |
| rd-2 | `tests/gate-acts.sh` fails (not skips) when ffmpeg is absent | `GateHook.test_the_acts_hook_fails_when_ffmpeg_is_absent` (PATH without ffmpeg) |
| rd-3 | `shutil.which` first, exit 2 "ffmpeg missing: cannot check" | `VideoCheck.test_a_missing_ffmpeg_is_a_clear_error` (PATH empty) |
| rd-4 | h2 rows list meta, rect, added | `IntendedRows.test_h2_lists_kinds_and_no_wildcard` |
| rd-5 | `baseline_none = ["act.capture"]` in intended.toml; an undeclared absence is a regression | `test_an_undeclared_absence_in_base_is_a_regression`, `test_a_declared_baseline_none_is_none`, `test_the_cli_reads_the_none_list_of_intended_toml` |
| rd-6 | a check green on BASE and not green on CAND, or red on CAND and not red on BASE, is a regression unless a row of kind check; also on none rows | `test_a_check_red_on_cand_and_new_is_a_regression`, `test_a_red_check_on_a_none_row_is_a_regression`, `test_a_new_red_check_with_a_check_row_is_a_change`, `test_a_check_green_on_base_and_gone_on_cand_is_a_regression`, `test_a_check_red_on_both_is_not_new` |
| rd-7 | flicker finds 1 or 2 frame flashes by ratio (back step under half of the step in and out), so testsrc2 motion does not hide it; every threshold is commented in the module docstring with the six v8 videos' step statistics | `VideoCheck.test_a_two_frame_flash_on_testsrc2_is_flagged`, `test_a_one_frame_flash_on_testsrc2_is_flagged`, `test_testsrc2_alone_has_no_flicker` |

Also covered now: CLI exit code (`test_the_exit_code_follows_the_verdict`), size mismatch, H50 scene only, `baseline: none`.

**v8 rerun** (same inputs as r1): t3 tile_left vs tile_bottom still regression (exit 1), with a pixel row change, same vs same same; 01-xp 21 freezes and 04-construct 7 as in r1, no black and no flicker in any of the six videos (02-tiling 13, 03-mc-light 6, 03b-mc-dark 4, 05-handover 7 freezes). The numbers did not change: the fixes touch absent captures, absent tools and flashes, none of which v8 has.

**Gate** on 4f4f1897 (`--jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts`): pre-commit PASS 2 s, gate-lib 84 s, ci-check 114 s, build 145 s, test 203 s, acts 265 s (cumulative), all PASS.
