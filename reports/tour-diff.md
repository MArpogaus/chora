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
