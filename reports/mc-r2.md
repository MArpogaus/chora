# mc r2 (rc 1, 11): mc-34 and mc-35 rest

Branch claude/mc, tip 66ca64a9 (session branch merged first, 655429b6). Captures: media/2026-11-16/mc-r2/
(light and dark, 1280x720 and 1280x800). Changed or new cells: 10 of 10 runs pass (5 dark, 5 light; the
paths gate runs dark only). tests/merge-gate.sh on 66ca64a9: all steps passed once (/tmp/merge-gate.ChdLYc),
including xp, hud, matrix, layout, mc-paths, mc-sofa.
An earlier gate run on 3c19f217 failed in `matrix` (gate-injected-presses.sh looked for the hit id
`mc-r1-footer-urgent-hit`); fixed in 66ca64a9, the gate was then run in full again.

| id | red | fix | gate cell, capture |
|---|---|---|---|
| mc-34 MAJOR | c884ed79 (unit tests `a_panel_names_the_size_of_its_text_from_14_to_48_pixels`, `a_sentence_with_a_size_is_drawn_larger_than_the_small_text_of_the_host`), e90abaaf (cell `answer`: FAIL, node 21 px high) | 0fbb751e | sofa `answer`, mc-sofa-answer-{dark,light} |
| mc-35 card | 3337875f (paths `urgent` changed first: FAIL "the card and the list do not both say the whole notice") | 3c19f217 | paths `urgent`, mc-paths-urgent |
| mc-35 overflow | 3337875f (sofa `notices`: FAIL "the Chat door [888,586,204,28] stands on the rule of KEYS [888,602,336,1]") | 3c19f217 | sofa `notices`, mc-sofa-notices-{dark,light} |
| LOW 6 (lead doc comment) | none (comment) | 3c19f217 | |
| LOW 7 (mic.rs line) | none (comment) | f3c4f030 | |

## mc-34
`size` key on a panel of hud.yaml (crates/chora-kit/src/hud.rs `Panel.size`, 14 to 48, docs/hud.md); today's value
is the default (none: Font::Ui, 11 px), so xp, tiling and construct do not change (`xp`, `hud`, `pixels` cells of the
merge gate unchanged and green). With a size, `plain()` of the host sets the sentence in the font of the chat
(Face::of) at that size, line height 147 %, wrapped at 640 px. mc sets 19 px (the size of its chat). The answer
in the capture is the serif of the page at 19 px (cell: node height >= 30 for one line, was 21).
`said` of `voice.state` is not named by the hud.yaml schema, so it is not drawn (the brief draws it only if named).

## mc-35
- The urgent notice is a card on top of the column (rule `urgent` right slot -3, below the line of the host's state
  word, which moved to slot -4), body size, red, wrapped in full, drawn close mark, a press takes it away and leaves it in
  the list. Where the column does not show (Super+n, bare scenes, presets without a right side) it stays the
  footer line (`placed`). The text of a card starts after the close mark and wraps in the room that is left.
- A full column sheds: items of `notify` carry `shed`, `fit` drops the last ones (then the title if no entry is left)
  until the flow ends one line above the foot; the rest never gives way. Unit test
  `the_oldest_notifications_give_way_when_the_side_is_full`, `a_critical_notice_stands_in_the_column_and_in_the_footer_without_one`.
  Capture: helper row plus two 90-character notices: card of 4 lines, helper row, Chat door at y 334, rule at 406.
- docs/configuration.md `notify.push` row and the comment of mc.yaml say it.

## Open
- Hidden items of `notify` are only hidden (visible false), as the planned set; nothing else in the column sheds,
  so a card of more than about 8 lines plus many helpers would still push the door onto the rule. No cap on the
  length of a notice (max_chars of `urgent`) was added.
- mc-paths `urgent` runs in the dark theme only (the gate has no THEME); the light card is in sofa `notices`.
- LOW 8 (rebuild of every scene on a side toggle) left as it is.
