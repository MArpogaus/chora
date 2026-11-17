# secfix c r2 (review-secfix-c-r1 findings 1-3, 5; head 75bc62e6, branch claude/secfix-c, 2026-11-17)

Commits: 0295f661 test (red) | f274e392 fix: refuse unshowable security changes and own HUD questions |
75bc62e6 docs: voice.md (refused set, own HUD no, security refusal, BUTTON_DELAY) + known-issues sh-27.
Red (0295f661, all 3 failed): client a_security_change_that_hides_text_is_refused (question asked);
main a_question_too_long_for_the_hud_is_refused (no answer, "a no at once" None != Some(false));
chat hides_refuses_only_what_hides_or_reorders ("⚠\u{fe0f} rm" refused). Green: voice 249, kit 208,
mc 126, construct 181 pass; clippy -D warnings clean.
F3: chora_kit::chat::hides() is one narrow check (C0 but \t\n\r, C1, U+202A-202E, U+2066-2069, U+200B,
U+2060-2064, U+FEFF); chat.rs and agents.rs use it; emoji, ZWJ, ZWNJ, NBSP, LRM, \r pass.
F1: ask_key returns Err(reason) via chat::unshowable (too_long || hides); hold_security answers the call
REJECTED with that reason and drops the change, no question. F2: main.rs hud(): without a claimer and
!mc::fits(prompt, detail, keys) it sends Input::Unshowable(id) instead of the report; core answers no
with the error earcon (sub-agent and chat questions alike). F5 doc line; F4 as known issue sh-27.
Deviations: (a) the restored red test lives in main.rs, not chat.rs: only Agent knows `claimed`.
(b) the red OSC case used a nested `phrases` object, which detail() JSON-escapes (shows \u001b, honest);
fix commit switched it to record_dir (string, raw). (c) Brief said "C0 but \n\t"; \r passes as the
brief's NOT-refused list requires. (d) a chat question refused by the own HUD reaches the LLM as
"The user declined" (answer channel is Option<bool>); not a distinct reason.
Gate: tests/merge-gate.sh, every step passed on 75bc62e6 (/tmp/merge-gate.W56Gbc).
Self-check: Unshowable only acts on the shown id while confirming; no new state; no SKIP.
Cut: plugins unchanged this round; kit gains hides(); no breach.

## Refactor proposals
- One showable gate chora_kit::chat::unshowable(detail) -> Option<Reason> used by chat, agents, client
  and main (now chat::unshowable + agents::unshowable duplicate the phrases) (S-M).
- Answer channel of chat confirms Option<bool> -> an enum with Refused(reason), so the LLM hears why (S).
- Input::ConfirmButton answers an enum (Taken, NotOpen, TooSoon) (S).
- mc.rs question_text TOO_LONG fallback is now reachable only in a claim race; drop it once claims are
  read in the mc thread (S).
- One detail builder for chat.rs:168 and agents.rs detail(); refusal phrases in config (M).
- mc agent.rs:97-133, construct hud.rs:956-1000 to kit ask::Question and Card (M, DB6).
