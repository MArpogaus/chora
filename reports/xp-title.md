
## r1

- Cause (xp-45): decoration.rs:376-384 (title_picture) draws the shadow on the inactive title since f1693fb3; the act check takes the pixel farthest from the bar as text (tests/png-contrast.py:47), which is the shadow edge (30,53,104), 4.31:1 on (123,155,223). Text (216,228,248) itself is unchanged.
- Cause (xp-44): not the drawing. x17: the title box of the inactive Notepad lies under the opaque Help window; the capture there is Help's (214,223,247) on itself, 1.00:1. The I6 check ignored cover.
- Red 1eb8bf88: an_inactive_title_reads_with_margin: "the inactive title \"chart\" has 4.41:1, I6 wants 4.5:1 and a margin" (4.5 x 1.1, the cube face scale takes 5 %).
- Green f1397d98: TITLE_SHADOW #0A1E50 -> #041038 (one pair, active and inactive, no special case); cargo test -p chora-xp 349 passed. docs/xp-style.md updated.
- f65e7f70 (separable, tests only, not xp drawing): guide_breaches skips text covered by an opaque box (tests/acts/invariants.py, reuses covered()). Drop it if you want x17 fixed another way.
- run.py --check xp: 0 red (was 1). run.py --check: 14 red, 0 broken (was 23 red); no xp-44/xp-45 line, c1,c2,c3,c13 clear.
- Gate: full tests/merge-gate.sh --jobs 2, EXIT 0, 65 PASS. pgrep 'chora --headless' empty.
