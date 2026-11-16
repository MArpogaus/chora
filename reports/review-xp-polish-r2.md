# Review: xp polish r2 (branch claude/xp-polish, head 5a17aab6)

Verdict: MERGE-OK (no BLOCK, no MAJOR)

Scope note: xp-25 (20fc059, red e754091) and xp-26 (64432a5, red 851c9b1) are
already ancestors of origin/claude/autonomous-dev-session-3w-i33pg7, so the
change of this branch is xp-27 only (cf78bef, e73198e, 03afb97, dde1df60) plus the
merge. The three-dot diff is noisy (multiple merge bases; the session branch
has since moved on to 9c4f1301, so the merge needs redoing before it lands).
xp-25 and xp-26 were not re-reviewed or re-run here: unverified, outside this diff.

## Evidence (CARGO_TARGET_DIR under /tmp, 19G free)
- cargo test -p chora-assistant: 230 passed, 0 failed.
- cargo clippy -p chora-assistant --all-targets -D warnings: clean.
- Red-first, rerun: tree of e73198e (tests, no fix) built and run: both
  `the_xp_card_keeps_its_keys_under_the_buttons_and_leaves_typing_to_the_field`
  and `a_balloon_on_a_small_work_area_grows_to_hold_the_card_under_its_header`
  FAIL ("the card at [286, 63] covers the header of the balloon"); both pass at HEAD. Real.
- tests/gate-xp-question.sh: 1x ok, 2x ok (own gate lock, because another
  session held /tmp/chora-gate.lock; binaries had to sit under a dir named
  `target` for the HUD lookup). Captures viewed: key words (Alt+Y/Alt+N, spoken
  yes) grey under Yes/No; "Typing does not answer." once, above the field; card
  inside the balloon at 1x and 2x. The gate cell's own red on the base: unverified
  (not built).
- Not rerun: the full merge-gate and the doors/mc failures reported by the
  builder (unverified; mc code untouched by this diff).
- No Microsoft asset: no binary/image/font file in the diff. No added `//`
  inline comments in this change (only `///` docs and the existing SAFETY lines
  from the base merge).

## Claims
- xp-27 key words under buttons: real (card.rs split_hints/under_lines, red test).
- xp-27 hint once: real (card.rs page_rows skips the note for xp; gate counts one hud-chat-hint node).
- xp-27 balloon grows: real (chat.rs Frame::card_need/lift, voice.rs passes lift, red test).
- xp-27 field follows: code present (chat/host.rs `self.chat.restyle(&frame)`), visible
  in the 2x capture (field under the card), but no test fails without it: weak coverage.
- xp-25, xp-26: in base, not part of this diff (see scope note).

## Findings
1. MINOR plugins/chora-assistant/src/chat/host.rs:302 (`restyle` call): "field follows" has no red-first
   test of its own. The gate checks card-in-balloon only, not that the field is clear of the card.
2. LOW tests/gate-xp-question.sh:55-58: the cell does not assert the key words sit under the
   buttons or that the card text lacks "Typing does not answer"; those live only in the unit test.
   An overlap check of hud-card against the field node would cover finding 1.
3. LOW commit 03afb97: one commit carries three fixes (key words, balloon growth, field follow)
   under a "key words" subject; hard to bisect. The test commit e73198e's test is also changed
   in the fix commit (adds `lift`), so its red is not byte-identical to the final test.
4. LOW at 2x the balloon covers the paperclip's head and scrolls the user line out of the
   transcript (capture xp-question-2x). Builder lists the paperclip part as open; accepted.
5. COSMETIC the builder report says "mixed subject"; fine. Report claims red-first for
   xp-25/26 on commits that are already merged; the review could only confirm ancestry.
