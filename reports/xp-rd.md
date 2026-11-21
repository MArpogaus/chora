
## r1

Branch claude/xp-rd, head 2dfc7507. Order: red, fix per finding.

| finding | red | fix | test |
| --- | --- | --- | --- |
| rd-2 MEDIUM (Volume slider acts off the track) | f4b7e9c1 | d8b2d156 | context::tests::a_click_on_the_label_of_the_slider_does_not_set_the_level |
| rd-3 MINOR (dy 0 scrolls the key table) | d5f1cd26 | 2dfc7507 | help::tests::a_sideways_wheel_does_not_scroll_the_table |

Fix rd-2 acts only with the pointer in left - TRACK_END .. left + TRACK + TRACK_END, else no act. Fix rd-3 returns false for dy 0.0; sub-row accumulation not done.

Checks: cargo test -p chora-xp 324 passed; clippy -D warnings clean; lint-ratchet ok.
Gate: tests/merge-gate.sh --jobs 1 (no --only), branch equal to origin/claude/autonomous-dev-session-3w-i33pg7 plus the four commits: all steps passed (49 cells PASS, 0 FAIL).

### PM review r1 (17:44 UTC; PM not the builder; diff a84effda..2dfc7507, 2 files +38 -1)
- rd-2 closed: release() returns None off left - TRACK_END .. left + TRACK + TRACK_END; the test presses the label, 20 px left of the track and 2 px right of it; red f4b7e9c1 before fix d8b2d156.
- rd-3 half closed: dy 0.0 returns false (red d5f1cd26, fix 2dfc7507). A touchpad step under half a row still scrolls a full line (lines 0 -> signum); the brief made accumulation optional. Left: rxr-1 MINOR, help.rs:340, to backlog (xp polish after the RC).
- Verdict computed: 0 BLOCK, 0 MAJOR -> MERGE-OK; joins t102.
