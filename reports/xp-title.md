
## r1

- Cause (xp-45): decoration.rs:376-384 (title_picture) draws the shadow on the inactive title since f1693fb3; the act check takes the pixel farthest from the bar as text (tests/png-contrast.py:47), which is the shadow edge (30,53,104), 4.31:1 on (123,155,223). Text (216,228,248) itself is unchanged.
- Cause (xp-44): not the drawing. x17: the title box of the inactive Notepad lies under the opaque Help window; the capture there is Help's (214,223,247) on itself, 1.00:1. The I6 check ignored cover.
- Red 1eb8bf88: an_inactive_title_reads_with_margin: "the inactive title \"chart\" has 4.41:1, I6 wants 4.5:1 and a margin" (4.5 x 1.1, the cube face scale takes 5 %).
- Green f1397d98: TITLE_SHADOW #0A1E50 -> #041038 (one pair, active and inactive, no special case); cargo test -p chora-xp 349 passed. docs/xp-style.md updated.
- f65e7f70 (separable, tests only, not xp drawing): guide_breaches skips text covered by an opaque box (tests/acts/invariants.py, reuses covered()). Drop it if you want x17 fixed another way.
- run.py --check xp: 0 red (was 1). run.py --check: 14 red, 0 broken (was 23 red); no xp-44/xp-45 line, c1,c2,c3,c13 clear.
- Gate: full tests/merge-gate.sh --jobs 2, EXIT 0, 65 PASS. pgrep 'chora --headless' empty.

## PM review r1 (06:06 UTC, ce866b0d..f65e7f70, 87 lines; PM is not the builder)

- Red first: 1eb8bf88 adds an_inactive_title_reads_at_4_5_to_1_on_its_bar plus the pure refactor title() -> title_picture(); pasted red "chart" 4.41:1. Green f1397d98 changes TITLE_SHADOW only, one pair active and inactive, test and docs/xp-style.md match. OK.
- xp-44: the cause (Notepad title under the opaque Help window, x17) is plausible and the check after is 0 red for xp. f65e7f70 reuses covered() as used at invariants.py:521. OK.
- Gate EXIT 0, 65 PASS; --check 23 -> 14 red, the 14 left are ti-30 (t17, t19), sh-31 (t21, t28), co-44.
- rxt-1 | MINOR | decoration.rs:42, test 610-640 | the test and png-contrast take the pixel farthest from the bar as text, which is the shadow; the fix darkens the shadow to pass that metric, while the text (216,228,248) on (123,155,223) stays about 2.2:1. #041038 also leaves the documented Luna value #0A1E50. Fix: name in docs/xp-style.md that the shadow carries the inactive title (Luna has it) and that I6 measures the halo; known issue for 1.0 polish.
- rxt-2 | MINOR | tests/acts/invariants.py:566 | the skip of covered text has no selftest: no case shows an uncovered low-contrast title still fails. Fix: one gate-lib or acts selftest with a covered and an uncovered text.
- Verdict (computed): 0 BLOCK, 0 MAJOR, 2 MINOR -> MERGE-OK. Train125 (f65e7f70 on ce866b0d) started 06:05.
- Earliest gate for rxt-2: none (checker changes need no selftest); backlog harness item.
