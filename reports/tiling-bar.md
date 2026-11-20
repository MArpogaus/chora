# tiling-bar

## r1

Brief `briefs/tiling-bar-r1.md`, branch `claude/tiling-bar` from 79c8d2b5.
Red 3149f16f, fix bbb105ba, follow-up 3a9a426f (clock width). Head 3a9a426f.

The head already had the shape of the spec (three islands, 32 high, 8 from
the top, 12 from the sides, Apps, Keys, power icon, Inter 500 on the doors).
It differed in the surface (opaque, with an accent glow), the edge (a
gradient accent to accent-2, not a 1 px `line` hairline), the radius (12)
and the face of the clock and tags (JetBrains Mono, not Inter).

Cell `tiling-look`, gate `tests/gate-tiling-bar-spec.sh` (Ink and Paper,
scale 1 and 2, flat black output so that the 92 % shows). Captures in
`reports/tiling-bar/` as `before-THEME-SIZE.png` and `after-THEME-SIZE.png`,
THEME ink or paper, SIZE 1280x800 or 2560x1600@2.

| assert | on the head | red | fix | capture |
|---|---|---|---|---|
| bar-height 32 | green already | 3149f16f | none needed | before and after, all four |
| bar-top 8 | green already | 3149f16f | none needed | same |
| bar-sides 12 | green already | 3149f16f | none needed | same |
| bar-radius 10 | red at scale 2 (corner cut at 12) | 3149f16f | bbb105ba | `*-2560x1600@2.png` |
| bar-surface 92 % | red (opaque surface plus glow, 37 37 55 in Ink) | 3149f16f | bbb105ba | all four |
| bar-hairline 1 px line | red (gradient edge, 156 153 244 at the top) | 3149f16f | bbb105ba | all four |
| bar-font Inter 14 px 500 | Apps and Keys green already; clock red (127 px, mono) | 3149f16f | bbb105ba, 3a9a426f | all four |
| bar-words Apps, Keys, power | green already | 3149f16f | none needed | all four |

Changed in the fix: `flavors/tiling/waybar/style.css` only (colours from
`colors.css`, no literal colour). `config.jsonc` needed no change.
`tests/gate-tiling-bar-palette.sh` expected the accent gradient on the island
edges; it now expects the line colour (same file, fix commit bbb105ba). No
stored pixel baseline file exists on the branch, so none changed.
3a9a426f: Inter made the clock 20 px narrower, which moved the centre island
off x 570, the pixel that `tiling-audit` and `gate-tiling-look-r2` probe for
the bar (first gate run: tiling-audit FAIL "the bar does not show" at
2560x1600@2); `#clock` has `min-width: 127px`, so the island stays at 563 to
716 whatever the digits.

Wobbly for floating drags: already a default (`effects: [wobbly]` in
`plugins/chora-tiling/tiling.yaml`, gate-tiling-wobbly in `tiling-look`); no
change, no new feature.

Gate (head already contained origin/claude/autonomous-dev-session-3w-i33pg7):
pre-commit, ci-check, build, test, clippy, layout, matrix PASS on bbb105ba;
tiling-look, tiling-audit, doors PASS on 3a9a426f (tiling-audit was red on
bbb105ba, caused by this branch, fixed by 3a9a426f; rerun alone: PASS). The
later commit changes one CSS rule; the cells of the first run were not
repeated for it.

Red before this branch, not caused by it, not in the cells of the brief:
`tests/gate-tiling-bar.sh` ("the left island ends at 436 with tile_left, at
411 with tile") and `tests/gate-tiling-spacing.sh` (gaps of 13 px where 12 is
asserted; 1 to 2 px on the head). Both fail the same on 79c8d2b5; neither is
in the merge-gate cells. For the PM.

Open for the PM: Q1 of tiling-v2 (bar shape) is not answered in the files
read; this keeps three islands (1A, the recommendation). The spec line
"empty tag = `muted` 50 %" is not done: gate-tiling-bar asserts 4.5:1 on
empty tags (P7), and 50 % muted fails it.
