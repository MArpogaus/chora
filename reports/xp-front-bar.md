
## r1

Head 40292419 on claude/xp-front-bar (on d588d573). rcf-11 MINOR fixed.

| Site | Means | Change |
|---|---|---|
| wm/dispatch.rs:186 `menu_rest` | the bar the user sees (tooltip area) | `front_bar()` |
| wm/desktops.rs:70 `turn` | xp's own cube on a real screen: `cube_off()` returns early on a hosted cube, so only the flat session reaches it | kept (flat first bar) |
| wm/desktops.rs:88 `grab_cube` | same, `cube_off()` guard | kept |
| wm/desktops.rs:192 `cube_frame` | same, runs only while xp's own turn runs | kept |
| other hits (output.rs, panels.rs, choices.rs, pointer.rs, keys.rs, items.rs, press.rs) | out of scope of the brief | not touched |

Red/green: test `the_taskbar_tooltip_follows_the_front_face` (hosted, front = 1).
On d588d573's `dispatch.rs` it failed: `the tooltip at -262144,-261399 lies outside the bar of face 1 [-524288, -262144, 1280, 800]`.
Deviation: the red commit failed its cargo-fmt hook, and the retry landed with the fix in one commit (40292419); no force push is allowed, so red was shown by running the test with the old line, not as its own commit.
The test needed `wm_on_fake_core_logging` (wm.rs) and `hosted_logging` (hosted.rs) to read the scene request.

Gate: `tests/merge-gate.sh --jobs 2` full, head merged with origin/claude/autonomous-dev-session-3w-i33pg7 (already contained): 63 PASS, EXIT 0, no foreign reds.

## PM review r1 (2026-11-23 23:20 UTC)
Base d588d573..40292419, 1 commit, 3 files, +56/-5.
- rcf-11: menu_rest (dispatch.rs:186) now bounds the tooltip by front_bar(); hosted.rs test turns the cube to face 1 and checks the tip lies in face 1's bar. Gate full EXIT 0, 63 PASS (worker).
- desktops.rs:70 (turn), 88 (grab_cube), 192 (cube_frame) kept on bars.first(): checked. turn and grab_cube return early on cube_off(); cube_frame needs effects.turn, which only those two start. Correct: flat-only.
- rxf-1 MAJOR (process, rules.md 8; seeded review "a fix with no red test commit before it is MAJOR"): red test and fix share 40292419. The red was shown by hand, not as a commit.
Verdict (computed): FIX. r2 splits the commit (test, then fix) on claude/xp-front-bar-r2 from d588d573 with the same final tree; no new code.
