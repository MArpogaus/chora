# kit4 r2 (F2): fix round after the opus security review

Model: sonnet, effort medium
Lane: cloud, branch claude/kit4 at aeb13ece (r1). One round, stop at 250 k. Cost about 0.7 %.
Run `git fetch --unshallow` first. Read reports/kit4.md (r1 and the review section the PM appends)
on claude/session-log. Fix only these, each with its red cell first:
1. MAJOR (sh-26 in construct): the 400 ms guard starts at Card::new in voice.rs:43 (follow), not
   when the card is drawn; draw_ask returns early while the HUD is hidden and remakes the card on
   resize, restyle, layers drop; Card::moved is never called. Fix: call `card.moved(now)` whenever
   draw_ask really draws, and in mc show_answers. Cell: asked while hidden, then drawn: a press at
   draw + 100 ms is TooSoon; a press at draw + 400 ms counts.
2. MINOR (mc): the guard resets when the drawer is placed again (same `moved` path).
3. MINOR (sh-27): construct hud.rs:1259-1261 hint lines ("ANSWER YES OR NO", "NEEDS A KEY PRESS
   OR A CLICK, NOT A SPOKEN YES") and mc agent.rs:564 "ja / nein" come from kit words
   (Word::NoVoiceYes and the yes/no words); a cell per look in a German session.
4. MINOR: mc cell a_press_inside_400_ms_of_the_card_is_refused injects `now` (no wall clock).
5. COSMETIC: show_answers doc comment ("Ja and Nein"); docs/configuration.md 479-481 wrap.
No other change, no new feature. Done when: cells red then green; merge gate once on the head
(poll it in the same turn; F27 apps-click known). Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and append "## r2" to
reports/kit4.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
