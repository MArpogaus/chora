# xp polish r3: question-card hint flake (branch claude/xp-polish, head edd0034)

## Cause
Not a lagging hint: a failed draw. voice.rs confirm() called ensure_top (which sets
self.top) but applied its commands only after chat_draw(). chat_draw saw top set and
ended the view with Move hud-top; the group did not exist yet ("command 10: node
hud-top does not exist"), the draw errored, and the card showed one step before the chat
view with its hint was drawn again. Every run logged that error; a cell polling
3 s later caught the view only when the late redraw had run, so it flaked.
Fix: apply the top group before the chat draw (voice.rs). Second, smaller lag fixed:
chat.question_of() sets the helper's state to question before the card is placed, so
hint_room and the first frame count the hint even if voice.state question arrives late.

## Commits (on c949cc84 merge 82c79e8)
- 22ed0a0 test: cell polls nodes up to 10 s, fails if the card shows without hud-chat-hint
  (state working, then confirm, question state sent after the card); unit test
  a_card_that_names_no_other_helper_has_its_hint_before_voice_says_question.
- edd0034 fix. The unit test also gains question_of(None) in the fix commit (its red
  at 22ed0a0 is the cell, 3/3 reds with "card showed without the hint (poll 3)").

## Evidence
- Red: cell at test commit plus the first fix attempt: 1x 9/10 FAIL, 2x 10/10 FAIL.
- Green at edd0034: tests/gate-xp-question.sh SCALE=1 runs 1-10 ok, SCALE=2 runs 1-10 ok.
- cargo test -p chora-assistant 231 passed; clippy --all-targets -D warnings clean.
- Not run: full tests/merge-gate.sh (cell only); r2 MINOR (restyle red test) skipped,
  not trivial. Not isolated: the cell red without question_of alone (the draw-order fix
  was what turned it green).
