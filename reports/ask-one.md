# ask-one

## r1

Branch claude/ask-one, head 322eef20 (from 1c9e64f7, the session head; the
dev branch had not moved, so it was already merged). Closes DB7 a, b, c and
RF3.

| Item | Red | Fix |
|---|---|---|
| DB7 a | ac0e5e53 `an_unshowable_question_replaced_on_screen_is_still_refused` (core.rs) | 018bd4b8: core remembers `sub_shown`; `Unshowable(id)` of a sub question whose card a chat question replaced answers that question no (earcon), never touches the chat question |
| DB7 b | ae2c388a `many_short_lines_do_not_fit` (mc.rs) | bb0d382c: `QUESTION_LINES = 30`; `fits` and `question_text` use one `shows(text)` (bytes and lines of the same text) |
| DB7 c | none: test only | 0aab9538 `hold_security_rejects_a_change_that_hides_text` and `hold_security_asks_for_a_change_that_shows` (client.rs, FakeCore): the caller gets REJECTED with the refusal text, no question reaches the core, `waiting` stays empty; mutation check (Ok instead of Err in the refusal branch) turns it red |
| RF3 | none: refactor, behaviour kept | 227ed78a kit `ask::ConfirmCall` (+ `Deserialize` on `Answer`) with unit test `confirm_takes_yes_or_no_and_nothing_else`; 51e3af5a client.rs uses it, `ConfirmParams` and the local `Answer` are gone; `grep -rn ConfirmParams services crates plugins` is empty. `always` stays refused (INVALID_PARAMS) |

322eef20 renames the caller of the test call to `hud` (cut-check counts
`chora-assistant` names in voice; my first version made it 3 lines, the
first gate run failed there at 2s).

Notes
- QUESTION_LINES = 30 is a fixed guess from 18 px font, 24 px top margin and
  a 720 px screen; `question` knows the output height, `fits` (main.rs) does
  not. Refactor 3 of the review (fit check into the core) would let it use
  the real height.
- Wrapped long lines are not counted (max_width wraps them); bytes cap them.

Gate
- `cargo test -p chora-voice -p chora-kit`: all green. `cargo clippy
  --workspace -- -D warnings`: green.
- `tests/merge-gate.sh --jobs 1` on 322eef20 (dev branch 1c9e64f7 already
  inside): red-first, identity, pre-commit, gate-lib, ci-check, build, test,
  clippy, 2d-guard, package-deps, package-lists, xp, matrix, virtual,
  xp-faces, xp-paperclip, xp-fields, xp-question, xp-notepad, hud, layout,
  tiling-audit PASS. **tiling-look FAIL (foreign)**: `FAIL: paper-1280x800
  bar-font: the clock is 120.0 wide, not Inter 14 px weight 500 (100-119)` and
  `FAIL: paper-2560x1600@2 bar-font: the clock is 121.0 wide ...`
  (tiling bar font metrics in this cloud image; no tiling, font or look file
  is in my diff: only crates/chora-kit/src/ask.rs and services/voice/src).
  The gate stops at the first red, so the rest ran with `--only` (not rerun
  to green): tiling-empty, chat-keys, doors, acts, last-words, cube, pull,
  placed, construct-hud, construct-extra, out, winit, wm-pane, pane-keys,
  walk, keys, pixels, mc-paths, mc-sofa, mc-column, mc-handover,
  launcher-click, voice-acp, voice-events, agent, e2e: all PASS ("partial
  run: nothing is pushed").
