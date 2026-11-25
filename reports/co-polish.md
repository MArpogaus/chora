# co-polish

## r1 (lane co-polish-r1)

Branch claude/co-polish-r1, head 205aeba02.

### co-45 (MINOR): done
- red c735b1831 (test): `hud::tests::the_key_overlay_headings_are_fourteen_px`
  failed with `construct-help-body-scope left: Some(11.0) right: Some(14.0)`.
- fix c70d486d6: scope and group heads use `Type::Small` 500 (14 px), upper-cased by hand.
  construct-list-body-scope shares the same `search` builder.
- 16 known-red rows removed (construct-list-body-scope, construct-help-body-scope,
  construct-help-body-head-*); acts PASS in the full gate (205aeba02 content, EXIT 0).
- The hints contrast row (mid-fade read) is kept.

### co-44 (COSMETIC): NOT fixed, cause not in place_chip
- The test `hud::tests::chips_keep_a_whole_unit_between_them` (in c735b1831) is NOT red:
  `place_chip` already gives gaps of exactly 8 for integer widths and any right edge
  (x = round(a), next edge a-8-w, an integer shift; checked by a 200k-sample sweep). The
  cause named in the finding does not hold.
- The acts still show the gap: acts.log "construct-mode-step-in and construct-mode-turn-left
  are 9 apart" (HEADLESS-1, evidence construct-I2), so the known-red row stays.
- Lead: in draw_modes (modes.rs ~264) while stood back only StepBack is placed (the others are
  skipped), so a chip that is shown again keeps a position from an earlier draw with other
  widths or text. Needs a trace of the chip positions across draws in the act.

### Gate
Full gate `tests/merge-gate.sh --jobs 2`: all steps passed, EXIT 0
(`acts PASS 2203s`, `demo PASS`, `e2e PASS`, `all steps passed; no --push, nothing is pushed`).
No `chora --headless` process left.
